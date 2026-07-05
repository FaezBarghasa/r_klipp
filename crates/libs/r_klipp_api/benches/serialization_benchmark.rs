use criterion::{black_box, criterion_group, criterion_main, Criterion};
use r_klipp_api::{LinkHealth, McuToHost};
use postcard::to_vec;
use heapless::Vec;

fn benchmark_telemetry_serialization(c: &mut Criterion) {
    let telemetry = McuToHost::Telemetry {
        pos: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        temps: [25.0, 26.0, 27.0, 28.0],
        currents: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
        link_health: LinkHealth {
            rtt_us: 1500,
            buffer_fill_percent: 75,
            dropped_packets: 1,
        },
    };

    c.bench_function("telemetry_serialization", |b| {
        b.iter(|| {
            let _ = to_vec::<_, Vec<u8, 256>>(black_box(&telemetry));
        })
    });
}

criterion_group!(benches, benchmark_telemetry_serialization);
criterion_main!(benches);
