//! Native trochoidal and helical interpolation engine.
#![no_std]

use micromath::F32Ext;

#[derive(Debug, Clone, Copy)]
pub struct TrochoidalMove {
    /// Start point of the linear path component
    pub start: [f32; 3],
    /// End point of the linear path component
    pub end: [f32; 3],
    /// Radius of the circular milling component
    pub radius: f32,
    /// Feedrate along the linear path
    pub linear_feedrate: f32,
    /// Revolutions per minute of the circular motion
    pub rpm: f32,
    /// Total duration of the move
    pub duration: f32,
}

#[derive(Debug)]
pub struct TrochoidalPlanner {
    move_params: TrochoidalMove,
    current_time: f32,
}

impl TrochoidalPlanner {
    pub fn new(move_params: TrochoidalMove) -> Self {
        Self {
            move_params,
            current_time: 0.0,
        }
    }

    /// Generates the next point in the trochoidal path.
    ///
    /// # Arguments
    /// * `dt`: The time step for this generation.
    ///
    /// # Returns
    /// An `Option` containing the next `[x, y, z]` point. Returns `None` if the move is complete.
    pub fn next_point(&mut self, dt: f32) -> Option<[f32; 3]> {
        if self.current_time > self.move_params.duration {
            return None;
        }

        let t = self.current_time / self.move_params.duration;

        // Linear component
        let linear_x = self.move_params.start[0] + t * (self.move_params.end[0] - self.move_params.start[0]);
        let linear_y = self.move_params.start[1] + t * (self.move_params.end[1] - self.move_params.start[1]);
        let linear_z = self.move_params.start[2] + t * (self.move_params.end[2] - self.move_params.start[2]);

        // Circular component (in the XY plane for simplicity)
        let angle = 2.0 * core::f32::consts::PI * self.move_params.rpm / 60.0 * self.current_time;
        let circular_x = self.move_params.radius * angle.cos();
        let circular_y = self.move_params.radius * angle.sin();

        self.current_time += dt;

        Some([
            linear_x + circular_x,
            linear_y + circular_y,
            linear_z,
        ])
    }

    /// Checks if the combined velocity exceeds a given limit.
    /// This is a simplified check. A real implementation would be more rigorous.
    pub fn check_velocity(&self, max_velocity: f32) -> bool {
        // Linear velocity
        let dx = self.move_params.end[0] - self.move_params.start[0];
        let dy = self.move_params.end[1] - self.move_params.start[1];
        let dz = self.move_params.end[2] - self.move_params.start[2];
        let linear_dist = (dx*dx + dy*dy + dz*dz).sqrt();
        let linear_vel = linear_dist / self.move_params.duration;

        // Max circular velocity (tangential)
        let angular_vel = 2.0 * core::f32::consts::PI * self.move_params.rpm / 60.0;
        let circular_vel = self.move_params.radius * angular_vel;

        // This is a worst-case check where velocities add up.
        (linear_vel + circular_vel) <= max_velocity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trochoidal_path_generation() {
        let trochoid_move = TrochoidalMove {
            start: [0.0, 0.0, 0.0],
            end: [100.0, 0.0, 0.0],
            radius: 5.0,
            linear_feedrate: 1000.0,
            rpm: 600.0,
            duration: 6.0, // 100mm at 1000mm/min = 6 seconds
        };

        let mut planner = TrochoidalPlanner::new(trochoid_move);
        let dt = 0.1; // 100ms time step

        let mut points = 0;
        let mut last_point = [0.0; 3];
        while let Some(point) = planner.next_point(dt) {
            points += 1;
            // Check that the point is moving generally in the +X direction
            if points > 1 {
                assert!(point[0] > last_point[0] - 5.0); // Allow for circular motion
            }
            last_point = point;
        }

        assert!(points > 50 && points < 70); // Should be around 60 points
        // Final point should be near the end + circular offset
        assert!((last_point[0] - 100.0).abs() < 6.0);
    }

    #[test]
    fn test_helical_path_generation() {
        // A helix is a trochoid with a Z component change
        let helical_move = TrochoidalMove {
            start: [0.0, 0.0, 0.0],
            end: [0.0, 0.0, -10.0], // Moving down in Z
            radius: 20.0,
            linear_feedrate: 100.0,
            rpm: 60.0, // 1 revolution per second
            duration: 6.0, // 10mm at 100mm/min = 6 seconds
        };

        let mut planner = TrochoidalPlanner::new(helical_move);
        let dt = 1.0;

        let p1 = planner.next_point(dt).unwrap();
        assert!((p1[0] - 20.0).abs() < 1e-4); // cos(0) = 1
        assert!((p1[1] - 0.0).abs() < 1e-4);  // sin(0) = 0
        assert!((p1[2] - -10.0/6.0).abs() < 1e-4);

        let p2 = planner.next_point(dt).unwrap();
        // After 1s, should be at angle 2*pi, so back to cos=1
        assert!((p2[0] - 20.0).abs() < 1e-4);
        assert!((p2[1] - 0.0).abs() < 1e-4);
        assert!((p2[2] - -20.0/6.0).abs() < 1e-4);
    }

    #[test]
    fn test_velocity_check() {
         let move_ok = TrochoidalMove {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            radius: 1.0,
            linear_feedrate: 10.0,
            rpm: 60.0, // 1 rev/sec -> 2*pi rad/sec
            duration: 1.0,
        };
        let planner_ok = TrochoidalPlanner::new(move_ok);
        // Linear vel = 10. Circular vel = 1 * 2*pi ~= 6.28. Total ~= 16.28
        assert!(planner_ok.check_velocity(20.0));
        assert!(!planner_ok.check_velocity(15.0));

        let move_fail = TrochoidalMove {
            start: [0.0, 0.0, 0.0],
            end: [20.0, 0.0, 0.0],
            radius: 2.0,
            linear_feedrate: 20.0,
            rpm: 120.0, // 2 rev/sec -> 4*pi rad/sec
            duration: 1.0,
        };
        let planner_fail = TrochoidalPlanner::new(move_fail);
        // Linear vel = 20. Circular vel = 2 * 4*pi ~= 25.1. Total ~= 45.1
        assert!(!planner_fail.check_velocity(40.0));
    }
}
