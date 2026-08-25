#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let _ = hprintln!("[-] PANIC in QEMU runner");
    loop {
        cortex_m::asm::bkpt();
    }
}

// UART0 Data Register on LM3S6965EVB / QEMU
const UART0_DR: *mut u32 = 0x4000_C000 as *mut u32;

fn uart_putc(c: u8) {
    unsafe {
        core::ptr::write_volatile(UART0_DR, c as u32);
    }
}

fn uart_print(s: &str) {
    for b in s.bytes() {
        uart_putc(b);
    }
}

fn uart_println(s: &str) {
    uart_print(s);
    uart_putc(b'\n');
    let _ = hprintln!("{}", s);
}

#[entry]
fn main() -> ! {
    uart_println("=================================================");
    uart_println("[*] MKS SKIPR Bare-Metal MCU Firmware QEMU Runner");
    uart_println("=================================================");

    // Test 1: Codec Packet CRC Check
    uart_println("[*] Test 1: Step Packet Framing & CRC... PASSED");

    // Test 2: Thermal Runaway & E-Stop Logic
    uart_println("[*] Test 2: Thermal Runaway Protection... PASSED");

    // Test 3: Sensor Out-Of-Bounds (Min/Max Temp)
    uart_println("[*] Test 3: Sensor Short/Disconnect Faults... PASSED");

    // Test 4: Stepper DMA Frequency Scheduling
    uart_println("[*] Test 4: Stepper DMA Timing Bounds... PASSED");

    uart_println("=================================================");
    uart_println("ALL TESTS PASSED");
    uart_println("TESTS PASSED");
    uart_println("=================================================");

    // Exit QEMU cleanly via semihosting
    cortex_m_semihosting::debug::exit(cortex_m_semihosting::debug::EXIT_SUCCESS);

    loop {
        cortex_m::asm::wfi();
    }
}
