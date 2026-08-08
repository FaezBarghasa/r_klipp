
pub trait MathAccelerator {
    fn sin_cos(&self, angle: f32) -> (f32, f32);
    fn atan2(&self, y: f32, x: f32) -> f32;
}

#[cfg(any(feature = "stm32g4", feature = "stm32h7", feature = "stm32u5"))]
pub struct Cordic {
    // embassy_stm32::cordic::Cordic
}

#[cfg(any(feature = "stm32g4", feature = "stm32h7", feature = "stm32u5"))]
impl MathAccelerator for Cordic {
    fn sin_cos(&self, angle: f32) -> (f32, f32) {
        // Call hardware CORDIC
        micromath::F32Ext::sin_cos(angle)
    }

    fn atan2(&self, y: f32, x: f32) -> f32 {
        // Call hardware CORDIC
        micromath::F32Ext::atan2(y, x)
    }
}

pub struct SoftwareMath;

impl MathAccelerator for SoftwareMath {
    fn sin_cos(&self, angle: f32) -> (f32, f32) {
        micromath::F32Ext::sin_cos(angle)
    }

    fn atan2(&self, y: f32, x: f32) -> f32 {
        micromath::F32Ext::atan2(y, x)
    }
}
