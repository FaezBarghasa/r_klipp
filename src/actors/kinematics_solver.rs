
// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::kinematics::math::se3::Transform;
use crate::kinematics::poe_fk::PoeKinematics;
use crate::kinematics::KinematicsError;
use crate::math::Vector;
use heapless::spsc::{Consumer, Producer, Queue};

pub struct KinematicsSolver<const N: usize> {
    kinematics: PoeKinematics<N>,
    request_queue: Consumer<'static, SolverRequest<N>>,
    response_queue: Producer<'static, SolverResponse<N>>,
}

pub struct SolverRequest<const N: usize> {
    pub target_transform: Transform,
    pub initial_joint_angles: [f64; N],
    pub secondary_objective: Option<Vector<N>>,
}

pub type SolverResponse<const N: usize> = Result<[f64; N], KinematicsError>;

impl<const N: usize> KinematicsSolver<N> {
    pub fn new(
        kinematics: PoeKinematics<N>,
        request_queue: Consumer<'static, SolverRequest<N>>,
        response_queue: Producer<'static, SolverResponse<N>>,
    ) -> Self {
        Self {
            kinematics,
            request_queue,
            response_queue,
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(request) = self.request_queue.dequeue() {
                let lambda = self
                    .kinematics
                    .adaptive_damping(&request.initial_joint_angles, 0.1, 0.01)
                    .unwrap_or(0.1);

                let result = self.kinematics.inverse_dls(
                    &request.target_transform,
                    &request.initial_joint_angles,
                    lambda,
                );

                let final_result = match (result, request.secondary_objective) {
                    (Ok(mut angles), Some(objective)) => {
                        let q_dot_null = self
                            .kinematics
                            .nullspace_joint_velocities(&angles, lambda, &objective)
                            .unwrap_or_else(|_| Vector::<N>::zero());

                        for i in 0..N {
                            angles[i] += q_dot_null[i] * 0.1; // Apply a small step
                        }
                        Ok(angles)
                    }
                    (Ok(angles), None) => Ok(angles),
                    (Err(_), _) => Err(KinematicsError::Unreachable),
                };

                if self.response_queue.enqueue(final_result).is_err() {
                    // Handle queue full error
                }
            }
            // Yield to the scheduler
            // In a real async runtime, this would be something like `tokio::task::yield_now().await;`
            // For now, we'll just continue the loop.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::math::se3::Twist;
    use crate::math::{Matrix4, Vector3};
    use heapless::spsc::Queue;

    fn get_test_kinematics() -> PoeKinematics<6> {
        let screw_axes = [
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::ZERO },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::ZERO },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.425) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.817) },
            Twist { angular: Vector3::new(0.0, 0.0, -1.0), linear: Vector3::ZERO },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -1.014) },
        ];
        let M = Transform(Matrix4::from_translation(&Vector3::new(0.0, -0.109, 1.014)));
        PoeKinematics { screw_axes, M }
    }

    #[test]
    fn test_solver_actor() {
        static mut Q_REQUEST: Queue<SolverRequest<6>, 1> = Queue::new();
        static mut Q_RESPONSE: Queue<SolverResponse<6>, 1> = Queue::new();

        let (p_req, c_req) = unsafe { Q_REQUEST.split() };
        let (p_resp, mut c_resp) = unsafe { Q_RESPONSE.split() };

        let kinematics = get_test_kinematics();
        let mut solver = KinematicsSolver::new(kinematics, c_req, p_resp);

        let target_angles = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let target_transform = solver.kinematics.forward(&target_angles).unwrap();
        let initial_joint_angles = [0.0; 6];

        let request = SolverRequest {
            target_transform,
            initial_joint_angles,
            secondary_objective: None,
        };
        p_req.enqueue(request).unwrap();

        // In a real scenario, we'd spawn the solver.run() task.
        // Here, we'll just call a single step of the loop.
        if let Some(req) = solver.request_queue.dequeue() {
            let lambda = solver.kinematics.adaptive_damping(&req.initial_joint_angles, 0.1, 0.01).unwrap();
            let result = solver.kinematics.inverse_dls(&req.target_transform, &req.initial_joint_angles, lambda);
            solver.response_queue.enqueue(result.map_err(|_| KinematicsError::Unreachable)).unwrap();
        }

        let response = c_resp.dequeue().unwrap();
        assert!(response.is_ok());
    }
}
