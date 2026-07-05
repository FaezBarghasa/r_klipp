use embassy_executor::task;
use embassy_time::{Instant, Timer};
use heapless::spsc::{Receiver, Sender};

use crate::hal::traits::Adc;
use crate::hal::traits::Pwm;

enum AutotuneState {
    Idle,
    Heating,
    Cooling,
    Done,
}

pub struct UpRelayAutotuner {
    setpoint: f32,
    hysteresis: f32,
    output: f32,
    state: AutotuneState,
    cycle_count: u32,
    target_cycles: u32,
    peak_temps: [f32; 5],
    peak_times: [f32; 5],
    last_time: f32,
}

impl UpRelayAutotuner {
    pub fn new(setpoint: f32, hysteresis: f32, cycles: u32) -> Self {
        Self {
            setpoint,
            hysteresis,
            output: 1.0,
            state: AutotuneState::Idle,
            cycle_count: 0,
            target_cycles: cycles.min(5),
            peak_temps: [0.0; 5],
            peak_times: [0.0; 5],
            last_time: 0.0,
        }
    }

    pub fn tune(&mut self, temp: f32, time: f32) -> bool {
        match self.state {
            AutotuneState::Idle => {
                if temp < self.setpoint {
                    self.state = AutotuneState::Heating;
                    self.output = 1.0;
                }
            }
            AutotuneState::Heating => {
                if temp > self.setpoint + self.hysteresis {
                    self.state = AutotuneState::Cooling;
                    self.output = 0.0;
                    self.peak_temps[self.cycle_count as usize] = temp;
                    self.peak_times[self.cycle_count as usize] = time - self.last_time;
                    self.last_time = time;
                }
            }
            AutotuneState::Cooling => {
                if temp < self.setpoint - self.hysteresis {
                    self.state = AutotuneState::Heating;
                    self.output = 1.0;
                    self.cycle_count += 1;
                    if self.cycle_count >= self.target_cycles {
                        self.state = AutotuneState::Done;
                        return true;
                    }
                }
            }
            AutotuneState::Done => return true,
        }
        false
    }

    pub fn get_output(&self) -> f32 {
        self.output
    }

    pub fn get_tunings(&self) -> (f32, f32, f32) {
        let a = self.peak_temps.iter().sum::<f32>() / self.cycle_count as f32;
        let t = self.peak_times.iter().sum::<f32>() / self.cycle_count as f32;
        let d = self.output;

        let ku = (4.0 * d) / (core::f32::consts::PI * a);
        let tu = t;

        // Ziegler-Nichols PID gains
        let kp = 0.6 * ku;
        let ki = 1.2 * ku / tu;
        let kd = 0.075 * ku * tu;

        (kp, ki, kd)
    }
}

pub struct TuneCommand {
    pub setpoint: f32,
    pub hysteresis: f32,
    pub cycles: u32,
}

#[task]
pub async fn thermal_tune_actor(
    mut command_receiver: Receiver<'static, TuneCommand, 1>,
    mut result_sender: Sender<'static, (f32, f32, f32), 1>,
    mut temp_sensor: impl Adc<u16> + 'static,
    mut heater_pwm: impl Pwm + 'static,
) {
    loop {
        if let Some(command) = command_receiver.dequeue() {
            let mut autotuner = UpRelayAutotuner::new(command.setpoint, command.hysteresis, command.cycles);
            let start_time = Instant::now();

            loop {
                let now = start_time.elapsed().as_secs_f32();
                let temp = temp_sensor.read().await.unwrap_or(0) as f32;

                if autotuner.tune(temp, now) {
                    let tunings = autotuner.get_tunings();
                    result_sender.enqueue(tunings).ok();
                    break;
                }

                heater_pwm.set_duty_cycle_percent(autotuner.get_output()).await.ok();

                Timer::after_millis(100).await;
            }
        }
        Timer::after_millis(100).await;
    }
}