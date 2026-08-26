use heapless::Deque;
use kinematics::ph_blending::PhCornerBlender;

#[derive(Debug, Clone, Copy)]
enum Motion {
    Linear([f32; 3]),
    Rapid([f32; 3]),
}

pub struct PlannerActor {
    lookahead_buffer: Deque<Motion, 32>,
    blender: PhCornerBlender,
    // stepper_queue: Queue<Step, 1024>,
}

impl PlannerActor {
    pub fn new(blending_tolerance: f64) -> Self {
        Self {
            lookahead_buffer: Deque::new(),
            blender: PhCornerBlender::new(blending_tolerance),
            // stepper_queue: Queue::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            // 1. Receive motion commands from the message bus and push to lookahead_buffer
            // For now, we'll simulate this.
            if self.lookahead_buffer.len() < 2 {
                let _ = self.lookahead_buffer.push_back(Motion::Linear([10.0, 0.0, 0.0]));
                let _ = self.lookahead_buffer.push_back(Motion::Linear([10.0, 10.0, 0.0]));
            }


            if self.lookahead_buffer.len() >= 2 {
                let first = self.lookahead_buffer.pop_front().unwrap();
                let second = self.lookahead_buffer.front().unwrap();

                if let (Motion::Linear(p1), Motion::Linear(p2)) = (&first, second) {
                    // A real implementation would need the start point of the first move.
                    let p0 = [0.0f64; 3];
                    let p1_f64 = [p1[0] as f64, p1[1] as f64, p1[2] as f64];
                    let p2_f64 = [p2[0] as f64, p2[1] as f64, p2[2] as f64];
                    let _blended_curve: heapless::Vec<[f64; 3], 16> = self.blender.blend_corner(p0, p1_f64, p2_f64);
                    // Then push the blended curve to the stepper queue
                } else {
                    // Handle non-linear moves or rapids
                }
            }

            embassy_time::Timer::after(embassy_time::Duration::from_millis(50)).await;
        }
    }
}
