//! # End-to-End Motion Integration Test
//!
//! This test verifies the complete motion pipeline, from the `motion` planner
//! to the `stepper` controller. It ensures that a planned move is correctly
//! translated into a sequence of step commands that result in the expected
//! final motor positions.

use heapless::spsc::Queue;
use motion::planner::MotionPlanner;
use mcu_drivers::stepper::{StepCommand, StepperController};

// Mock hardware traits for the test environment
mod mock_hw {
    use core::cell::RefCell;
    use critical_section::Mutex;
    use mcu_drivers::stepper::{AtomicGpioPort, Timer};

    #[derive(Default)]
    pub struct MockPort(u8);
    impl AtomicGpioPort for MockPort {
        fn set_and_clear_atomic(&mut self, set_mask: u8, _clear_mask: u8) {
            self.0 |= set_mask;
        }
        fn write(&mut self, mask: u8) {
            self.0 = mask;
        }
    }

    #[derive(Default)]
    pub struct MockTimer {
        pub stopped: bool,
    }
    impl Timer for MockTimer {
        fn schedule_next(&mut self, _ticks: u16) {
            self.stopped = false;
        }
        fn trigger_now(&mut self) {
            self.stopped = false;
        }
        fn stop(&mut self) {
            self.stopped = true;
        }
    }

    // Static resources for the test
    pub static MOCK_STEP_PORT: Mutex<RefCell<MockPort>> = Mutex::new(RefCell::new(MockPort(0)));
    pub static MOCK_DIR_PORT: Mutex<RefCell<MockPort>> = Mutex::new(RefCell::new(MockPort(0)));
    pub static MOCK_TIMER: Mutex<RefCell<MockTimer>> = Mutex::new(RefCell::new(MockTimer::default()));
}

#[test]
fn test_full_move_pipeline() {
    // 1. --- Setup ---
    static mut COMMAND_QUEUE: Queue<StepCommand, 256> = Queue::new();
    let (mut producer, consumer) = unsafe { COMMAND_QUEUE.split() };

    let mut planner = MotionPlanner::new();
    let mut controller = StepperController::<2>::new(consumer);

    // 2. --- Plan a Move ---
    let mut target_pos = [0; 8];
    target_pos[0] = 100; // Move stepper 0 by 100 steps forward
    target_pos[1] = 50;  // Move stepper 1 by 50 steps backward
    planner.current_position[1] = 100; // Set initial position for stepper 1

    planner.plan_move(target_pos, 1000.0, 5000.0).expect("Failed to plan move");

    // 3. --- Generate Steps ---
    let steps_generated = planner.generate_steps(&mut producer).expect("Failed to generate steps");
    assert_eq!(steps_generated, 100); // Dominant axis has 100 steps
    assert_eq!(producer.len(), 100);

    // 4. --- Execute Steps ---
    controller.start(&mock_hw::MOCK_DIR_PORT, &mock_hw::MOCK_TIMER);
    for _ in 0..steps_generated {
        controller.on_timer_interrupt(
            &mock_hw::MOCK_STEP_PORT,
            &mock_hw::MOCK_DIR_PORT,
            &mock_hw::MOCK_TIMER,
        );
    }

    // 5. --- Verify Final State ---
    assert_eq!(controller.get_position(0), Some(100)); // Stepper 0 moved +100
    assert_eq!(controller.get_position(1), Some(50));  // Stepper 1 moved from 100 to 50 (-50)
    
    // Check if the timer was stopped after the last step
    critical_section::with(|cs| {
        assert!(mock_hw::MOCK_TIMER.borrow(cs).borrow().stopped);
    });
}
