//! G-code parser extensions for spline commands.
#![no_std]

use heapless::Vec;
use crate::motion::kinematics::splines::math::ControlPoint;

const MAX_SPLINE_POINTS: usize = 50;

/// Represents a single G-code command after parsing.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Gcode {
    /// Standard linear or rapid move (G00, G01)
    LinearMove { x: f32, y: f32, z: f32 },
    /// Enter NURBS/B-Spline interpolation mode (G05.1 Q1)
    EnterSplineMode,
    /// Exit NURBS/B-Spline interpolation mode (G05.1 Q0)
    ExitSplineMode,
    /// A control point for the current spline definition.
    SplineControlPoint(ControlPoint),
    /// An unknown or unsupported G-code.
    Unknown,
}

/// A simple error type for parsing.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidFormat,
    BufferFull,
    NotInSplineMode,
}

/// Buffers control points for a spline until the block is complete.
pub struct SplineBuffer {
    pub points: Vec<ControlPoint, MAX_SPLINE_POINTS>,
    is_active: bool,
}

impl SplineBuffer {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            is_active: false,
        }
    }

    /// Adds a control point to the buffer.
    pub fn add_point(&mut self, point: ControlPoint) -> Result<(), ParseError> {
        if !self.is_active {
            return Err(ParseError::NotInSplineMode);
        }
        self.points.push(point).map_err(|_| ParseError::BufferFull)
    }

    /// Activates the spline buffer.
    pub fn begin(&mut self) {
        self.points.clear();
        self.is_active = true;
    }

    /// Deactivates the buffer and returns the collected points.
    pub fn end(&mut self) -> Vec<ControlPoint, MAX_SPLINE_POINTS> {
        self.is_active = false;
        core::mem::take(&mut self.points)
    }
}

/// A very basic G-code line parser for demonstration.
/// In a real system, this would be much more robust.
pub fn parse_line(line: &str) -> Result<Gcode, ParseError> {
    let parts: Vec<&str, 10> = line.trim().split_whitespace().collect();
    let command = parts.get(0).ok_or(ParseError::InvalidFormat)?;

    match *command {
        "G1" | "G01" | "G0" | "G00" => {
            let mut gcode = Gcode::LinearMove { x: 0.0, y: 0.0, z: 0.0 };
            // This is a simplified parser. A real one would handle missing axes.
            if let Gcode::LinearMove { ref mut x, ref mut y, ref mut z } = gcode {
                for part in &parts[1..] {
                    if part.starts_with('X') {
                        *x = part[1..].parse().map_err(|_| ParseError::InvalidFormat)?;
                    } else if part.starts_with('Y') {
                        *y = part[1..].parse().map_err(|_| ParseError::InvalidFormat)?;
                    } else if part.starts_with('Z') {
                        *z = part[1..].parse().map_err(|_| ParseError::InvalidFormat)?;
                    }
                }
            }
            Ok(gcode)
        }
        "G05.1" => {
            let q_val = parts.get(1).ok_or(ParseError::InvalidFormat)?;
            if *q_val == "Q1" {
                Ok(Gcode::EnterSplineMode)
            } else if *q_val == "Q0" {
                Ok(Gcode::ExitSplineMode)
            } else {
                Err(ParseError::InvalidFormat)
            }
        }
        // Lines within a G05.1 block are treated as control points.
        // A real parser would have state to know it's in spline mode.
        // Here we assume a line with X/Y/Z without a G-code is a control point.
        _ if command.starts_with('X') || command.starts_with('Y') || command.starts_with('Z') => {
            let mut cp = ControlPoint::new(0.0, 0.0, 0.0);
            for part in &parts {
                 if part.starts_with('X') {
                    cp.x = part[1..].parse().map_err(|_| ParseError::InvalidFormat)?;
                } else if part.starts_with('Y') {
                    cp.y = part[1..].parse().map_err(|_| ParseError::InvalidFormat)?;
                } else if part.starts_with('Z') {
                    cp.z = part[1..].parse().map_err(|_| ParseError::InvalidFormat)?;
                } else if part.starts_with('P') { // Weight for NURBS
                    cp.w = part[1..].parse().map_err(|_| ParseError::InvalidFormat)?;
                }
            }
            Ok(Gcode::SplineControlPoint(cp))
        }
        _ => Ok(Gcode::Unknown),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_g05_1_q1() {
        let line = "G05.1 Q1";
        assert_eq!(parse_line(line), Ok(Gcode::EnterSplineMode));
    }

    #[test]
    fn test_parse_g05_1_q0() {
        let line = "G05.1 Q0";
        assert_eq!(parse_line(line), Ok(Gcode::ExitSplineMode));
    }

    #[test]
    fn test_parse_control_point() {
        let line = "X10.0 Y20.5 Z-5.0";
        let expected = Gcode::SplineControlPoint(ControlPoint::new(10.0, 20.5, -5.0));
        assert_eq!(parse_line(line), Ok(expected));
    }

    #[test]
    fn test_parse_nurbs_control_point() {
        let line = "X10.0 Y20.5 Z-5.0 P0.707";
        let expected = Gcode::SplineControlPoint(ControlPoint::new_rational(10.0, 20.5, -5.0, 0.707));
        assert_eq!(parse_line(line), Ok(expected));
    }

    #[test]
    fn test_spline_buffer_workflow() {
        let mut buffer = SplineBuffer::new();

        // Cannot add points when not active
        assert_eq!(buffer.add_point(ControlPoint::new(1.0, 1.0, 1.0)), Err(ParseError::NotInSplineMode));

        buffer.begin();
        assert!(buffer.is_active);
        assert!(buffer.points.is_empty());

        buffer.add_point(ControlPoint::new(1.0, 1.0, 1.0)).unwrap();
        buffer.add_point(ControlPoint::new(2.0, 2.0, 2.0)).unwrap();
        assert_eq!(buffer.points.len(), 2);

        let points = buffer.end();
        assert!(!buffer.is_active);
        assert!(buffer.points.is_empty());
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].x, 1.0);
    }

    #[test]
    fn test_full_spline_block_parsing_simulation() {
        let gcode_lines = [
            "G01 X0 Y0 Z0",
            "G05.1 Q1",
            "X10 Y20 Z5",
            "X20 Y25 Z6 P0.8",
            "X30 Y15 Z7",
            "G05.1 Q0",
            "G01 X40 Y20 Z10",
        ];

        let mut buffer = SplineBuffer::new();
        let mut parsed_spline_points: Vec<ControlPoint, MAX_SPLINE_POINTS> = Vec::new();

        for line in gcode_lines {
            match parse_line(line).unwrap() {
                Gcode::EnterSplineMode => buffer.begin(),
                Gcode::ExitSplineMode => parsed_spline_points = buffer.end(),
                Gcode::SplineControlPoint(cp) => {
                    if buffer.is_active {
                        buffer.add_point(cp).unwrap();
                    }
                },
                _ => {} // Ignore other commands for this test
            }
        }

        assert_eq!(parsed_spline_points.len(), 3);
        assert_eq!(parsed_spline_points[0], ControlPoint::new(10.0, 20.0, 5.0));
        assert_eq!(parsed_spline_points[1], ControlPoint::new_rational(20.0, 25.0, 6.0, 0.8));
        assert_eq!(parsed_spline_points[2], ControlPoint::new(30.0, 15.0, 7.0));
    }
}
