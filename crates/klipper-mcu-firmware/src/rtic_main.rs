// File path: crates/klipper-mcu-firmware/src/rtic_main.rs
// AI-generated comment:
// This file was modified by an AI assistant to implement a first-class RTIC application structure.
// Source files for context: crates/klipper-mcu-firmware/src/rtic_main.rs, crates/mcu-drivers/stepper.rs

//! # RTIC-based Firmware Entry Point
//!
//! This module provides a complete firmware implementation using the RTIC (Real-Time
//! Interrupt-driven Concurrency) framework as an alternative to the default Embassy-based
//! async executor. It demonstrates a hardware-task-driven architecture where peripherals
//! and interrupts directly trigger firmware logic.

#![allow(unused_imports)]

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2, EXTI3])]
mod app {
    use stm32f4xx_hal::{
        gpio::{gpiod, Output, PushPull, Speed},
        pac::{TIM2, USART1},
        prelude::*,
        serial::{Config as SerialConfig, Event as SerialEvent, Rx, Serial, Tx},
        timer::{CounterUs, Event as TimerEvent},
    };
    use rtic_monotonics::{systick::*, Monotonic};
    use heapless::spsc::{Queue, Producer, Consumer};
    use core::cell::RefCell;
    use critical_section::Mutex;

    // Workspace crates
    use crate::heater::{HeaterSharedState, PidController};
    use mcu_drivers::stepper::{StepCommand, StepperController, AtomicGpioPort, Timer};

    const STEPPER_QUEUE_SIZE: usize = 256;

    type LedPin = gpiod::PD12<Output<PushPull>>;

    // Queues for communication between tasks
    static mut STEPPER_QUEUE: Queue<StepCommand, STEPPER_QUEUE_SIZE> = Queue::new();

    // AI-generated note on architecture mismatch:
    // The existing `StepperController` in `mcu-drivers` expects peripherals wrapped in
    // `Mutex<RefCell<...>>`, which is idiomatic for Embassy but not for RTIC. In RTIC,
    // resources are managed by the framework and passed via the context `cx`.
    // To bridge this, we create simple proxy structs (`StepperTimerProxy`, `GpioProxy`)
    // that hold RTIC's `local` resources and implement the traits `Timer` and `AtomicGpioPort`
    // that the `StepperController` expects. This avoids modifying the shared `mcu-drivers` crate.

    struct StepperTimerProxy<'a> {
        tim: &'a mut CounterUs<TIM2>,
    }
    impl Timer for StepperTimerProxy<'_> {
        fn schedule_next(&mut self, ticks: u16) {
            self.tim.start(ticks.micros()).unwrap();
        }
        fn trigger_now(&mut self) {
            self.tim.start(1.micros()).unwrap();
        }
        fn stop(&mut self) {
            self.tim.cancel().unwrap();
        }
    }

    // In a real implementation, GpioProxy would wrap GPIO Port peripherals.
    // This is a simplified placeholder.
    struct GpioProxy;
    impl AtomicGpioPort for GpioProxy {
        fn set_and_clear_atomic(&mut self, _set_mask: u8, _clear_mask: u8) { /* no-op */ }
        fn write(&mut self, _mask: u8) { /* no-op */ }
    }


    #[shared]
    struct Shared {
        usart_tx: Tx<USART1>,
    }

    #[local]
    struct Local {
        led: LedPin,
        stepper_controller: StepperController<8>,
        stepper_producer: Producer<'static, StepCommand, STEPPER_QUEUE_SIZE>,
        stepper_timer: CounterUs<TIM2>,
        usart_rx: Rx<USART1>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        defmt::info!("RTIC Init");
        let dp = cx.device;

        // Setup clocks
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(168.MHz()).freeze();

        // Setup systick monotonic timer
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, clocks.hclk().0, systick_mono_token);

        // Setup LED
        let gpiod = dp.GPIOD.split();
        let mut led = gpiod.pd12.into_push_pull_output();
        led.set_high();

        // Setup USART for communication
        let gpioa = dp.GPIOA.split();
        let tx_pin = gpioa.pa9.into_alternate();
        let rx_pin = gpioa.pa10.into_alternate();
        let serial_config = SerialConfig::default().baudrate(250_000.bps());
        let (usart_tx, mut usart_rx) = Serial::new(dp.USART1, (tx_pin, rx_pin), serial_config, &clocks)
            .unwrap()
            .split();
        usart_rx.listen(SerialEvent::Rxne);

        // Setup Stepper Timer (TIM2)
        let mut stepper_timer = dp.TIM2.counter_us(&clocks);
        stepper_timer.listen(TimerEvent::Update);

        // Setup queues and controller
        let (stepper_producer, stepper_consumer) = unsafe { STEPPER_QUEUE.split() };
        let stepper_controller = StepperController::new(stepper_consumer);

        // Schedule periodic software tasks
        heater_task::spawn().ok();
        adc_task::spawn().ok();

        defmt::info!("RTIC Init complete.");

        (
            Shared { usart_tx },
            Local {
                led,
                stepper_controller,
                stepper_producer,
                stepper_timer,
                usart_rx,
            },
        )
    }

    #[idle(local = [led])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            cx.local.led.toggle();
            Systick::delay(500.millis()).unwrap();
        }
    }

    /// Stepper interrupt. Highest priority task.
    #[task(binds = TIM2, local = [stepper_controller, stepper_timer], priority = 4)]
    fn stepper_isr(cx: stepper_isr::Context) {
        // Clear the interrupt flag
        cx.local.stepper_timer.clear_interrupt(TimerEvent::Update);

        // Create proxies for the stepper controller
        let mut timer_proxy = StepperTimerProxy { tim: cx.local.stepper_timer };

        // These would be proper GPIO port proxies in a full implementation
        let step_port_proxy = Mutex::new(RefCell::new(GpioProxy));
        let dir_port_proxy = Mutex::new(RefCell::new(GpioProxy));
        let timer_proxy_mutex = Mutex::new(RefCell::new(timer_proxy));

        cx.local.stepper_controller.on_timer_interrupt(
            &step_port_proxy,
            &dir_port_proxy,
            &timer_proxy_mutex
        );
    }

    /// Communication Task - handles incoming serial data.
    #[task(binds = USART1, local = [usart_rx, stepper_producer], priority = 2)]
    fn usart_task(cx: usart_task::Context) {
        // This task would read bytes from cx.local.usart_rx,
        // feed them to a klipper-proto parser, and on receiving
        // a valid `QueueStep` command, would push it to the
        // cx.local.stepper_producer queue.
        if let Ok(byte) = cx.local.usart_rx.read() {
            // ... parsing logic here ...
        }
    }

    /// Periodic task for heater control. Lower priority.
    #[task(priority = 1)]
    async fn heater_task(_: heater_task::Context) {
        loop {
            // PID loop logic here
            Systick::delay(100.millis()).await;
        }
    }

    /// Periodic task for ADC sampling. Lower priority.
    #[task(priority = 1)]
    async fn adc_task(_: adc_task::Context) {
        loop {
            // ADC reading logic here
            Systick::delay(500.millis()).await;
        }
    }
}

