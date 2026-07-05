
use nalgebra::Vector3;
use r_klipp_api::{HostToMcu, TimeParams};

pub fn generate_coefficients(waypoints: &[Vector3<f32>]) -> Vec<HostToMcu> {
    let mut coefficients = Vec::new();

    for window in waypoints.windows(2) {
        let start = window[0];
        let end = window[1];

        // This is a simplified linear interpolation. A real implementation would involve
        // Pythagorean-Hodograph (PH) Bézier curves for corner blending.
        let mut points = heapless::Vec::new();
        points.push((start.x * 1000.0) as i32).unwrap();
        points.push((start.y * 1000.0) as i32).unwrap();
        points.push((start.z * 1000.0) as i32).unwrap();
        points.push((end.x * 1000.0) as i32).unwrap();
        points.push((end.y * 1000.0) as i32).unwrap();
        points.push((end.z * 1000.0) as i32).unwrap();

        coefficients.push(HostToMcu::TrajectoryCoefficients {
            points,
            time_params: TimeParams { duration: 100 }, // Simplified
        });
    }

    coefficients
}
