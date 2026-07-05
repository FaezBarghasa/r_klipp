use r_klipp_api::{HostToMcu, LinkHealth, Waypoint, FixedPoint};
use heapless::Vec;

pub struct AdaptiveMathEngine {
    tier: Tier,
    last_tier_change_time: u64,
}

#[derive(PartialEq, Debug)]
pub enum Tier {
    Tier1, // Predictive
    Tier2, // Basic
}

impl AdaptiveMathEngine {
    pub fn new() -> Self {
        Self {
            tier: Tier::Tier1,
            last_tier_change_time: 0,
        }
    }

    pub fn update_link_health(&mut self, health: &LinkHealth, current_time: u64) {
        let hysteresis_passed = current_time - self.last_tier_change_time > 5000; // 5 seconds

        if hysteresis_passed {
            if health.rtt_us > 5000 || health.buffer_fill_percent < 30 {
                if self.tier == Tier::Tier1 {
                    self.tier = Tier::Tier2;
                    self.last_tier_change_time = current_time;
                }
            } else if health.rtt_us < 2000 && health.buffer_fill_percent > 50 {
                if self.tier == Tier::Tier2 {
                    self.tier = Tier::Tier1;
                    self.last_tier_change_time = current_time;
                }
            }
        }
    }

    pub fn generate_trajectory(&self, waypoints: &Vec<Waypoint, 32>) -> HostToMcu {
        match self.tier {
            Tier::Tier1 => {
                // In a real implementation, this would call the NURBS and MPCC generators
                let mut nurbs_points = Vec::new();
                nurbs_points.push(FixedPoint { x: 1, y: 2, z: 3 }).unwrap();
                HostToMcu::PredictiveTrajectory {
                    nurbs_points,
                    mpcc_feedforward: [1.0, 2.0, 3.0],
                }
            }
            Tier::Tier2 => {
                HostToMcu::BasicTrajectory {
                    waypoints: waypoints.clone(),
                    max_jerk: 10.0,
                }
            }
        }
    }

    pub fn get_tier(&self) -> &Tier {
        &self.tier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r_klipp_api::LinkHealth;

    #[test]
    fn test_adaptive_tier_switching() {
        let mut engine = AdaptiveMathEngine::new();
        let mut current_time = 0;

        // Start in Tier 1
        assert_eq!(*engine.get_tier(), Tier::Tier1);

        // Degraded link
        let degraded_health = LinkHealth { rtt_us: 6000, buffer_fill_percent: 25, dropped_packets: 0 };
        engine.update_link_health(&degraded_health, current_time);
        assert_eq!(*engine.get_tier(), Tier::Tier2);

        // Link recovers, but hysteresis prevents immediate switch back
        current_time += 1000;
        let recovered_health = LinkHealth { rtt_us: 1500, buffer_fill_percent: 75, dropped_packets: 0 };
        engine.update_link_health(&recovered_health, current_time);
        assert_eq!(*engine.get_tier(), Tier::Tier2);

        // Hysteresis time passes, link is still good, switch back to Tier 1
        current_time += 5000;
        engine.update_link_health(&recovered_health, current_time);
        assert_eq!(*engine.get_tier(), Tier::Tier1);
    }
}
