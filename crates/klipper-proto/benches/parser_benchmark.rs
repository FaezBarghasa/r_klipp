use criterion::{black_box, criterion_group, criterion_main, Criterion};
use klipper_proto::{crc::crc16_ccitt, parser::{Parser, SYNC_BYTE}};

/// Helper to construct a valid Klipper message frame for testing.
fn build_bench_frame(seq: u8, command_id: u8, payload: &[u8]) -> Vec<u8> {
    let msg_len = 1 (seq) + 1 (cmd_id) + payload.len() + 2 (crc);
    let mut frame = Vec::with_capacity(msg_len + 1);
    frame.push(SYNC_BYTE);
    frame.push(msg_len as u8);
    frame.push(seq);
    frame.push(command_id);
    frame.extend_from_slice(payload);

    let crc = crc16_ccitt(&frame[1..], msg_len - 1);
    frame.extend_from_slice(&crc.to_be_bytes());
    frame
}


fn benchmark_parse(c: &mut Criterion) {
    let parser = Parser::new();
    let payload = [
        0x00, 0x00, 0x27, 0x10, // interval = 10000
        0x00, 0x01, // count = 1
        0x00, 0x00, // add = 0
    ];
    let frame = build_bench_frame(1, 0x10, &payload);

    c.bench_function("parse_queue_step", |b| {
        b.iter(|| {
            // Black box prevents the compiler from optimizing away the operation
            let _ = parser.parse(black_box(&frame));
        })
    });
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
