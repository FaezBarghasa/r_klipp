use crate::traits::QuadratureEncoder;

pub struct Stm32QuadratureEncoder {
    count: i32,
}

impl QuadratureEncoder for Stm32QuadratureEncoder {
    type Error = core::convert::Infallible;
    fn read_count(&self) -> i32 {
        self.count
    }

    fn reset_count(&mut self) {
        self.count = 0;
    }
}

