//! Implementation for the TMC2209 stepper motor driver.

use crate::Error;
use bitfield::bitfield;
use embedded_hal::{digital::OutputPin, serial::Write};

// Sync byte and slave address for UART communication
const SYNC: u8 = 0x05;
const MASTER_ADDR: u8 = 0x00;

/// Represents the slave address of the TMC2209.
#[derive(Clone, Copy)]
pub enum SlaveAddress {
    Default = 0b00,
    Addr1 = 0b01,
    Addr2 = 0b10,
    Addr3 = 0b11,
}

/// Register addresses for the TMC2209.
#[repr(u8)]
enum Register {
    GCONF = 0x00,
    IOIN = 0x06,
    IHOLDIRUN = 0x10,
    CHOPCONF = 0x6C,
}

bitfield! {
    /// Global Configuration Register (GCONF)
    struct GConf(u32);
    impl Debug;
    u32;
    pdn_disable, set_pdn_disable: 10;
}

bitfield! {
    /// Chopper Configuration Register (CHOPCONF)
    struct ChopConf(u32);
    impl Debug;
    u32;
    mres, set_mres: 27, 24;
    dedge, set_dedge: 29;
    intpol, set_intpol: 28;
}

/// Microstep resolution settings.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Microsteps {
    Steps256 = 0,
    Steps128 = 1,
    Steps64 = 2,
    Steps32 = 3,
    Steps16 = 4,
    Steps8 = 5,
    Steps4 = 6,
    Steps2 = 7,
    FullStep = 8,
}

/// Driver for the TMC2209.
pub struct Tmc2209<SERIAL> {
    serial: SERIAL,
    slave_addr: u8,
}

impl<SERIAL, E> Tmc2209<SERIAL>
where
    SERIAL: Write<u8, Error = E>,
{
    /// Creates a new TMC2209 driver instance.
    pub fn new(serial: SERIAL, address: SlaveAddress) -> Self {
        Self {
            serial,
            slave_addr: address as u8,
        }
    }

    /// Enables the driver by pulling the EN pin low.
    pub fn enable<PIN, PE>(&mut self, en_pin: &mut PIN) -> Result<(), PE>
    where
        PIN: OutputPin<Error = PE>,
    {
        en_pin.set_low()
    }

    /// Disables the driver by pulling the EN pin high.
    pub fn disable<PIN, PE>(&mut self, en_pin: &mut PIN) -> Result<(), PE>
    where
        PIN: OutputPin<Error = PE>,
    {
        en_pin.set_high()
    }

    /// Sets the run current (0-31).
    pub fn set_run_current(&mut self, current: u8) -> Result<(), Error<E>> {
        self.write_register(Register::IHOLDIRUN, (current as u32 & 0x1F) << 8)
    }

    /// Sets the hold current (0-31).
    pub fn set_hold_current(&mut self, current: u8) -> Result<(), Error<E>> {
        self.write_register(Register::IHOLDIRUN, current as u32 & 0x1F)
    }

    /// Configures the UART for single-driver operation.
    pub fn set_pdn_disable(&mut self, disable: bool) -> Result<(), Error<E>> {
        self.write_register(Register::GCONF, (disable as u32) << 4)
    }

    /// Sets the microstep resolution.
    pub fn set_microsteps(&mut self, mres: Microsteps) -> Result<(), Error<E>> {
        let mut chop_conf = ChopConf(0);
        chop_conf.set_mres(mres as u32);
        chop_conf.set_intpol(1); // Enable interpolation
        self.write_register(Register::CHOPCONF, chop_conf.0)
    }

    /// Helper to write to a register via UART.
    fn write_register(&mut self, reg_addr: Register, data: u32) -> Result<(), Error<E>> {
        let mut datagram: [u8; 8] = [0; 8];
        datagram[0] = SYNC;
        datagram[1] = self.slave_addr;
        datagram[2] = 0x80 | (reg_addr as u8); // Write access
        datagram[3..7].copy_from_slice(&data.to_be_bytes());
        datagram[7] = self.calculate_crc(&datagram[0..7]);

        self.serial
            .write(&datagram)
            .map_err(Error::Serial)
    }

    /// Calculates the CRC8 for a given datagram.
    fn calculate_crc(&self, datagram: &[u8]) -> u8 {
        let mut crc: u8 = 0;
        for byte in datagram {
            let mut current_byte = *byte;
            for _ in 0..8 {
                if (crc >> 7) ^ (current_byte & 0x01) != 0 {
                    crc = (crc << 1) ^ 0x07;
                } else {
                    crc <<= 1;
                }
                current_byte >>= 1;
            }
        }
        crc
    }
}
