//! Lock‑free step packet encoder for the Klipper protocol.
//!
//! Each packet encodes a single step instruction for a given stepper motor.
//! The binary layout (little endian) is:
//!   u32 timestamp_us   – MCU timer value (microseconds)
//!   u16 interval_us    – Time since previous step (microseconds)
//!   u8  stepper_id    – Identifier for the stepper (0‑255)
//!   u8  direction     – 0 = forward, 1 = backward
//!   u16 crc16          – CRC‑16‑CCITT (poly 0x1021) over the first 8 bytes
//!
//! The encoder is designed to be used from interrupt context without heap
//! allocation. It pushes encoded packets into a pre‑allocated `heapless::spsc::Queue`.
//! The queue is defined by the caller; this module only provides the encoding
//! routine and CRC calculation.

use crc::{Crc, CRC_16_IBM_3740};
use heapless::spsc::Queue;

/// Fixed‑size packet according to the protocol layout.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepPacket {
    pub timestamp_us: u32,
    pub interval_us: u16,
    pub stepper_id: u8,
    pub direction: u8,
    pub crc16: u16,
}

impl StepPacket {
    /// Encode fields and compute the CRC.
    #[inline]
    pub fn new(timestamp_us: u32, interval_us: u16, stepper_id: u8, direction: u8) -> Self {
        let mut pkt = StepPacket {
            timestamp_us,
            interval_us,
            stepper_id,
            direction,
            crc16: 0,
        };
        // Compute CRC over the first 8 bytes (exclude crc16 itself).
        let crc = Crc::<u16>::new(&CRC_16_IBM_3740);
        let bytes: &[u8] = unsafe { core::slice::from_raw_parts(&pkt as *const _ as *const u8, 8) };
        pkt.crc16 = crc.checksum(bytes);
        pkt
    }

    /// Validate the CRC of a received packet.
    #[inline]
    pub fn validate(&self) -> bool {
        let crc = Crc::<u16>::new(&CRC_16_IBM_3740);
        let bytes: &[u8] = unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, 8) };
        crc.checksum(bytes) == self.crc16
    }
}

/// Encoder that pushes packets into a lock‑free SPSC queue.
pub struct StepEncoder {
    queue: Queue<StepPacket, 1024>, // capacity 1024 packets – adjust as needed.
    last_timestamp: u32,
}

impl StepEncoder {
    /// Create a new encoder with an empty queue.
    pub fn new() -> Self {
        StepEncoder {
            queue: Queue::new(),
            last_timestamp: 0,
        }
    }

    /// Encode a new step and push it into the queue.
    /// Returns `Ok(())` if the packet was enqueued, `Err(())` if the queue is full.
    pub fn encode(&mut self, timestamp_us: u32, stepper_id: u8, direction: u8) -> Result<(), ()> {
        let interval = if self.last_timestamp == 0 {
            0
        } else {
            timestamp_us.wrapping_sub(self.last_timestamp) as u16
        };
        self.last_timestamp = timestamp_us;
        let pkt = StepPacket::new(timestamp_us, interval, stepper_id, direction);
        self.queue.enqueue(pkt).map_err(|_| ())
    }

    /// Attempt to dequeue a packet for transmission.
    pub fn try_dequeue(&mut self) -> Option<StepPacket> {
        self.queue.dequeue()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_crc_roundtrip() {
        let pkt = StepPacket::new(123_456, 250, 3, 0);
        assert!(pkt.validate());
    }

    #[test]
    fn encoder_queueing() {
        let mut enc = StepEncoder::new();
        assert!(enc.encode(1_000, 1, 0).is_ok());
        assert!(enc.encode(1_250, 1, 0).is_ok());
        let p1 = enc.try_dequeue().expect("first packet");
        let p2 = enc.try_dequeue().expect("second packet");
        assert_eq!(p1.interval_us, 0);
        assert_eq!(p2.interval_us, 250);
    }
}