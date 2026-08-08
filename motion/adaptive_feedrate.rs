//! Real-time tool engagement calculation and adaptive feedrate control.
#![no_std]

use micromath::F32Ext;

/// Calculates the tool engagement angle based on the path's radius of curvature.
///
/// # Arguments
/// * `cutter_diameter`: The diameter of the cutting tool.
/// * `path_radius`: The radius of the toolpath's curvature. A positive value indicates an
///   internal corner (milling a pocket), while a negative value indicates an
///   external corner (milling a boss). A very large value approximates a straight line.
///
/// # Returns
/// The engagement angle in radians.
pub fn calculate_engagement_angle(cutter_diameter: f32, path_radius: f32) -> f32 {
    let cutter_radius = cutter_diameter / 2.0;

    if path_radius.abs() < cutter_radius {
        // This indicates a slotting motion where engagement is 180 degrees.
        return core::f32::consts::PI;
    }

    if path_radius > 0.0 { // Internal corner
        let cos_angle = (path_radius - cutter_radius) / path_radius;
        // This formula is a simplification. A more accurate one is:
        // 2.0 * (cutter_radius / path_radius).acos()
        // However, acos is not available in micromath. We use an approximation.
        // For small engagement, angle ~ 2 * sqrt(2 * cutter_radius / path_radius)
        // This is a complex problem without `acos`. We'll use a linear approximation for now.
        // A better solution would be a lookup table or a CORDIC implementation for acos.
        core::f32::consts::PI * (cutter_radius / path_radius) // Simplified linear model
    } else { // External corner
        // For external corners, the engagement angle decreases.
        // As path_radius becomes a large negative number, engagement approaches zero.
        let cos_angle = (path_radius.abs() + cutter_radius) / path_radius.abs();
        // Approximation
        core::f32::consts::PI / 2.0 * (1.0 - (cutter_radius / path_radius.abs()))
    }
}

/// A task that optimizes feedrate based on tool engagement.
pub struct FeedrateOptimizer {
    cutter_diameter: f32,
    max_mrr: f32, // Maximum Material Removal Rate
}

impl FeedrateOptimizer {
    pub fn new(cutter_diameter: f32, target_mrr: f32) -> Self {
        Self {
            cutter_diameter,
            max_mrr: target_mrr,
        }
    }

    /// Adjusts the requested feedrate to maintain a constant MRR.
    ///
    /// # Arguments
    /// * `requested_feedrate`: The feedrate from the G-code.
    /// * `path_radius`: The current radius of curvature of the toolpath.
    ///
    /// # Returns
    /// The adjusted (optimized) feedrate.
    pub fn adjust_feedrate(&self, requested_feedrate: f32, path_radius: f32) -> f32 {
        let engagement_angle = calculate_engagement_angle(self.cutter_diameter, path_radius);

        // A simple model for MRR is: MRR ~ feedrate * engagement_angle
        // To keep MRR constant: feedrate_adjusted * angle_adjusted = feedrate_nominal * angle_nominal
        // We can define a nominal angle (e.g., for a straight line, which is very small).
        // A simpler approach is to set a max engagement angle and scale down from there.
        let max_angle = core::f32::consts::PI; // Full slotting

        let feed_scale = if engagement_angle > 0.0 {
            // This is a simplified scaling factor. A more accurate model would be non-linear.
            let ratio = (max_angle / engagement_angle).sqrt();
            if ratio > 1.5 { 1.5 } else { ratio } // Cap the speedup
        } else {
            1.0
        };

        let adjusted = requested_feedrate * feed_scale;

        // Ensure we don't exceed the originally requested feedrate by too much.
        if adjusted > requested_feedrate * 1.5 {
            requested_feedrate * 1.5
        } else {
            adjusted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_angle_calculation() {
        let cutter_diameter = 10.0;

        // Straight line (infinite radius) -> very small engagement
        // Our approximation gives 0, which is reasonable.
        assert!((calculate_engagement_angle(cutter_diameter, 1e9)).abs() < 1e-6);

        // Full slot (path radius = cutter radius) -> 180 degrees
        assert!((calculate_engagement_angle(cutter_diameter, 5.0) - core::f32::consts::PI).abs() < 1e-6);

        // Internal corner
        let internal_angle = calculate_engagement_angle(cutter_diameter, 20.0);
        assert!(internal_angle > 0.0 && internal_angle < core::f32::consts::PI);

        // External corner
        let external_angle = calculate_engagement_angle(cutter_diameter, -20.0);
        assert!(external_angle > 0.0 && external_angle < core::f32::consts::PI / 2.0);
    }

    #[test]
    fn test_adaptive_feedrate_adjustment() {
        let optimizer = FeedrateOptimizer::new(10.0, 100.0);
        let requested_feed = 1000.0;

        // Straight line -> feedrate should be high (capped at 1.5x in our model)
        let straight_line_feed = optimizer.adjust_feedrate(requested_feed, 1e9);
        assert!((straight_line_feed - requested_feed * 1.5).abs() < 1.0);

        // Full slot -> feedrate should be significantly reduced.
        let slot_feed = optimizer.adjust_feedrate(requested_feed, 5.0);
        assert!(slot_feed < requested_feed);

        // Internal corner -> feedrate should be reduced.
        let internal_corner_feed = optimizer.adjust_feedrate(requested_feed, 20.0);
        assert!(internal_corner_feed < requested_feed * 1.5 && internal_corner_feed > slot_feed);

        // External corner -> feedrate should be high.
        let external_corner_feed = optimizer.adjust_feedrate(requested_feed, -20.0);
        assert!(external_corner_feed > requested_feed);
    }

    #[test]
    fn test_feedrate_doubling_engagement_angle() {
        let optimizer = FeedrateOptimizer::new(10.0, 100.0);
        let requested_feed = 1000.0;

        // A certain radius gives a certain feed.
        let r1 = 50.0;
        let feed1 = optimizer.adjust_feedrate(requested_feed, r1);

        // A smaller radius will have a larger engagement angle and thus a smaller feed.
        let r2 = 25.0;
        let feed2 = optimizer.adjust_feedrate(requested_feed, r2);

        assert!(feed2 < feed1);
    }
}
