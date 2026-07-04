pub trait MinimalHal {
    fn system_tick() -> u64;
    fn gpio_set(&mut self, pin: u8, state: bool);
    fn gpio_get(&self, pin: u8) -> bool;
    fn delay_us(&self, us: u32);
}

pub struct BitBangStepGenerator<M: MinimalHal> {
    hal: M,
    step_pin: u8,
    dir_pin: u8,
}

impl<M: MinimalHal> BitBangStepGenerator<M> {
    pub fn new(hal: M, step_pin: u8, dir_pin: u8) -> Self {
        Self { hal, step_pin, dir_pin }
    }

    pub fn step(&mut self, dir: bool) {
        self.hal.gpio_set(self.dir_pin, dir);
        self.hal.gpio_set(self.step_pin, true);
        self.hal.delay_us(1);
        self.hal.gpio_set(self.step_pin, false);
        self.hal.delay_us(1);
    }
}
