use host_ui::{
    AppWindow, AxisPosition, BedMeshPoint, ConsoleLine, FanDevice, FanEntry, GCodeFileInfo,
    MacroEntry, MenuItemInfo, PackageInfo, PowerDevice, PrintStats, SpoolInfo, SystemInfo,
    TemperaturePreset, ThermalSensor, WifiNetwork,
};
use slint::Model;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_all_custom_types_instantiation() {
    let _menu = MenuItemInfo {
        name: "Move".into(),
        icon_name: "move".into(),
        icon: Default::default(),
        panel_name: "move".into(),
        is_submenu: false,
    };

    let _gcode = GCodeFileInfo {
        name: "test.gcode".into(),
        is_dir: false,
        size: "1.0 MB".into(),
        thumbnail: Default::default(),
    };

    let _cline = ConsoleLine {
        text: "echo: M112 Emergency Stop".into(),
        is_input: false,
    };

    let _thermal = ThermalSensor {
        name: "heater_bed".into(),
        actual: 60.0,
        target: 65.0,
        power: 0.85,
    };

    let _preset = TemperaturePreset {
        label: "PLA".into(),
        nozzle: 205.0,
        bed: 60.0,
    };

    let _fan_entry = FanEntry {
        name: "Part Fan".into(),
        speed: 1.0,
        controllable: true,
    };

    let _macro = MacroEntry {
        name: "BED_MESH_CALIBRATE".into(),
        description: "Runs bed mesh calibration".into(),
        visible: true,
    };

    let _mesh_pt = BedMeshPoint {
        col: 0,
        row: 0,
        value: 0.025,
    };

    let _wifi = WifiNetwork {
        ssid: "Lab".into(),
        bssid: "11:22:33:44:55:66".into(),
        signal: 90,
        connected: true,
        known: true,
        security: "WPA2".into(),
    };

    let _pwr = PowerDevice {
        name: "printer_power".into(),
        status: "on".into(),
    };

    let _pkg = PackageInfo {
        name: "klipper".into(),
        version: "v0.12.0".into(),
        up_to_date: true,
        can_restart: true,
    };

    let _spool = SpoolInfo {
        id: 1,
        name: "Prusament Galaxy Black".into(),
        material: "PLA".into(),
        weight_g: 1000.0,
        used_g: 250.0,
        color: slint::Color::from_rgb_u8(20, 20, 20),
    };

    let _axis = AxisPosition {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        e: 0.0,
        homed_axes: "xyz".into(),
    };

    let _stats = PrintStats {
        filename: "test.gcode".into(),
        state: "printing".into(),
        progress: 50.0,
        print_duration_sec: 1800,
        filament_used_mm: 1250.0,
        layer_current: 50,
        layer_total: 100,
        speed_factor: 1.0,
        extrude_factor: 1.0,
    };

    let _sys = SystemInfo {
        hostname: "skipr".into(),
        ip_address: "192.168.1.100".into(),
        cpu_usage: 0.15,
        cpu_cores: Rc::new(slint::VecModel::from(vec![0.1f32, 0.2, 0.1, 0.2])).into(),
        memory_usage: 0.45,
        memory_text: "1.8 GB / 4.0 GB".into(),
        date_time: "2026-08-26".into(),
        os_version: "Debian 11".into(),
        kernel_version: "5.10.0".into(),
        klipper_version: "v0.12.0".into(),
        moonraker_version: "v0.8.0".into(),
    };
}

#[test]
fn test_klipperscreen_full_suite() {
    let app = AppWindow::new().expect("Failed to initialize AppWindow");

    // 1. Verify All 29 Screen Routes
    let screens = [
        "main_menu",
        "job_status",
        "move",
        "temperature",
        "extrude",
        "fan",
        "files",
        "console",
        "bed_level",
        "bed_mesh",
        "zcalibrate",
        "input_shaper",
        "limits",
        "pressure_advance",
        "retraction",
        "fine_tune",
        "pins",
        "network",
        "system",
        "settings",
        "updater",
        "power",
        "shutdown",
        "camera",
        "spoolman",
        "spool_editor",
        "gcode_macros",
        "exclude",
        "led",
    ];

    for screen in &screens {
        app.set_active_screen((*screen).into());
        assert_eq!(app.get_active_screen(), *screen);
    }

    // 2. Verify Telemetry Properties
    app.set_header_time("12:34 PM".into());
    app.set_printer_state("printing".into());
    app.set_nozzle_temp_actual(220.5);
    app.set_nozzle_temp_target(220.0);
    app.set_bed_temp_actual(60.1);
    app.set_bed_temp_target(65.0);
    app.set_chamber_temp_actual(45.0);
    app.set_chamber_temp_target(50.0);

    assert_eq!(app.get_header_time(), "12:34 PM");
    assert_eq!(app.get_printer_state(), "printing");
    assert_eq!(app.get_nozzle_temp_actual(), 220.5);
    assert_eq!(app.get_nozzle_temp_target(), 220.0);
    assert_eq!(app.get_bed_temp_actual(), 60.1);
    assert_eq!(app.get_bed_temp_target(), 65.0);
    assert_eq!(app.get_chamber_temp_actual(), 45.0);
    assert_eq!(app.get_chamber_temp_target(), 50.0);

    // 3. Verify Motion Properties
    app.set_pos_x(150.0);
    app.set_pos_y(120.0);
    app.set_pos_z(25.4);
    app.set_pos_e(100.0);
    app.set_homed_axes("xyz".into());
    app.set_jog_distance(5.0);
    app.set_jog_speed(100.0);

    assert_eq!(app.get_pos_x(), 150.0);
    assert_eq!(app.get_pos_y(), 120.0);
    assert_eq!(app.get_pos_z(), 25.4);
    assert_eq!(app.get_pos_e(), 100.0);
    assert_eq!(app.get_homed_axes(), "xyz");
    assert_eq!(app.get_jog_distance(), 5.0);
    assert_eq!(app.get_jog_speed(), 100.0);

    // 4. Verify Print Job Properties
    app.set_current_print_file("3dbenchy_pla.gcode".into());
    app.set_print_progress(0.82);
    app.set_print_duration_sec(3600);
    app.set_layer_current(180);
    app.set_layer_total(220);
    app.set_speed_factor(1.10);
    app.set_flow_factor(0.98);

    assert_eq!(app.get_current_print_file(), "3dbenchy_pla.gcode");
    assert_eq!(app.get_print_progress(), 0.82);
    assert_eq!(app.get_print_duration_sec(), 3600);
    assert_eq!(app.get_layer_current(), 180);
    assert_eq!(app.get_layer_total(), 220);
    assert_eq!(app.get_speed_factor(), 1.10);
    assert_eq!(app.get_flow_factor(), 0.98);

    // 5. Verify Fans Model
    let fans = vec![
        FanDevice { name: "Part Fan".into(), speed: 1.0, changeable: true },
        FanDevice { name: "Aux Fan".into(), speed: 0.5, changeable: true },
        FanDevice { name: "Chamber Fan".into(), speed: 0.25, changeable: false },
    ];
    app.set_fans(Rc::new(slint::VecModel::from(fans)).into());
    assert_eq!(app.get_fans().row_count(), 3);
    assert_eq!(app.get_fans().row_data(0).unwrap().name, "Part Fan");
    assert_eq!(app.get_fans().row_data(0).unwrap().speed, 1.0);
    assert!(app.get_fans().row_data(0).unwrap().changeable);

    // 6. Verify Files Model
    let files = vec![
        GCodeFileInfo {
            name: "stealthburner.gcode".into(),
            size: "4.5 MB".into(),
            is_dir: false,
            thumbnail: Default::default(),
        },
        GCodeFileInfo {
            name: "calibration".into(),
            size: "".into(),
            is_dir: true,
            thumbnail: Default::default(),
        },
    ];
    app.set_gcode_files(Rc::new(slint::VecModel::from(files)).into());
    assert_eq!(app.get_gcode_files().row_count(), 2);
    assert_eq!(app.get_gcode_files().row_data(0).unwrap().name, "stealthburner.gcode");
    assert!(!app.get_gcode_files().row_data(0).unwrap().is_dir);

    // 7. Verify Console Lines Model
    let lines = vec![
        ConsoleLine { text: "Connecting to Moonraker...".into(), is_input: false },
        ConsoleLine { text: "G28".into(), is_input: true },
        ConsoleLine { text: "ok".into(), is_input: false },
    ];
    app.set_console_lines(Rc::new(slint::VecModel::from(lines)).into());
    assert_eq!(app.get_console_lines().row_count(), 3);
    assert_eq!(app.get_console_lines().row_data(1).unwrap().text, "G28");
    assert!(app.get_console_lines().row_data(1).unwrap().is_input);

    // 8. Verify WiFi Networks Model
    let networks = vec![
        WifiNetwork {
            ssid: "Workshop_WiFi".into(),
            bssid: "00:11:22:33:44:55".into(),
            signal: 88,
            connected: true,
            known: true,
            security: "WPA2".into(),
        },
    ];
    app.set_networks(Rc::new(slint::VecModel::from(networks)).into());
    assert_eq!(app.get_networks().row_count(), 1);
    assert_eq!(app.get_networks().row_data(0).unwrap().ssid, "Workshop_WiFi");
    assert!(app.get_networks().row_data(0).unwrap().connected);

    // 9. Verify System Metrics
    app.set_host_name("mks-skipr-klipper".into());
    app.set_ip_address("192.168.1.120".into());
    app.set_cpu_usage(0.28);
    app.set_mem_usage(0.45);
    app.set_mem_text("1.8 GB / 4.0 GB".into());

    assert_eq!(app.get_host_name(), "mks-skipr-klipper");
    assert_eq!(app.get_ip_address(), "192.168.1.120");
    assert_eq!(app.get_cpu_usage(), 0.28);
    assert_eq!(app.get_mem_usage(), 0.45);
    assert_eq!(app.get_mem_text(), "1.8 GB / 4.0 GB");

    // 10. Verify Callback Registrations
    let nav_hit = Rc::new(Cell::new(false));
    let nh_c = nav_hit.clone();
    app.on_navigate_to(move |_s| nh_c.set(true));

    let estop_hit = Rc::new(Cell::new(false));
    let es_c = estop_hit.clone();
    app.on_emergency_stop(move || es_c.set(true));

    let home_hit = Rc::new(Cell::new(false));
    let hm_c = home_hit.clone();
    app.on_home_axis(move |_a| hm_c.set(true));

    let jog_hit = Rc::new(Cell::new(false));
    let jg_c = jog_hit.clone();
    app.on_jog_axis(move |_a, _d, _s| jg_c.set(true));

    let noz_hit = Rc::new(Cell::new(false));
    let nz_c = noz_hit.clone();
    app.on_set_nozzle_target(move |_t| nz_c.set(true));

    let bed_hit = Rc::new(Cell::new(false));
    let bd_c = bed_hit.clone();
    app.on_set_bed_target(move |_t| bd_c.set(true));

    let fan_hit = Rc::new(Cell::new(false));
    let fn_c = fan_hit.clone();
    app.on_set_fan_speed(move |_i, _s| fn_c.set(true));

    let gcode_hit = Rc::new(Cell::new(false));
    let gc_c = gcode_hit.clone();
    app.on_send_gcode_command(move |_c| gc_c.set(true));

    app.on_disable_motors(|| {});
    app.on_set_chamber_target(|_t| {});
    app.on_extrude(|_d, _s| {});
    app.on_start_print(|_f| {});
    app.on_pause_print(|| {});
    app.on_resume_print(|| {});
    app.on_cancel_print(|| {});
    app.on_refresh_files(|| {});
    app.on_run_macro(|_m| {});
    app.on_update_package(|_p| {});
    app.on_restart_service(|_s| {});
    app.on_shutdown_system(|| {});
    app.on_reboot_system(|| {});
}
