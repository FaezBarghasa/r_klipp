use criterion::{black_box, criterion_group, criterion_main, Criterion};
use motion::smart_controller::{SmartController, ControlMode};
use motion::tuner_trait::TuningDomain;
use motion::motion_tuner::MotionStepTuner;

fn pid_benchmark(c: &mut Criterion) {
    let mut smart_controller = SmartController::new(TuningDomain::Temperature, 1.0, 0.1, 0.01);
    c.bench_function("pid_compute", |b| {
        b.iter(|| smart_controller.compute(black_box(25.0), black_box(1.0)))
    });
}

fn motion_tuner_benchmark(c: &mut Criterion) {
    let mut motion_tuner = MotionStepTuner::new(100.0, 50.0);
    c.bench_function("motion_tuner_analysis", |b| {
        b.iter(|| motion_tuner.update(black_box(50.0), black_box(50.0), black_box(5.0)))
    });
}

criterion_group!(benches, pid_benchmark, motion_tuner_benchmark);
criterion_main!(benches);