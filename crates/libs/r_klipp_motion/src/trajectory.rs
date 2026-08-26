
use nalgebra::Vector3;
use r_klipp_api::{HostCommand, HostToMcu, Waypoint};

pub fn generate_coefficients(waypoints: &[Vector3<f32>]) -> Vec<HostToMcu> {
    let mut coefficients = Vec::new();

    for window in waypoints.windows(2) {
        let start = window[0];
        let end = window[1];

        let mut pts = heapless::Vec::new();
        let _ = pts.push(Waypoint {
            position: [start.x, start.y, start.z],
            feedrate: 100.0,
        });
        let _ = pts.push(Waypoint {
            position: [end.x, end.y, end.z],
            feedrate: 100.0,
        });

        coefficients.push(HostToMcu::new(
            0,
            HostCommand::BasicTrajectory {
                waypoints: pts,
                max_jerk: 10.0,
            },
        ));
    }

    coefficients
}

