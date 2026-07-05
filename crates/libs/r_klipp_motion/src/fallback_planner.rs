use r_klipp_api::Waypoint;
use heapless::Vec;
use micromath::F32Ext;

pub struct FallbackPlanner {}

impl FallbackPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn plan_basic_trajectory(&self, waypoints: &Vec<Waypoint, 32>, max_jerk: f32) -> Vec<[u32; 3], 32> {
        let mut planned_steps = Vec::new();

        if waypoints.len() < 2 {
            return planned_steps;
        }

        for i in 0..waypoints.len() - 1 {
            let start = waypoints[i];
            let end = waypoints[i + 1];

            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let dz = end.z - start.z;

            let distance = (dx*dx + dy*dy + dz*dz).sqrt();
            let time = distance / start.speed;

            // This is a simplified trapezoidal velocity profile.
            // A real implementation would be more complex.
            let accel_time = start.speed / max_jerk;
            let accel_dist = 0.5 * max_jerk * accel_time * accel_time;

            if distance < 2.0 * accel_dist {
                // Triangle profile
                let t = (distance / max_jerk).sqrt();
                let steps = (distance * 100.0) as u32; // Assuming 100 steps/mm
                planned_steps.push([steps, (t * 1000.0) as u32, (t * 1000.0) as u32]).unwrap();
            } else {
                // Trapezoid profile
                let cruise_dist = distance - 2.0 * accel_dist;
                let cruise_time = cruise_dist / start.speed;
                let total_time = 2.0 * accel_time + cruise_time;
                let steps = (distance * 100.0) as u32;
                planned_steps.push([steps, (total_time * 1000.0) as u32, (accel_time * 1000.0) as u32]).unwrap();
            }
        }

        if waypoints.len() >= 3 {
             self.apply_junction_blending(&mut planned_steps, waypoints);
        }


        planned_steps
    }

    fn apply_junction_blending(&self, planned_steps: &mut Vec<[u32; 3], 32>, waypoints: &Vec<Waypoint, 32>) {
        for i in 0..waypoints.len() - 2 {
            let p1 = waypoints[i];
            let p2 = waypoints[i+1];
            let p3 = waypoints[i+2];

            let v1 = [p2.x - p1.x, p2.y - p1.y, p2.z - p1.z];
            let v2 = [p3.x - p2.x, p3.y - p2.y, p3.z - p2.z];

            let v1_mag = (v1[0]*v1[0] + v1[1]*v1[1] + v1[2]*v1[2]).sqrt();
            let v2_mag = (v2[0]*v2[0] + v2[1]*v2[1] + v2[2]*v2[2]).sqrt();

            if v1_mag == 0.0 || v2_mag == 0.0 {
                continue;
            }

            let dot = v1[0]*v2[0] + v1[1]*v2[1] + v1[2]*v2[2];
            let cos_theta = dot / (v1_mag * v2_mag);
            let angle = cos_theta.acos(); // in radians

            // Reduce speed at sharp corners
            if angle > 0.1 { // e.g. > 5.7 degrees
                let junction_speed_factor = (1.0 - (angle / 3.14159)).max(0.1);
                let original_time = planned_steps[i][1] as f32 / 1000.0;
                planned_steps[i][1] = (original_time / junction_speed_factor) as u32;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use r_klipp_api::Waypoint;
    use heapless::Vec;

    #[test]
    fn test_junction_blending() {
        let planner = FallbackPlanner::new();
        let mut waypoints = Vec::new();
        waypoints.push(Waypoint { x: 0.0, y: 0.0, z: 0.0, speed: 100.0 }).unwrap();
        waypoints.push(Waypoint { x: 10.0, y: 0.0, z: 0.0, speed: 100.0 }).unwrap();
        waypoints.push(Waypoint { x: 10.0, y: 10.0, z: 0.0, speed: 100.0 }).unwrap();

        let mut planned_steps = planner.plan_basic_trajectory(&waypoints, 10.0);

        // The first segment's time should be increased due to the 90-degree corner
        assert!(planned_steps[0][1] > 100); // 10mm / 100mm/s = 0.1s = 100ms
    }
}
