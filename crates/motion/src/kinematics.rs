//! Cartesian and stepper coordinate mapping.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartesianMapper {
    pub x_steps_per_mm: f64,
    pub y_steps_per_mm: f64,
    pub z_steps_per_mm: f64,
    pub e_steps_per_mm: f64,
}

impl CartesianMapper {
    pub fn new(x_steps_per_mm: f64, y_steps_per_mm: f64, z_steps_per_mm: f64, e_steps_per_mm: f64) -> Self {
        Self {
            x_steps_per_mm,
            y_steps_per_mm,
            z_steps_per_mm,
            e_steps_per_mm,
        }
    }

    /// Converts toolhead coordinate (in mm) to motor step counts.
    pub fn mm_to_steps(&self, pos: [f64; 4]) -> [i64; 4] {
        [
            (pos[0] * self.x_steps_per_mm).round() as i64,
            (pos[1] * self.y_steps_per_mm).round() as i64,
            (pos[2] * self.z_steps_per_mm).round() as i64,
            (pos[3] * self.e_steps_per_mm).round() as i64,
        ]
    }

    /// Converts motor step counts back to toolhead coordinate (in mm).
    pub fn steps_to_mm(&self, steps: [i64; 4]) -> [f64; 4] {
        [
            steps[0] as f64 / self.x_steps_per_mm,
            steps[1] as f64 / self.y_steps_per_mm,
            steps[2] as f64 / self.z_steps_per_mm,
            steps[3] as f64 / self.e_steps_per_mm,
        ]
    }
}