use criterion::{black_box, criterion_group, criterion_main, Criterion};
use heapless::spsc::Queue;
use mcu_drivers::stepper::StepCommand;

// Static queue for benchmarks
static mut BENCH_QUEUE: Queue<StepCommand, 256> = Queue::new();

fn benchmark_enqueue(c: &mut Criterion) {
    let (mut producer, _consumer) = unsafe {
        BENCH_QUEUE.clear();
        BENCH_QUEUE.split()
    };

    let cmd = StepCommand {
        stepper_mask: 0b0001,
        direction_mask: 0b0001,
        interval_ticks: 1234,
    };

    c.bench_function("stepper_enqueue", |b| {
        b.iter(|| {
            // The queue will fill up, but enqueue on a full SPSC queue is a
            // cheap no-op, which is representative of what might happen if the
            // motion planner gets ahead of the stepper controller.
            let _ = producer.enqueue(black_box(cmd));
        })
    });
}

criterion_group!(benches, benchmark_enqueue);
criterion_main!(benches);
