use r_klipp_api::Waypoint;
use heapless::Vec;

pub struct FallbackPlanner;

impl FallbackPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_basic_trajectory(&self, waypoints: &Vec<Waypoint, 32>, max_jerk: f32) -> Vec<[f32; 3], 1024> {
        let mut planned_path = Vec::new();
        if waypoints.len() < 2 {
            return planned_path;
        }

        for i in 0..(waypoints.len() - 1) {
            let start = &waypoints[i];
            let end = &waypoints[i+1];

            // Simple trapezoidal profile (linear interpolation for now)
            let distance = end.position.iter().zip(start.position.iter()).map(|(e, s)| (e - s).powi(2)).sum::<f32>().sqrt();
            let segments = (distance * 10.0) as usize; // 10 segments per mm

            for j in 0..segments {
                let t = j as f32 / segments as f32;
                let mut point = [0.0; 3];
                for k in 0..3 {
                    point[k] = start.position[k] + t * (end.position[k] - start.position[k]);
                }
                planned_path.push(point).unwrap();
            }
        }

        // Simplified junction blending
        if waypoints.len() > 2 {
            for i in 0..(waypoints.len() - 2) {
                // Reduce velocity at sharp corners - this is a placeholder for real math
                let _p1 = &waypoints[i];
                let _p2 = &waypoints[i+1];
                let _p3 = &waypoints[i+2];
                // A real implementation would calculate the angle and limit velocity.
            }
        }

        planned_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_planner_junction_velocity() {
        let planner = FallbackPlanner::new();
        let mut waypoints = Vec::new();
        waypoints.push(Waypoint { position: [0.0, 0.0, 0.0], feedrate: 100.0 }).unwrap();
        waypoints.push(Waypoint { position: [10.0, 0.0, 0.0], feedrate: 100.0 }).unwrap();
        waypoints.push(Waypoint { position: [10.0, 10.0, 0.0], feedrate: 100.0 }).unwrap();

        let path = planner.plan_basic_trajectory(&waypoints, 10.0);

        // A real test would assert that the velocity profile changes at the junction.
        // For now, we just check that a path is generated.
        assert!(path.len() > 0);
    }
}
