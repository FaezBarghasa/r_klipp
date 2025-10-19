//! Pin mappings for the MKS SKIPR board.
//! This is a starting point and may need to be adjusted based on your specific setup.

use embassy_stm32::gpio::{AnyPin, Pin};

/// Struct to hold all the board pins.
pub struct BoardPins {
    pub led: AnyPin,
    pub stepper_x_step: AnyPin,
    pub stepper_x_dir: AnyPin,
    pub stepper_y_step: AnyPin,
    pub stepper_y_dir: AnyPin,
    pub temp_extruder: AnyPin,
    pub temp_bed: AnyPin,
    pub heater_extruder: AnyPin,
    pub heater_bed: AnyPin,
    pub uart_tx: AnyPin,
    pub uart_rx: AnyPin,
}

impl BoardPins {
    /// Creates a new BoardPins struct for the MKS SKIPR.
    pub fn new(p: embassy_stm32::Peripherals) -> Self {
        use embassy_stm32::gpio::Pin as _;

        // Note: These are example pins. You will need to verify them against the MKS SKIPR schematic.
        Self {
            led: p.PB1.degrade(),
            stepper_x_step: p.PE2.degrade(),
            stepper_x_dir: p.PB8.degrade(),
            stepper_y_step: p.PC5.degrade(),
            stepper_y_dir: p.PB2.degrade(),
            temp_extruder: p.PA0.degrade(),
            temp_bed: p.PA1.degrade(),
            heater_extruder: p.PC8.degrade(),
            heater_bed: p.PC9.degrade(),
            uart_tx: p.PA9.degrade(),
            uart_rx: p.PA10.degrade(),
        }
    }
}
