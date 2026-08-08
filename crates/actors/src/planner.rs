use comms::Message;
use heapless::spsc::Queue;
use heapless::VecDeque;
use kinematics::ph_blending::PhBlender;

enum Motion {
    Linear([f32; 3]),
    Rapid([f32; 3]),
}

pub struct PlannerActor<const N: usize> {
    lookahead_buffer: VecDeque<Motion, 32>,
    blender: PhBlender<N>,
    // stepper_queue: Queue<Step, 1024>,
}

impl<const N: usize> PlannerActor<N> {
    pub fn new(blending_tolerance: f32) -> Self {
        Self {
            lookahead_buffer: VecDeque::new(),
            blender: PhBlender::new(blending_tolerance),
            // stepper_queue: Queue::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            // 1. Receive motion commands from the message bus and push to lookahead_buffer
            // For now, we'll simulate this.
            if self.lookahead_buffer.len() < 2 {
                self.lookahead_buffer.push_back(Motion::Linear([10.0, 0.0, 0.0])).unwrap();
                self.lookahead_buffer.push_back(Motion::Linear([10.0, 10.0, 0.0])).unwrap();
            }


            if self.lookahead_buffer.len() >= 2 {
                let first = self.lookahead_buffer.pop_front().unwrap();
                let second = self.lookahead_buffer.front().unwrap();

                if let (Motion::Linear(p1), Motion::Linear(p2)) = (&first, second) {
                    // A real implementation would need the start point of the first move.
                    let p0 = [0.0; 3];
                    let blended_curve = self.blender.blend(p0, *p1, *p2);
                    // Then push the blended curve to the stepper queue
                } else {
                    // Handle non-linear moves or rapids
                }
            }

            embassy_time::Timer::after(embassy_time::Duration::from_millis(50)).await;
        }
    }
}
