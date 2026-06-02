use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mcu_drivers::stepper::{StepSegment, StepperController};

fn benchmark_enqueue(c: &mut Criterion) {
    let mut controller = StepperController::<256>::new(1);
    let segment = StepSegment {
        interval_ticks: 1234,
        direction: true,
        enable_mask: 0b0001,
    };

    c.bench_function("stepper_enqueue", |b| {
        b.iter(|| {
            let _ = controller.enqueue_segment(black_box(segment));
        })
    });
}

criterion_group!(benches, benchmark_enqueue);
criterion_main!(benches);
