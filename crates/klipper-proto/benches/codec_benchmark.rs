use criterion::{criterion_group, criterion_main, Criterion, black_box};
use klipper_proto::codec::KlipperCodec;
use klipper_proto::commands::{Command, Message};
use bytes::BytesMut;
use tokio_util::codec::{Encoder, Decoder};

fn benchmark_encode(c: &mut Criterion) {
    let mut codec = KlipperCodec::new();
    let message = Message::Command(Command::GCode("G1 X10.5 Y20.3 Z30.1 F3000".into()));
    let mut buffer = BytesMut::with_capacity(256);

    c.bench_function("encode_gcode", |b| {
        b.iter(|| {
            buffer.clear();
            // Black box prevents the compiler from optimizing away the operation
            codec.encode(black_box(message.clone()), &mut buffer).unwrap();
        })
    });
}

fn benchmark_decode(c: &mut Criterion) {
    let mut codec = KlipperCodec::new();
    let message = Message::Command(Command::GCode("G1 X10.5 Y20.3 Z30.1 F3000".into()));
    let mut buffer = BytesMut::with_capacity(256);
    codec.encode(message, &mut buffer).unwrap();

    c.bench_function("decode_gcode", |b| {
        b.iter(|| {
            let mut buf_clone = buffer.clone();
            // Black box prevents the compiler from optimizing away the operation
            let _ = codec.decode(black_box(&mut buf_clone)).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_encode, benchmark_decode);
criterion_main!(benches);

