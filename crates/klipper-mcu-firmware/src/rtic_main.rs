// File path: crates/klipper-mcu-firmware/src/rtic_main.rs

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
    use mcu_drivers::stepper::{StepSegment, StepperController};

    const STEPPER_QUEUE_SIZE: usize = 256;

    type LedPin = gpiod::PD12<Output<PushPull>>;

    #[shared]
    struct Shared {
        usart_tx: Tx<USART1>,
        stepper_controller: StepperController<STEPPER_QUEUE_SIZE>,
    }

    #[local]
    struct Local {
        led: LedPin,
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

        // Setup stepper controller (e.g. step pin mask 1)
        let stepper_controller = StepperController::new(1);

        // Schedule periodic software tasks
        heater_task::spawn().ok();
        adc_task::spawn().ok();

        defmt::info!("RTIC Init complete.");

        (
            Shared { usart_tx, stepper_controller },
            Local {
                led,
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
    #[task(binds = TIM2, shared = [stepper_controller], local = [stepper_timer], priority = 4)]
    fn stepper_isr(cx: stepper_isr::Context) {
        // Clear the interrupt flag
        cx.local.stepper_timer.clear_interrupt(TimerEvent::Update);

        cx.shared.stepper_controller.lock(|controller| {
            // Using typical STM32F4 registers for demo (BSRR for GPIOD, ARR for TIM2)
            let bsrr = 0x4002_0C18 as *mut u32; // GPIOD BSRR
            let arr = 0x4000_002C as *mut u32;  // TIM2 ARR
            unsafe {
                controller.execute_next_step_isr(bsrr, arr);
            }
        });
    }

    /// Communication Task - handles incoming serial data.
    #[task(binds = USART1, shared = [stepper_controller], local = [usart_rx], priority = 2)]
    fn usart_task(cx: usart_task::Context) {
        // This task would read bytes from cx.local.usart_rx,
        // feed them to a klipper-proto parser, and on receiving
        // a valid step segment, would push it to the controller queue.
        if let Ok(_byte) = cx.local.usart_rx.read() {
            // ... parsing logic here ...
            // When segment is parsed:
            let segment = StepSegment {
                interval_ticks: 1000,
                direction: true,
                enable_mask: 1,
            };
            cx.shared.stepper_controller.lock(|controller| {
                let _ = controller.enqueue_segment(segment);
            });
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
