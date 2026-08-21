//! Position-Synchronized Hardware Discrete I/O Scheduler.
//!
//! Schedules hardware pin events (camera triggers, vacuum solenoids, laser pulses)
//! at exact microsecond timestamps corresponding to trajectory waypoint encounters.

use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledIoEvent {
    pub pin_id: u8,
    pub target_timestamp_us: u64,
    pub state: bool,
}

#[derive(Debug, Clone)]
pub struct SyncIoScheduler<const CAPACITY: usize = 32> {
    pub events: Vec<ScheduledIoEvent, CAPACITY>,
}

impl<const CAPACITY: usize> SyncIoScheduler<CAPACITY> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Schedules an I/O event at a future hardware timestamp
    pub fn schedule_event(&mut self, pin_id: u8, target_timestamp_us: u64, state: bool) -> Result<(), ScheduledIoEvent> {
        let event = ScheduledIoEvent {
            pin_id,
            target_timestamp_us,
            state,
        };

        self.events.push(event).map_err(|e| e)
    }

    /// Pops all events that are ready to fire up to the given current timestamp
    pub fn poll_due_events(&mut self, current_time_us: u64) -> Vec<ScheduledIoEvent, CAPACITY> {
        let mut due = Vec::new();
        let mut i = 0;
        while i < self.events.len() {
            if self.events[i].target_timestamp_us <= current_time_us {
                let event = self.events.swap_remove(i);
                let _ = due.push(event);
            } else {
                i += 1;
            }
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_io_scheduler_camera_trigger() {
        let mut scheduler = SyncIoScheduler::<8>::new();
        // Camera shutter on pin 5 at 50,000 us (50ms)
        scheduler.schedule_event(5, 50_000, true).unwrap();
        // Camera shutter off on pin 5 at 52,000 us (2ms pulse)
        scheduler.schedule_event(5, 52_000, false).unwrap();

        // At t = 30ms -> no events due
        let due_30 = scheduler.poll_due_events(30_000);
        assert_eq!(due_30.len(), 0);

        // At t = 50ms -> shutter trigger event fires
        let due_50 = scheduler.poll_due_events(50_000);
        assert_eq!(due_50.len(), 1);
        assert_eq!(due_50[0].pin_id, 5);
        assert_eq!(due_50[0].state, true);

        // At t = 55ms -> shutter release event fires
        let due_55 = scheduler.poll_due_events(55_000);
        assert_eq!(due_55.len(), 1);
        assert_eq!(due_55[0].state, false);
    }
}
