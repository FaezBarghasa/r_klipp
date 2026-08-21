pub mod hal;
pub mod pipeline;
pub mod export;
pub mod physics;
pub mod sensors;
pub mod fake_mcu;
pub mod harness;

pub use hal::{MockHal, MockHalState, MockStepEvent};
pub use pipeline::{SimulationPipeline, SimulationReport, TrajectoryPoint};
pub use export::{export_to_csv, export_to_svg};
pub use fake_mcu::{SimMcu, McuCommand, McuResponse};
