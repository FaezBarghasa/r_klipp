#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Plane {
    XY, // G17
    XZ, // G18
    YZ, // G19
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Units {
    Inches, // G20
    Millimeters, // G21
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceMode {
    Absolute, // G90
    Relative, // G91
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeedrateMode {
    UnitsPerMinute, // G94
    InverseTime,    // G93
    UnitsPerRevolution, // G95
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    G54,
    G55,
    G56,
    G57,
    G58,
    G59,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalState {
    pub plane: Plane,
    pub units: Units,
    pub distance_mode: DistanceMode,
    pub feedrate_mode: FeedrateMode,
    pub coordinate_system: CoordinateSystem,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            plane: Plane::XY,
            units: Units::Millimeters,
            distance_mode: DistanceMode::Absolute,
            feedrate_mode: FeedrateMode::UnitsPerMinute,
            coordinate_system: CoordinateSystem::G54,
        }
    }
}
