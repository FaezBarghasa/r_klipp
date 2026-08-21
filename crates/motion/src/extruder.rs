//! Generic Extruder Axis with Volumetric Flow Scaling and Pressure Advance.
//!
//! Applies linear advance compensation: $E_{\text{comp}} = E + k \cdot v_{\text{extruder}}$
//! to eliminate corner over-extrusion and seam blobbing in 3D printing and paste dispensing.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenericExtruderAxis {
    pub steps_per_mm: f64,
    pub volumetric_multiplier: f64,
    pub pressure_advance_k: f64,
    pub smooth_time_s: f64,
    pub current_position_mm: f64,
}

impl GenericExtruderAxis {
    pub fn new(steps_per_mm: f64, pressure_advance_k: f64) -> Self {
        Self {
            steps_per_mm: steps_per_mm.max(1.0),
            volumetric_multiplier: 1.0,
            pressure_advance_k: pressure_advance_k.max(0.0),
            smooth_time_s: 0.04, // Default 40ms smoothing
            current_position_mm: 0.0,
        }
    }

    /// Computes advance-compensated extrusion distance given nominal distance and velocity.
    pub fn compute_compensated_advance(&self, nominal_delta_e_mm: f64, current_velocity_mms: f64) -> f64 {
        let scaled_nominal = nominal_delta_e_mm * self.volumetric_multiplier;
        let advance_offset = self.pressure_advance_k * current_velocity_mms;
        scaled_nominal + advance_offset
    }

    /// Converts compensated distance in mm to discrete motor steps.
    pub fn mm_to_steps(&self, distance_mm: f64) -> i64 {
        (distance_mm * self.steps_per_mm).round() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extruder_pressure_advance_compensation() {
        let extruder = GenericExtruderAxis::new(830.0, 0.04);
        let nominal_e = 1.0; // 1mm extrusion
        let velocity = 100.0; // 100mm/s print speed

        let comp = extruder.compute_compensated_advance(nominal_e, velocity);
        // Advance offset = 0.04 * 100 = 4.0mm
        // Total compensated = 1.0 + 4.0 = 5.0mm
        assert_eq!(comp, 5.0);

        let steps = extruder.mm_to_steps(comp);
        assert_eq!(steps, 4150);
    }
}
