//! G-code Parser and Dispatcher
//!
//! This module provides a parser for G-code commands and a central dispatcher
//! that receives commands from a channel and acts on them, updating the printer
//! state and sending instructions to the MCU.

use crate::{
    config::PrinterConfig,
    kinematics::{CartesianKinematics, Kinematics},
    state::{Position, PrinterState},
};
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{info, warn};

/// A command to be sent to the MCU.
#[derive(Debug)]
pub enum McuCommand {
    Move(Vec<crate::kinematics::Step>),
    Home,
    EmergencyStop,
    GetTemp,
}

/// Represents a single parsed G-code command.
#[derive(Debug, PartialEq, Clone)]
pub struct GCode {
    pub command: String,
    pub params: Vec<(char, f32)>,
}

/// G-code parser that handles commands, parameters, and comments.
pub fn parse_gcode(line: &str) -> Option<GCode> {
    let clean_line = line.split(';').next().unwrap_or("").trim();
    if clean_line.is_empty() {
        return None;
    }

    let mut parts = clean_line.split_whitespace();
    let command = parts.next()?.to_uppercase();
    let mut params = Vec::new();

    for part in parts {
        let mut chars = part.chars();
        if let (Some(key), Some(value_str)) = (chars.next(), chars.next_to_string().into()) {
            if let Ok(value) = value_str.parse::<f32>() {
                params.push((key.to_ascii_uppercase(), value));
            }
        }
    }

    Some(GCode { command, params })
}

/// The central dispatcher for processing G-code commands.
pub struct GCodeDispatcher {
    config: Arc<PrinterConfig>,
    state: Arc<Mutex<PrinterState>>,
    mcu_tx: Sender<McuCommand>,
    kinematics: Box<dyn Kinematics + Send>,
}

impl GCodeDispatcher {
    /// Creates a new `GCodeDispatcher`.
    pub fn new(
        config: Arc<PrinterConfig>,
        state: Arc<Mutex<PrinterState>>,
        mcu_tx: Sender<McuCommand>,
    ) -> Self {
        // In a real application, you would select the kinematics based on the config.
        let kinematics = Box::new(CartesianKinematics {
            steps_per_mm_x: config.stepper_x.steps_per_mm,
            steps_per_mm_y: config.stepper_y.steps_per_mm,
            steps_per_mm_z: config.stepper_z.steps_per_mm,
        });
        Self {
            config,
            state,
            mcu_tx,
            kinematics,
        }
    }

    /// The main run loop that listens for and processes G-code commands.
    pub async fn run(&mut self, mut gcode_rx: Receiver<GCode>) {
        info!("G-code dispatcher is running.");
        while let Some(gcode) = gcode_rx.recv().await {
            info!("Dispatching G-code: {:?}", gcode);
            if let Err(e) = self.dispatch(gcode).await {
                warn!("Error dispatching G-code: {}", e);
            }
        }
    }

    /// Routes a G-code command to the appropriate handler.
    async fn dispatch(&mut self, gcode: GCode) -> Result<()> {
        match gcode.command.as_str() {
            "G0" | "G1" => self.handle_g0_g1(gcode).await?,
            "G28" => self.handle_g28().await?,
            "M112" => self.handle_m112().await?,
            _ => warn!("Unknown G-code command: {}", gcode.command),
        }
        Ok(())
    }

    /// Handles G0/G1 (Linear Move) commands.
    async fn handle_g0_g1(&mut self, gcode: GCode) -> Result<()> {
        let mut current_pos = self.state.lock().position.clone();
        let mut new_pos = current_pos.clone();

        for (param, value) in gcode.params {
            match param {
                'X' => new_pos.x = value,
                'Y' => new_pos.y = value,
                'Z' => new_pos.z = value,
                'E' => new_pos.e = value,
                'F' => { /* TODO: Handle feedrate */ }
                _ => {}
            }
        }

        let steps = self.kinematics.calculate_move(&current_pos, &new_pos);
        if !steps.is_empty() {
            self.mcu_tx.send(McuCommand::Move(steps)).await?;
            // In a real system, you'd wait for an "ok" from the MCU before updating state.
            self.state.lock().position = new_pos;
        }

        Ok(())
    }

    /// Handles G28 (Auto Home) commands.
    async fn handle_g28(&mut self) -> Result<()> {
        info!("Homing axes...");
        self.mcu_tx.send(McuCommand::Home).await?;
        // The MCU would eventually report back that homing is complete,
        // which would then update the state.
        self.state.lock().position = Position::default();
        info!("Homing complete. Position reset.");
        Ok(())
    }

    /// Handles M112 (Emergency Stop) commands.
    async fn handle_m112(&mut self) -> Result<()> {
        warn!("Emergency stop requested!");
        self.mcu_tx.send(McuCommand::EmergencyStop).await?;
        self.state.lock().status = crate::state::PrinterStatus::Error;
        self.state.lock().status_message = "Emergency Stop".to_string();
        Ok(())
    }
}

