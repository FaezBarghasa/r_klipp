use criterion::{criterion_group, criterion_main, Criterion};
use mcu_drivers::stepper::{StepSegment, StepperController};

fn bench_stepper_isr_wcet(c: &mut Criterion) {
    let mut controller = StepperController::<256>::new(1);

    // Pre-fill the controller's internal queue to simulate a real scenario
    for _ in 0..10 {
        let _ = controller.enqueue_segment(StepSegment {
            interval_ticks: 1000,
            direction: true,
            enable_mask: 1,
        });
    }

    let mut bsrr: u32 = 0;
    let mut arr: u32 = 0;

    c.bench_function("stepper_isr_wcet", |b| {
        b.iter(|| {
            unsafe {
                controller.execute_next_step_isr(&mut bsrr as *mut u32, &mut arr as *mut u32);
            }
        })
    });
}

criterion_group!(benches, bench_stepper_isr_wcet);
criterion_main!(benches);
