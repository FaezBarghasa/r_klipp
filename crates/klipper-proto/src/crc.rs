#![deny(clippy::all)]
#![deny(warnings)]

//! CRC-16-CCITT implementation for Klipper protocol message integrity.

/// Calculates the CRC-16-CCITT checksum.
///
/// This is a `const fn` implementation, allowing it to be used in compile-time
/// contexts if needed. It matches the algorithm used by the C Klipper firmware.
///
/// # Arguments
/// * `data` - The byte slice to checksum.
/// * `len` - The number of bytes in the slice to process.
pub const fn crc16_ccitt(data: &[u8], len: usize) -> u16 {
    let mut crc: u16 = 0xFFFF;
    let mut i = 0;
    while i < len {
        crc ^= (data[i] as u16) << 8;
        let mut j = 0;
        while j < 8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            j += 1;
        }
        i += 1;
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_ccitt_klipper_vector() {
        // Test vector from Klipper's C implementation (`command.c`)
        // For the message `identify {"version": "123"}`
        // The frame (excluding sync and crc) is:
        // len=21, seq=1, cmd_id=1, payload...
        let data: [u8; 21] = [
            0x15, 0x01, 0x01, 0x83, 0xa7, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0xa3, 0x31,
            0x32, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(crc16_ccitt(&data, data.len()), 0x343D);
    }

    #[test]
    fn test_crc16_ccitt_standard_vector() {
        // Standard test vector "123456789"
        let data = b"123456789";
        assert_eq!(crc16_ccitt(data, data.len()), 0x2189);
    }
}
