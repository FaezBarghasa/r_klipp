//! Global Printer State Machine with Strict Transition Rules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultReason {
    ThermalRunaway(String),
    EndstopTriggeredUnexpectedly(String),
    WatchdogTimeout,
    EmergencyStopPressed,
    SerialLinkLoss,
    KinematicsSingularity,
    Generic(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrinterState {
    Idle,
    Homing,
    Printing,
    Paused,
    Error(FaultReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateEvent {
    StartHoming,
    HomingComplete,
    StartPrint,
    PausePrint,
    ResumePrint,
    FinishPrint,
    CancelPrint,
    Fault(FaultReason),
    EmergencyStop,
    ClearFault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    InvalidTransition { current: PrinterState, event: StateEvent },
    NotHomed,
}

pub struct GlobalStateMachine {
    state: PrinterState,
    is_homed: bool,
}

impl GlobalStateMachine {
    pub fn new() -> Self {
        Self {
            state: PrinterState::Idle,
            is_homed: false,
        }
    }

    pub fn state(&self) -> &PrinterState {
        &self.state
    }

    pub fn is_homed(&self) -> bool {
        self.is_homed
    }

    pub fn transition(&mut self, event: StateEvent) -> Result<PrinterState, TransitionError> {
        // Emergency Stop and Faults can happen from ANY state
        match &event {
            StateEvent::EmergencyStop => {
                self.state = PrinterState::Error(FaultReason::EmergencyStopPressed);
                return Ok(self.state.clone());
            }
            StateEvent::Fault(reason) => {
                self.state = PrinterState::Error(reason.clone());
                return Ok(self.state.clone());
            }
            _ => {}
        }

        match (&self.state, &event) {
            (PrinterState::Idle, StateEvent::StartHoming) => {
                self.state = PrinterState::Homing;
                Ok(self.state.clone())
            }
            (PrinterState::Homing, StateEvent::HomingComplete) => {
                self.is_homed = true;
                self.state = PrinterState::Idle;
                Ok(self.state.clone())
            }
            (PrinterState::Idle, StateEvent::StartPrint) => {
                if !self.is_homed {
                    return Err(TransitionError::NotHomed);
                }
                self.state = PrinterState::Printing;
                Ok(self.state.clone())
            }
            (PrinterState::Printing, StateEvent::PausePrint) => {
                self.state = PrinterState::Paused;
                Ok(self.state.clone())
            }
            (PrinterState::Paused, StateEvent::ResumePrint) => {
                self.state = PrinterState::Printing;
                Ok(self.state.clone())
            }
            (PrinterState::Printing, StateEvent::FinishPrint) | (PrinterState::Paused, StateEvent::CancelPrint) | (PrinterState::Printing, StateEvent::CancelPrint) => {
                self.state = PrinterState::Idle;
                Ok(self.state.clone())
            }
            (PrinterState::Error(_), StateEvent::ClearFault) => {
                self.state = PrinterState::Idle;
                self.is_homed = false; // Reset homing requirement on error recovery
                Ok(self.state.clone())
            }
            (current, event) => Err(TransitionError::InvalidTransition {
                current: current.clone(),
                event: event.clone(),
            }),
        }
    }
}

impl Default for GlobalStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_state_flow() {
        let mut sm = GlobalStateMachine::new();
        assert_eq!(*sm.state(), PrinterState::Idle);

        // Printing without homing should fail
        assert!(sm.transition(StateEvent::StartPrint).is_err());

        // Homing flow
        assert_eq!(sm.transition(StateEvent::StartHoming).unwrap(), PrinterState::Homing);
        assert_eq!(sm.transition(StateEvent::HomingComplete).unwrap(), PrinterState::Idle);
        assert!(sm.is_homed());

        // Print -> Pause -> Resume -> Finish
        assert_eq!(sm.transition(StateEvent::StartPrint).unwrap(), PrinterState::Printing);
        assert_eq!(sm.transition(StateEvent::PausePrint).unwrap(), PrinterState::Paused);
        assert_eq!(sm.transition(StateEvent::ResumePrint).unwrap(), PrinterState::Printing);
        assert_eq!(sm.transition(StateEvent::FinishPrint).unwrap(), PrinterState::Idle);
    }

    #[test]
    fn test_emergency_stop_from_any_state() {
        let mut sm = GlobalStateMachine::new();
        sm.transition(StateEvent::EmergencyStop).unwrap();
        assert!(matches!(sm.state(), PrinterState::Error(FaultReason::EmergencyStopPressed)));
    }
}
