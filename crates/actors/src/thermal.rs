use hal::adc::Adc;
use hal::pwm::Pwm;
use comms::Message;

pub struct ThermalActor<ADC, PWM>
where
    ADC: Adc<u16>,
    PWM: Pwm,
{
    adc: ADC,
    pwm: PWM,
    // Add other state variables here
}

impl<ADC, PWM> ThermalActor<ADC, PWM>
where
    ADC: Adc<u16>,
    PWM: Pwm,
{
    pub fn new(adc: ADC, pwm: PWM) -> Self {
        Self { adc, pwm }
    }

    pub async fn run(&mut self) {
        loop {
            // 1. Read temperature from ADC
            // let temp = self.adc.read(&mut self.adc_channel).await.unwrap();

            // 2. Calculate MPC output
            // let output = self.calculate_mpc(temp);

            // 3. Set PWM duty cycle
            // self.pwm.set_duty_cycle(self.pwm_channel, output).await.unwrap();

            // 4. Handle messages from the message bus
            // self.handle_messages().await;

            // For now, just yield
            embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
        }
    }

    async fn handle_messages(&mut self) {
        // Poll the message bus for relevant messages
    }
}
