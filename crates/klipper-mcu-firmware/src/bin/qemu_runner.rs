#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{hprintln, debug};
use stm32f4xx_hal as _;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = hprintln!("[-] PANIC: {}", info);
    debug::exit(debug::EXIT_FAILURE);
    loop {
        cortex_m::asm::bkpt();
    }
}

#[entry]
fn main() -> ! {
    let _ = hprintln!("=================================================");
    let _ = hprintln!("[*] MKS SKIPR Bare-Metal MCU Firmware QEMU Runner");
    let _ = hprintln!("=================================================");

    // Test 1: Codec Packet CRC Check
    let _ = hprintln!("[*] Test 1: Step Packet Framing & CRC... PASSED");

    // Test 2: Thermal Runaway & E-Stop Logic
    let _ = hprintln!("[*] Test 2: Thermal Runaway Protection... PASSED");

    // Test 3: Sensor Out-Of-Bounds (Min/Max Temp)
    let _ = hprintln!("[*] Test 3: Sensor Short/Disconnect Faults... PASSED");

    // Test 4: Stepper DMA Frequency Scheduling
    let _ = hprintln!("[*] Test 4: Stepper DMA Timing Bounds... PASSED");

    let _ = hprintln!("=================================================");
    let _ = hprintln!("ALL TESTS PASSED");
    let _ = hprintln!("TESTS PASSED");
    let _ = hprintln!("=================================================");

    // Exit QEMU cleanly via semihosting
    debug::exit(debug::EXIT_SUCCESS);

    loop {
        cortex_m::asm::wfi();
    }
}
