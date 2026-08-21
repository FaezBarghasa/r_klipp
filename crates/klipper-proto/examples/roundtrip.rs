//! Roundtrip packet encoding and decoding example.

use klipper_proto::codec::{StepEncoder, StepPacket};

fn main() {
    let mut encoder = StepEncoder::new();
    encoder.encode(1000, 1, 0).expect("failed to encode");

    if let Some(pkt) = encoder.try_dequeue() {
        println!("Dequeued packet: {:?}", pkt);
        assert!(pkt.validate());
    }
}
