use r_klipp_api::LinkHealth;
use std::fs;

pub fn host_self_update(package_path: &str) {
    // In a real implementation, we'd verify the signature first
    let new_binary = fs::read(package_path).unwrap();

    // Atomic rename for update
    let current_exe = std::env::current_exe().unwrap();
    let old_exe = current_exe.with_extension("old");
    fs::rename(&current_exe, old_exe).unwrap();
    fs::write(current_exe, new_binary).unwrap();

    // Graceful restart
    // This is platform-specific and complex.
}

pub fn mcu_ota(link_health: &LinkHealth, firmware_path: &str) -> Result<(), &'static str> {
    if link_health.rtt_us > 2000 || link_health.dropped_packets > 0 {
        return Err("Link too unstable for safe firmware flashing.");
    }

    // Proceed with MCU OTA
    let firmware = fs::read(firmware_path).unwrap();
    // Send firmware to MCU in chunks

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcu_ota_link_gating() {
        let good_link = LinkHealth { rtt_us: 1000, buffer_fill_percent: 80, dropped_packets: 0 };
        let bad_link = LinkHealth { rtt_us: 3000, buffer_fill_percent: 50, dropped_packets: 1 };

        assert!(mcu_ota(&good_link, "dummy.bin").is_ok());
        assert!(mcu_ota(&bad_link, "dummy.bin").is_err());
    }
}
