use r_klipp_api::{LinkHealth, HostToMcu, Waypoint, FixedPoint};
use heapless::Vec;

pub enum MathTier {
    Tier1, // Predictive
    Tier2, // Basic
}

pub struct AdaptiveMathEngine {
    current_tier: MathTier,
}

impl AdaptiveMathEngine {
    pub fn new() -> Self {
        Self {
            current_tier: MathTier::Tier1,
        }
    }

    pub fn update_tier(&mut self, link_health: &LinkHealth) {
        match self.current_tier {
            MathTier::Tier1 => {
                if link_health.rtt_us > 5000 || link_health.buffer_fill_percent < 30 {
                    self.current_tier = MathTier::Tier2;
                }
            }
            MathTier::Tier2 => {
                if link_health.rtt_us < 2000 && link_health.buffer_fill_percent > 50 {
                    self.current_tier = MathTier::Tier1;
                }
            }
        }
    }

    pub fn generate_trajectory(&self, waypoints: &Vec<Waypoint, 32>) -> HostToMcu {
        match self.current_tier {
            MathTier::Tier1 => HostToMcu::PredictiveTrajectory {
                nurbs_points: Vec::new(), // Placeholder for NURBS points
                mpcc_feedforward: [0.0; 3],
            },
            MathTier::Tier2 => HostToMcu::BasicTrajectory {
                waypoints: waypoints.clone(),
                max_jerk: 10.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_tier_switching() {
        let mut engine = AdaptiveMathEngine::new();

        // Pristine -> Degraded
        let bad_link = LinkHealth { rtt_us: 6000, buffer_fill_percent: 25, dropped_packets: 1 };
        engine.update_tier(&bad_link);
        assert!(matches!(engine.current_tier, MathTier::Tier2));

        // Degraded -> Still Degraded (hysteresis)
        let slightly_better_link = LinkHealth { rtt_us: 4000, buffer_fill_percent: 40, dropped_packets: 1 };
        engine.update_tier(&slightly_better_link);
        assert!(matches!(engine.current_tier, MathTier::Tier2));

        // Degraded -> Pristine
        let good_link = LinkHealth { rtt_us: 1500, buffer_fill_percent: 60, dropped_packets: 0 };
        engine.update_tier(&good_link);
        assert!(matches!(engine.current_tier, MathTier::Tier1));
    }
}
