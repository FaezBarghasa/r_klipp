
// This code is highly dependent on the specific MCU and its memory layout.
// This is a conceptual implementation.

// Represents the backup registers or a dedicated flash page for boot info
struct BootInfo {
    boot_bank: u8,
    magic: u32,
}

const BOOT_MAGIC: u32 = 0xDEADBEEF;

fn write_boot_info(bank: u8) {
    // Write to backup registers or flash
}

fn read_boot_info() -> BootInfo {
    // Read from backup registers or flash
    BootInfo { boot_bank: 1, magic: 0 }
}

pub fn bootloader() {
    let boot_info = read_boot_info();
    if boot_info.magic == BOOT_MAGIC && boot_info.boot_bank == 2 {
        // Start IWDG timer
        // Jump to Bank 2
    } else {
        // Jump to Bank 1
    }
}

pub fn receive_ota_firmware(data: &[u8]) {
    // Write data to flash bank 2
    // ...

    // Verify SHA-256 hash of the written data
    // ...

    // If hash is valid, write to boot info to boot from bank 2
    write_boot_info(2);
    // Reboot
}
