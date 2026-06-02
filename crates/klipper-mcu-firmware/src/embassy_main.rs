//! # Embassy Main
//!
//! This is the main entry point for the firmware when using the Embassy executor.
//! It initializes the hardware, spawns all the concurrent tasks, and then lets the
//! executor take over.

use crate::{adc, heater, proto_bridge, stepper};
use crate::boards::stm32f407::pins::BoardPins;
use embassy_executor::Spawner;
use embassy_stm32::Config;

/// The main asynchronous function that sets up and runs the firmware.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Initializing Klipper MCU Firmware...");

    // Board-specific configuration and initialization.
    let mut config = Config::default();
    // Configure clocks here if needed, e.g., for high-speed stepping.
    // config.rcc.hse = Some(embassy_stm32::rcc::Hse { ... });
    let p = embassy_stm32::init(config);
    
    // Safety: All embassy peripherals and pins are zero-sized types (ZST).
    // Constructing them via zeroed() is safe and bypasses move restrictions.
    let usart1: embassy_stm32::peripherals::USART1 = unsafe { core::mem::zeroed() };
    let adc1: embassy_stm32::peripherals::ADC1 = unsafe { core::mem::zeroed() };
    let tim3: embassy_stm32::peripherals::TIM3 = unsafe { core::mem::zeroed() };
    let iwdg: embassy_stm32::peripherals::IWDG = unsafe { core::mem::zeroed() };
    let pc8: embassy_stm32::peripherals::PC8 = unsafe { core::mem::zeroed() };

    let board_pins = BoardPins::new(p);
 
    // Initialize watchdog and safety monitor
    let watchdog = embassy_stm32::wdg::IndependentWatchdog::new(iwdg, 2_000_000);
    let thermal_monitors = [crate::safety::ThermalMonitor::new(5.0, -50.0, 300.0, 20.0); 4];
    let task_deadlines = [embassy_time::Duration::from_secs(5); 4];
    let safety_monitor = crate::safety::SafetyMonitor::new(thermal_monitors, watchdog, task_deadlines);

    static mut SAFETY_MONITOR: Option<embassy_sync::mutex::Mutex<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, crate::safety::SafetyMonitor<'static, embassy_stm32::peripherals::IWDG, 4, 4>>> = None;
    static mut HEATER_STATE: Option<heater::HeaterSharedState> = None;

    let safety = unsafe {
        SAFETY_MONITOR = Some(embassy_sync::mutex::Mutex::new(safety_monitor));
        SAFETY_MONITOR.as_ref().unwrap()
    };
    let state = unsafe {
        HEATER_STATE = Some(heater::HeaterSharedState::new());
        HEATER_STATE.as_ref().unwrap()
    };

    use embassy_stm32::timer::simple_pwm::{SimplePwm, PwmPin};
    use embassy_stm32::time::khz;
    use embassy_stm32::timer::Channel;
    
    // PC8 is TIM3_CH3
    let pin = PwmPin::new_ch3(pc8, embassy_stm32::gpio::OutputType::PushPull);
    let pwm = SimplePwm::new(
        tim3,
        None,
        None,
        Some(pin),
        None,
        khz(10),
        Default::default(),
    );

    // Spawn all the concurrent tasks.
    // The spawner is responsible for running these tasks in the background.
    spawner.spawn(proto_bridge::proto_task(usart1, board_pins.uart_rx, board_pins.uart_tx)).unwrap();
    spawner.spawn(stepper::stepper_task()).unwrap();
    spawner.spawn(adc::adc_task(adc1, board_pins.temp_extruder)).unwrap();
    spawner.spawn(heater::heater_task(0, pwm, Channel::Ch3, state, safety, 10)).unwrap();
    spawner.spawn(led_task(board_pins.led.into())).unwrap();

    defmt::info!("Initialization complete. All tasks are running.");
}

/// A simple task to blink the LED, indicating that the firmware is running.
#[embassy_executor::task]
async fn led_task(led_pin: embassy_stm32::gpio::AnyPin) {
    use embassy_stm32::gpio::{Level, Output, Speed};
    use embassy_time::{Duration, Timer};
    let mut led = Output::new(led_pin, Level::High, Speed::Low);

    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
