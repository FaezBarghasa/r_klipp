//! Industrial Probing (G38.2/G38.3) and Enclosure Safety Interlock Supervisor.
//!
//! Handles:
//! - Sub-microsecond touch probe contact latching
//! - Instant motion abort on probe contact (`G38.2`)
//! - Hardware enclosure safety interlock (cuts spindle PWM and engages axis E-Stop if door opens during run)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeState {
    Idle,
    ProbingToward,
    ProbingAway,
    TriggeredContact,
    ErrorMissingContact,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbeSupervisor {
    pub state: ProbeState,
    pub latched_position: [f64; 3],
    pub probe_active_low: bool,
}

impl ProbeSupervisor {
    pub fn new(probe_active_low: bool) -> Self {
        Self {
            state: ProbeState::Idle,
            latched_position: [0.0; 3],
            probe_active_low,
        }
    }

    pub fn start_probing(&mut self) {
        self.state = ProbeState::ProbingToward;
    }

    /// Evaluates touch probe digital pin state and latches position on contact
    pub fn on_probe_pin_change(&mut self, pin_high: bool, current_pos: [f64; 3]) -> bool {
        let is_contact = if self.probe_active_low { !pin_high } else { pin_high };

        if self.state == ProbeState::ProbingToward && is_contact {
            self.state = ProbeState::TriggeredContact;
            self.latched_position = current_pos;
            true // Trigger instant halt
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnclosureInterlock {
    pub door_open: bool,
    pub is_estop_engaged: bool,
}

impl EnclosureInterlock {
    pub fn new() -> Self {
        Self {
            door_open: false,
            is_estop_engaged: false,
        }
    }

    /// Evaluates door switch state. If machine is active and door opens, triggers immediate E-Stop.
    pub fn update_door_switch(&mut self, door_open: bool, machine_active: bool) -> bool {
        self.door_open = door_open;
        if self.door_open && machine_active {
            self.is_estop_engaged = true;
            true // Trip E-Stop
        } else {
            false
        }
    }

    pub fn clear_estop(&mut self) -> Result<(), ()> {
        if self.door_open {
            Err(()) // Cannot clear E-Stop while door is still open
        } else {
            self.is_estop_engaged = false;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_contact_latching() {
        let mut supervisor = ProbeSupervisor::new(true); // Active low probe
        supervisor.start_probing();

        // Pin high -> no contact
        assert!(!supervisor.on_probe_pin_change(true, [10.0, 10.0, 5.0]));
        assert_eq!(supervisor.state, ProbeState::ProbingToward);

        // Pin goes low -> contact made!
        assert!(supervisor.on_probe_pin_change(false, [10.0, 10.0, 2.345]));
        assert_eq!(supervisor.state, ProbeState::TriggeredContact);
        assert_eq!(supervisor.latched_position, [10.0, 10.0, 2.345]);
    }

    #[test]
    fn test_enclosure_interlock_estop() {
        let mut interlock = EnclosureInterlock::new();

        // Door opens while machine is cutting -> E-Stop engaged
        assert!(interlock.update_door_switch(true, true));
        assert!(interlock.is_estop_engaged);

        // Attempt to clear while open -> fails
        assert!(interlock.clear_estop().is_err());

        // Close door -> clear succeeds
        interlock.door_open = false;
        assert!(interlock.clear_estop().is_ok());
        assert!(!interlock.is_estop_engaged);
    }
}
