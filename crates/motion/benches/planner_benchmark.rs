use criterion::{black_box, criterion_group, criterion_main, Criterion};
use motion::{
    kinematics::CartesianKinematics,
    planner::{Move, Planner, TrapezoidalPlanner},
    CartesianPoint,
};

fn benchmark_planner(c: &mut Criterion) {
    let kinematics = CartesianKinematics {
        steps_per_mm_x: 80.0,
        steps_per_mm_y: 80.0,
        steps_per_mm_z: 400.0,
    };
    let planner = TrapezoidalPlanner::default();
    let mv = Move {
        from: CartesianPoint { x: 0.0, y: 0.0, z: 0.0 },
        to: CartesianPoint { x: 150.0, y: 50.0, z: 20.0 },
        max_velocity: 100.0,
        max_acceleration: 1500.0,
    };

    c.bench_function("plan_long_move", |b| {
        b.iter(|| {
            // Because the planner returns an iterator, we must consume it
            // to measure the actual planning work.
            #[cfg(feature = "alloc")]
            {
                let _steps = planner.plan_move_to_vec(black_box(&kinematics), black_box(mv));
            }
            #[cfg(not(feature = "alloc"))]
            {
                // If no allocator, just drain the iterator
                planner.plan_move(black_box(&kinematics), black_box(mv)).for_each(|_| {});
            }
        })
    });
}

criterion_group!(benches, benchmark_planner);
criterion_main!(benches);
