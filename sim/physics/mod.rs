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

use crate::math::{Point, Vector3};

pub struct RigidBody {
    pub mass: f64,
    pub position: Point,
    pub velocity: Vector3,
    pub force: Vector3,
}

impl RigidBody {
    pub fn new(mass: f64, position: Point) -> Self {
        Self {
            mass,
            position,
            velocity: Vector3::zero(),
            force: Vector3::zero(),
        }
    }

    pub fn step(&mut self, dt: f64) {
        let acceleration = self.force / self.mass;
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;
        self.force = Vector3::zero();
    }
}

pub struct PhysicsEngine {
    pub bodies: Vec<RigidBody>,
    // Add constraints like joints, motors, etc.
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self { bodies: Vec::new() }
    }

    pub fn step(&mut self, dt: f64) {
        for body in &mut self.bodies {
            body.step(dt);
        }
    }
}