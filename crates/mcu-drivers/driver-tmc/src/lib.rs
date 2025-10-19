//! A `no_std` driver for Trinamic stepper motor drivers.
//!
//! This crate provides a hardware-agnostic interface to configure and control
//! TMC stepper drivers, primarily targeting the TMC2209 over UART. It is built
//! upon the `embedded-hal` traits and can be tested with `embedded-hal-mock`.
//!
//! ## Features
//! - `tmc2209`: Enables implementation for the TMC2209 driver.
//! - `tmc2208`: Enables implementation for the TMC2208 driver.
//! - `std`: Required for running host-based tests.

#![no_std]

#[cfg(feature = "tmc2209")]
pub mod tmc2209;

#[cfg(feature = "tmc2208")]
pub mod tmc2208;

/// Represents errors that can occur while interacting with a TMC driver.
#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// An error occurred during serial communication (UART/SPI).
    Serial(E),
    /// The CRC checksum in a received message was invalid.
    InvalidCrc,
    /// The operation is not supported by the driver variant.
    NotSupported,
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    use embedded_hal_mock::pin::{Mock as PinMock, State as PinState, Transaction as PinTransaction};

    #[cfg(feature = "tmc2209")]
    use tmc2209::{Tmc2209, SlaveAddress};
    #[cfg(feature = "tmc2208")]
    use tmc2208::{Tmc2208, SlaveAddress as Tmc2208SlaveAddress};

    /// Tests the initialization sequence for a TMC2209 driver.
    #[test]
    #[cfg(feature = "tmc2209")]
    fn test_tmc2209_init_and_enable() {
        // Mock a UART peripheral and an EN pin
        let mut serial = SerialMock::new(&[
            // Set GCONF `pdn_disable` to true
            SerialTransaction::write_many(vec![0x05, 0x00, 0x00, 0xC0, 0x01, 0x04, 0x31, 0xF9]),
            // Set IHOLD_IRUN
            SerialTransaction::write_many(vec![0x05, 0x00, 0x00, 0xC0, 0x10, 0x00, 0x0F, 0x0F, 0x29, 0x4D]),
            // Set CHOPCONF `mres` to 16 microsteps and enable driver
            SerialTransaction::write_many(vec![0x05, 0x00, 0x00, 0xC0, 0x6C, 0x10, 0x00, 0x00, 0xC3, 0xB0, 0xF9]),
        ]);

        let mut en_pin = PinMock::new(&[
            PinTransaction::set(PinState::Low), // Enable driver
        ]);

        // Create the driver instance
        let mut driver = Tmc2209::new(serial.clone(), SlaveAddress::Default);

        // Enable the driver and configure it
        driver.set_pdn_disable(true).unwrap();
        driver.set_run_current(15).unwrap();
        driver.set_hold_current(15).unwrap();
        driver.set_microsteps(tmc2209::Microsteps::Steps16).unwrap();
        driver.enable(&mut en_pin).unwrap();

        // Verify all mock expectations were met
        serial.done();
        en_pin.done();
    }

    /// Tests the initialization sequence for a TMC2208 driver.
    #[test]
    #[cfg(feature = "tmc2208")]
    fn test_tmc2208_init_and_enable() {
        // Mock a UART peripheral and an EN pin
        let mut serial = SerialMock::new(&[
            // Set GCONF `pdn_disable` to true
            SerialTransaction::write_many(vec![0x05, 0x00, 0x00, 0xC0, 0x01, 0x04, 0x31, 0xF9]),
            // Set IHOLD_IRUN
            SerialTransaction::write_many(vec![0x05, 0x00, 0x00, 0xC0, 0x10, 0x00, 0x0F, 0x0F, 0x29, 0x4D]),
            // Set CHOPCONF `mres` to 16 microsteps and enable driver
            SerialTransaction::write_many(vec![0x05, 0x00, 0x00, 0xC0, 0x6C, 0x10, 0x00, 0x00, 0xC3, 0xB0, 0xF9]),
        ]);

        let mut en_pin = PinMock::new(&[
            PinTransaction::set(PinState::Low), // Enable driver
        ]);

        // Create the driver instance
        let mut driver = Tmc2208::new(serial.clone(), Tmc2208SlaveAddress::Default);

        // Enable the driver and configure it
        driver.set_pdn_disable(true).unwrap();
        driver.set_run_current(15).unwrap();
        driver.set_hold_current(15).unwrap();
        driver.set_microsteps(tmc2208::Microsteps::Steps16).unwrap();
        driver.enable(&mut en_pin).unwrap();

        // Verify all mock expectations were met
        serial.done();
        en_pin.done();
    }
}

