use nalgebra::{Matrix4, Vector3, Vector6};

// Product of Exponentials (PoE) for Forward Kinematics
pub fn forward_kinematics(twists: &[Vector6<f32>], a: &[f32]) -> Matrix4<f32> {
    let mut transform = Matrix4::identity();
    for (i, twist) in twists.iter().enumerate() {
        transform *= twist_to_transform(twist, a[i]);
    }
    transform
}

// Convert a twist to a homogeneous transformation matrix
fn twist_to_transform(twist: &Vector6<f32>, angle: f32) -> Matrix4<f32> {
    let v = twist.fixed_rows::<3>(0);
    let w = twist.fixed_rows::<3>(3);

    if w.norm_squared() < 1e-9 { // Prismatic joint
        let mut mat = Matrix4::identity();
        mat.fixed_slice_mut::<3, 1>(0, 3).copy_from(&(v * angle));
        return mat;
    }

    // Revolute joint
    let w_hat = w.cross_matrix();
    let w_hat_sq = w_hat * w_hat;
    let identity = Matrix4::identity();
    let r = Matrix4::identity() + w_hat * angle.sin() + w_hat_sq * (1.0 - angle.cos());

    let g = (Matrix4::identity() - r) * w_hat + w * w.transpose() * angle;
    let p = g * v;

    let mut transform = r;
    transform.fixed_slice_mut::<3, 1>(0, 3).copy_from(&p);
    transform
}

// Placeholder for analytical Jacobian
pub fn analytical_jacobian(twists: &[Vector6<f32>], a: &[f32]) -> Matrix4<f32> {
    Matrix4::identity()
}
