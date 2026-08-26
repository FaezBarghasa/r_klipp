use bytes::{Buf, BufMut, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub const ETX_DELIMITER: u8 = 0x03;

/// Framed Codec for Klipper's \x03 (ETX) delimited JSON stream.
#[derive(Debug, Default, Clone)]
pub struct EtxCodec {
    max_frame_len: usize,
}

impl EtxCodec {
    pub fn new() -> Self {
        Self {
            max_frame_len: 16 * 1024 * 1024, // 16MB max frame limit
        }
    }
}

impl Decoder for EtxCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find the index of \x03 delimiter
        if let Some(pos) = src.iter().position(|&b| b == ETX_DELIMITER) {
            if pos > self.max_frame_len {
                src.advance(pos + 1);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Frame length exceeded maximum allowed size",
                ));
            }

            let frame = src.split_to(pos).to_vec();
            src.advance(1); // Consume the \x03 delimiter
            Ok(Some(frame))
        } else {
            if src.len() > self.max_frame_len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Incomplete frame exceeded maximum size",
                ));
            }
            Ok(None)
        }
    }
}

impl Encoder<Vec<u8>> for EtxCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len() + 1);
        dst.put_slice(&item);
        dst.put_u8(ETX_DELIMITER);
        Ok(())
    }
}

impl Encoder<&str> for EtxCodec {
    type Error = io::Error;

    fn encode(&mut self, item: &str, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len() + 1);
        dst.put_slice(item.as_bytes());
        dst.put_u8(ETX_DELIMITER);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etx_codec_single_frame() {
        let mut codec = EtxCodec::new();
        let mut buf = BytesMut::from(&b"{\"id\":1,\"result\":\"ok\"}\x03"[..]);

        let decoded = codec.decode(&mut buf).unwrap().expect("should decode frame");
        assert_eq!(decoded, b"{\"id\":1,\"result\":\"ok\"}");
        assert!(buf.is_empty());
    }

    #[test]
    fn test_etx_codec_multiple_frames_in_single_buffer() {
        let mut codec = EtxCodec::new();
        let mut buf = BytesMut::from(&b"{\"msg\":1}\x03{\"msg\":2}\x03"[..]);

        let f1 = codec.decode(&mut buf).unwrap().expect("frame 1");
        assert_eq!(f1, b"{\"msg\":1}");

        let f2 = codec.decode(&mut buf).unwrap().expect("frame 2");
        assert_eq!(f2, b"{\"msg\":2}");

        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn test_etx_codec_encode() {
        let mut codec = EtxCodec::new();
        let mut buf = BytesMut::new();

        codec.encode("{\"id\":1}".as_bytes().to_vec(), &mut buf).unwrap();
        assert_eq!(buf.as_ref(), b"{\"id\":1}\x03");
    }
}
