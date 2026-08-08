//! A simple thermal simulation to demonstrate and tune the PID controller.

use thermal::filter::{Filter, LowPassFilter};
use thermal::pid::Pid;
use thermal::safety::{HeaterSafety, SafetyLimits, ThermalState};

/// A simple model of a heater and its environment.
struct HeaterModel {
    /// Current temperature of the heater block.
    temperature: f64,
    /// Ambient temperature of the environment.
    ambient_temp: f64,
    /// A factor representing how quickly the heater heats up.
    heat_coefficient: f64,
    /// A factor representing how quickly it cools down to ambient.
    cooling_coefficient: f64,
}

impl HeaterModel {
    fn new(ambient_temp: f64) -> Self {
        Self {
            temperature: ambient_temp,
            ambient_temp,
            heat_coefficient: 2.5,    // Degrees C per second at full power
            cooling_coefficient: 0.1, // Degrees C per second per degree above ambient
        }
    }

    /// Update the model's temperature based on heater output.
    fn update(&mut self, heater_output: f64, dt: f64) {
        // Heating effect
        let heating = heater_output * self.heat_coefficient * dt;

        // Cooling effect (Newton's law of cooling)
        let cooling = (self.temperature - self.ambient_temp) * self.cooling_coefficient * dt;

        self.temperature += heating - cooling;
    }
}

fn main() {
    let ambient_temp = 25.0;
    let setpoint = 210.0;
    let sim_time_s = 180.0;
    let dt = 0.5;

    let mut model = HeaterModel::new(ambient_temp);
    let mut pid = Pid::new(
        10.0, // Kp
        0.1,  // Ki
        5.0,  // Kd
        setpoint,
        0.0,  // Min output
        1.0,  // Max output (0% to 100%)
    );

    let safety_limits = SafetyLimits {
        max_temp: 280.0,
        min_heat_gain_temp: 5.0,
        min_heat_gain_time_s: 20.0,
        max_deviation: 15.0,
    };
    let mut safety = HeaterSafety::new(safety_limits);

    let mut adc_filter = LowPassFilter::new(0.5, ambient_temp);

    println!("Time (s), Setpoint (°C), Temp (°C), PID Out, Safety State");

    let mut current_time = 0.0;
    while current_time < sim_time_s {
        // Add some simulated noise to the temperature reading
        let noisy_temp = model.temperature + (rand::random::<f64>() - 0.5) * 0.5;
        adc_filter.add_sample(noisy_temp);
        let filtered_temp = adc_filter.output();

        let output = pid.update(filtered_temp, dt);
        model.update(output, dt);

        let heater_on = output > 0.0;
        let state = safety.update(current_time, model.temperature, setpoint, heater_on);

        println!(
            "{:.1}, {:.1}, {:.2}, {:.3}, {:?}",
            current_time, setpoint, model.temperature, output, state
        );

        if state == ThermalState::Shutdown {
            println!("\nSAFETY SHUTDOWN TRIGGERED!");
            break;
        }

        current_time += dt;
    }
}
