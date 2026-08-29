use host_ui::{AppWindow, ConsoleLine, FanDevice, GCodeFileInfo};
use slint::{ComponentHandle, Model};
use std::env;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let screen = if args.len() > 1 {
        args[1].clone()
    } else {
        "main_menu".to_string()
    };

    let app = AppWindow::new().expect("Failed to initialize Slint AppWindow");

    // Populate mock telemetry matching KlipperScreen standard display
    app.set_header_time("12:45 PM".into());
    app.set_printer_state("printing".into());
    app.set_nozzle_temp_actual(215.5);
    app.set_nozzle_temp_target(220.0);
    app.set_bed_temp_actual(60.2);
    app.set_bed_temp_target(65.0);
    app.set_chamber_temp_actual(45.0);
    app.set_chamber_temp_target(50.0);

    // Motion
    app.set_pos_x(125.4);
    app.set_pos_y(100.0);
    app.set_pos_z(14.8);
    app.set_homed_axes("xyz".into());
    app.set_jog_distance(10.0);
    app.set_jog_speed(100.0);

    // Print job
    app.set_current_print_file("voron_cube_v2.gcode".into());
    app.set_print_progress(0.64);
    app.set_print_duration_sec(2840);
    app.set_layer_current(78);
    app.set_layer_total(120);
    app.set_speed_factor(1.05);
    app.set_flow_factor(1.00);

    // GCode files
    let files = vec![
        GCodeFileInfo {
            name: "voron_cube_v2.gcode".into(),
            size: "1.2 MB".into(),
            is_dir: false,
            thumbnail: Default::default(),
        },
        GCodeFileInfo {
            name: "3dbenchy_pla_0.2.gcode".into(),
            size: "4.5 MB".into(),
            is_dir: false,
            thumbnail: Default::default(),
        },
        GCodeFileInfo {
            name: "stealthburner_main.gcode".into(),
            size: "8.9 MB".into(),
            is_dir: false,
            thumbnail: Default::default(),
        },
        GCodeFileInfo {
            name: "calibration_prints".into(),
            size: "dir".into(),
            is_dir: true,
            thumbnail: Default::default(),
        },
    ];
    app.set_gcode_files(Rc::new(slint::VecModel::from(files)).into());

    // Fans
    let fans = vec![
        FanDevice { name: "Part Cooling Fan".into(), speed: 1.0, changeable: true },
        FanDevice { name: "Chamber Exhaust Fan".into(), speed: 0.5, changeable: true },
    ];
    app.set_fans(Rc::new(slint::VecModel::from(fans)).into());

    // Console
    let console = vec![
        ConsoleLine { text: "r_klipp Slint UI v0.1.0 Ready".into(), is_input: false },
        ConsoleLine { text: "G28 (Homing all axes)".into(), is_input: true },
        ConsoleLine { text: "ok".into(), is_input: false },
        ConsoleLine { text: "SDCARD_PRINT_FILE FILENAME=\"voron_cube_v2.gcode\"".into(), is_input: true },
        ConsoleLine { text: "File opened: voron_cube_v2.gcode Size: 1245088".into(), is_input: false },
    ];
    app.set_console_lines(Rc::new(slint::VecModel::from(console)).into());

    // System
    app.set_host_name("mks-skipr-rklipp".into());
    app.set_ip_address("192.168.1.185".into());
    app.set_cpu_usage(0.24);
    app.set_mem_usage(0.38);
    app.set_mem_text("1.5 GB / 4.0 GB".into());

    app.set_active_screen(screen.clone().into());
    println!("Loaded screen: {}", screen);

    // Auto-quit after rendering frame (e.g. 500ms) if single snapshot mode
    if args.iter().any(|a| a == "--auto-quit") {
        slint::Timer::single_shot(Duration::from_millis(600), move || {
            let _ = slint::quit_event_loop();
        });
    }

    app.run().unwrap();
}
