//! 5-axis tool orientation smoothing and lead-in/lead-out generation.
#![no_std]

use micromath::F32Ext;
use heapless::Vec;

/// Represents a point in a 5-axis toolpath, including position and orientation.
/// Orientation is represented as a quaternion [w, x, y, z].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolPoint {
    pub pos: [f32; 3],
    pub orient: [f32; 4],
}

/// Smooths the tool orientation part of a 5-axis toolpath using SLERP.
///
/// # Arguments
/// * `points`: A slice of `ToolPoint`s representing the original toolpath.
/// * `smoothed_points`: A mutable `Vec` to store the resulting smoothed path.
/// * `num_steps`: The number of interpolation steps between each original point.
pub fn smooth_tool_orientation<const N: usize>(
    points: &[ToolPoint],
    smoothed_points: &mut Vec<ToolPoint, N>,
    num_steps: usize,
) {
    smoothed_points.clear();
    if points.is_empty() {
        return;
    }

    smoothed_points.push(points[0]).unwrap();

    for i in 0..(points.len() - 1) {
        let p0 = points[i];
        let p1 = points[i+1];

        let q0 = p0.orient;
        let mut q1 = p1.orient;

        // Handle dot product > 1 due to floating point inaccuracies
        let mut dot = q0[0] * q1[0] + q0[1] * q1[1] + q0[2] * q1[2] + q0[3] * q1[3];
        if dot < 0.0 {
            // Take the shorter path on the 4D sphere
            q1 = [-q1[0], -q1[1], -q1[2], -q1[3]];
            dot = -dot;
        }

        let dot_threshold = 0.9995;
        if dot > dot_threshold {
            // Linear interpolation for very small angles
            for j in 1..=num_steps {
                let t = j as f32 / num_steps as f32;
                let new_orient = [
                    q0[0] * (1.0 - t) + q1[0] * t,
                    q0[1] * (1.0 - t) + q1[1] * t,
                    q0[2] * (1.0 - t) + q1[2] * t,
                    q0[3] * (1.0 - t) + q1[3] * t,
                ];
                // Normalize
                let norm = (new_orient[0]*new_orient[0] + new_orient[1]*new_orient[1] + new_orient[2]*new_orient[2] + new_orient[3]*new_orient[3]).sqrt();
                let new_pos = [
                    p0.pos[0] * (1.0 - t) + p1.pos[0] * t,
                    p0.pos[1] * (1.0 - t) + p1.pos[1] * t,
                    p0.pos[2] * (1.0 - t) + p1.pos[2] * t,
                ];
                smoothed_points.push(ToolPoint { pos: new_pos, orient: [new_orient[0]/norm, new_orient[1]/norm, new_orient[2]/norm, new_orient[3]/norm] }).unwrap();
            }
        } else {
            // SLERP
            let theta_0 = dot.acos();
            let theta = theta_0 / num_steps as f32;
            let sin_theta = theta.sin();
            let sin_theta_0 = theta_0.sin();

            for j in 1..=num_steps {
                 let t = j as f32 / num_steps as f32;
                 let s0 = ((1.0 - t) * theta_0).sin() / sin_theta_0;
                 let s1 = (t * theta_0).sin() / sin_theta_0;
                 let new_orient = [
                    q0[0] * s0 + q1[0] * s1,
                    q0[1] * s0 + q1[1] * s1,
                    q0[2] * s0 + q1[2] * s1,
                    q0[3] * s0 + q1[3] * s1,
                 ];
                 let new_pos = [
                    p0.pos[0] * (1.0 - t) + p1.pos[0] * t,
                    p0.pos[1] * (1.0 - t) + p1.pos[1] * t,
                    p0.pos[2] * (1.0 - t) + p1.pos[2] * t,
                ];
                smoothed_points.push(ToolPoint { pos: new_pos, orient: new_orient }).unwrap();
            }
        }
    }
}

/// Generates a tangential lead-in arc for a cutting move.
///
/// # Arguments
/// * `start_point`: The first point of the cutting path.
/// * `tangent`: The tangent vector of the path at the start point.
/// * `radius`: The radius of the lead-in arc.
/// * `num_points`: The number of points to generate for the arc.
///
/// # Returns
/// A `Vec` of `[x, y, z]` points representing the lead-in arc.
pub fn generate_lead_in_arc<const N: usize>(
    start_point: [f32; 3],
    tangent: [f32; 3],
    radius: f32,
    num_points: usize,
) -> Vec<[f32; 3], N> {
    let mut arc_points = Vec::new();

    // Normalize tangent vector
    let t_norm = (tangent[0]*tangent[0] + tangent[1]*tangent[1] + tangent[2]*tangent[2]).sqrt();
    let t_hat = [tangent[0]/t_norm, tangent[1]/t_norm, tangent[2]/t_norm];

    // Normal vector (in XY plane for simplicity)
    let n_hat = [-t_hat[1], t_hat[0], 0.0];

    let center_x = start_point[0] - n_hat[0] * radius;
    let center_y = start_point[1] - n_hat[1] * radius;

    let start_angle = (start_point[1] - center_y).atan2(start_point[0] - center_x);

    for i in (0..=num_points).rev() {
        let angle = start_angle - (i as f32 / num_points as f32) * (core::f32::consts::PI / 2.0);
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        arc_points.push([x, y, start_point[2]]).unwrap();
    }

    arc_points
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slerp_smoothing() {
        let p1 = ToolPoint { pos: [0.0, 0.0, 0.0], orient: [1.0, 0.0, 0.0, 0.0] }; // No rotation
        let p2 = ToolPoint { pos: [10.0, 0.0, 0.0], orient: [0.0, 1.0, 0.0, 0.0] }; // 180 deg rotation around X

        let mut smoothed = Vec::<ToolPoint, 100>::new();
        smooth_tool_orientation(&[p1, p2], &mut smoothed, 10);

        assert_eq!(smoothed.len(), 11);
        assert_eq!(smoothed[0], p1);
        assert_eq!(*smoothed.last().unwrap(), p2);

        // Check intermediate orientation
        let mid_point = smoothed[5];
        // Halfway through, should be a 90 deg rotation
        let expected_w = (core::f32::consts::PI / 4.0).cos(); // cos(45)
        let expected_x = (core::f32::consts::PI / 4.0).sin(); // sin(45)
        assert!((mid_point.orient[0] - expected_w).abs() < 1e-4);
        assert!((mid_point.orient[1] - expected_x).abs() < 1e-4);
    }

    #[test]
    fn test_lead_in_arc_generation() {
        let start_point = [10.0, 0.0, 0.0];
        let tangent = [0.0, 1.0, 0.0]; // Moving in +Y direction
        let radius = 5.0;

        let arc = generate_lead_in_arc::<20>(start_point, tangent, radius, 10);

        assert_eq!(arc.len(), 11);

        // First point of the arc should be at [5, -5, 0]
        assert!((arc[0][0] - 5.0).abs() < 1e-4);
        assert!((arc[0][1] - -5.0).abs() < 1e-4);

        // Last point of the arc should be the start_point
        assert!((arc.last().unwrap()[0] - start_point[0]).abs() < 1e-4);
        assert!((arc.last().unwrap()[1] - start_point[1]).abs() < 1e-4);
    }
}
