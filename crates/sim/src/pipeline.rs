//! End-to-end G-Code Ingestion and Motion Execution Simulation Pipeline.

use crate::hal::MockHal;
use parser::stream::GcodeLineParser;
use parser::ast::AstNode;
use motion::lookahead::LookaheadQueue;
use motion::trapezoidal::TrapezoidalProfile;
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct TrajectoryPoint {
    pub time_s: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub velocity: f64,
    pub accel: f64,
    pub jerk: f64,
}

#[derive(Debug, Clone, Default)]
pub struct SimulationReport {
    pub moves_planned: usize,
    pub total_steps: usize,
    pub total_simulated_time_s: f64,
    pub trajectory: Vec<TrajectoryPoint>,
}

pub struct SimulationPipeline {
    pub hal: MockHal,
    pub parser: GcodeLineParser,
    pub lookahead: LookaheadQueue<64>,
    pub current_pos: [f64; 3],
    pub feedrate: f64,
    pub max_accel: f64,
    pub steps_per_mm: f64,
    pub timer_freq_hz: f64,
}

impl SimulationPipeline {
    pub fn new(junction_deviation: f64, steps_per_mm: f64, timer_freq_hz: f64) -> Self {
        Self {
            hal: MockHal::new(),
            parser: GcodeLineParser::new(),
            lookahead: LookaheadQueue::new(junction_deviation),
            current_pos: [0.0, 0.0, 0.0],
            feedrate: 60.0, // Default 60mm/s
            max_accel: 3000.0, // Default 3000mm/s^2
            steps_per_mm,
            timer_freq_hz,
        }
    }

    /// Ingests a multi-line G-Code string and executes the motion through the simulator.
    pub fn run_gcode(&mut self, gcode_content: &str) -> Result<SimulationReport> {
        let mut report = SimulationReport::default();
        let mut simulated_time = 0.0;

        for line in gcode_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            if let Ok(Some(node)) = self.parser.parse_line(line) {
                match node {
                    AstNode::RapidMove { x, y, z, feedrate, .. } | AstNode::LinearMove { x, y, z, feedrate, .. } => {
                        if let Some(f) = feedrate {
                            self.feedrate = (f / 60.0) as f64; // G-Code F is mm/min, convert to mm/s
                        }

                        let target = [
                            x.map(|v| v as f64).unwrap_or(self.current_pos[0]),
                            y.map(|v| v as f64).unwrap_or(self.current_pos[1]),
                            z.map(|v| v as f64).unwrap_or(self.current_pos[2]),
                        ];

                        let _ = self.lookahead.push_move(self.current_pos, target, self.feedrate, self.max_accel);
                        self.current_pos = target;
                    }
                    AstNode::Home { x, y, z } => {
                        if x { self.current_pos[0] = 0.0; }
                        if y { self.current_pos[1] = 0.0; }
                        if z { self.current_pos[2] = 0.0; }
                    }
                    _ => {}
                }
            }
        }

        // Drain lookahead queue and execute each move in MockHal
        let mut current_toolhead = [0.0, 0.0, 0.0];

        while let Some(profile) = self.lookahead.pop_move() {
            report.moves_planned += 1;
            let move_duration = profile.total_time;

            // Generate step intervals into MockHal
            let step_intervals = profile.compute_step_intervals(self.steps_per_mm, self.timer_freq_hz);
            for interval in &step_intervals {
                self.hal.advance_ticks(*interval as u64);
                self.hal.step_pulse(0, true);
            }

            // Sample trajectory curves for visualization export (20 samples per move)
            let num_samples = 20;
            for s in 0..=num_samples {
                let rel_t = (s as f64 / num_samples as f64) * move_duration;
                let t_abs = simulated_time + rel_t;
                let s_dist = profile.position_at(rel_t);
                let v = profile.velocity_at(rel_t);
                let a = profile.acceleration_at(rel_t);

                // Direction scaling
                let dir_fraction = if profile.distance > 1e-9 { s_dist / profile.distance } else { 0.0 };
                let pt_x = current_toolhead[0] + dir_fraction * (self.current_pos[0] - current_toolhead[0]);
                let pt_y = current_toolhead[1] + dir_fraction * (self.current_pos[1] - current_toolhead[1]);
                let pt_z = current_toolhead[2] + dir_fraction * (self.current_pos[2] - current_toolhead[2]);

                report.trajectory.push(TrajectoryPoint {
                    time_s: t_abs,
                    x: pt_x,
                    y: pt_y,
                    z: pt_z,
                    velocity: v,
                    accel: a,
                    jerk: 0.0,
                });
            }

            simulated_time += move_duration;
            current_toolhead = self.current_pos;
        }

        report.total_steps = self.hal.total_step_count();
        report.total_simulated_time_s = simulated_time;

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_pipeline_execution() {
        let mut sim = SimulationPipeline::new(0.05, 80.0, 1_000_000.0);
        let gcode = r#"
            G28 ; Home
            G1 X50 Y25 F3000
            G1 X100 Y50 F6000
            G1 X100 Y100 F6000
        "#;

        let report = sim.run_gcode(gcode).expect("pipeline failed");
        assert!(report.moves_planned >= 2);
        assert!(report.total_steps > 0);
        assert!(report.total_simulated_time_s > 0.0);
        assert!(!report.trajectory.is_empty());
    }
}
