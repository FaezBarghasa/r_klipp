use host_ui::{AppWindow, ConsoleLine, GCodeFileInfo};
use slint::Model;
use std::rc::Rc;

#[test]
fn test_slint_ui_full_lifecycle() {
    // 1. Initialize Slint AppWindow
    let window = AppWindow::new().expect("Failed to initialize Slint AppWindow");

    // 2. Test Nozzle & Bed Temperature Property setters/getters
    window.set_nozzle_temp_actual(215.5);
    window.set_nozzle_temp_target(220.0);
    window.set_bed_temp_actual(60.2);
    window.set_bed_temp_target(65.0);

    assert_eq!(window.get_nozzle_temp_actual(), 215.5);
    assert_eq!(window.get_nozzle_temp_target(), 220.0);
    assert_eq!(window.get_bed_temp_actual(), 60.2);
    assert_eq!(window.get_bed_temp_target(), 65.0);

    // 3. Test Print Progress & Current Print File
    window.set_print_progress(0.45);
    window.set_current_print_file("test_benchy.gcode".into());

    assert_eq!(window.get_print_progress(), 0.45);
    assert_eq!(window.get_current_print_file(), "test_benchy.gcode");

    // 4. Test Console Log Output Model
    let initial_console: Vec<ConsoleLine> = vec![
        ConsoleLine { text: "Init r_klipp".into(), is_input: false },
        ConsoleLine { text: "MKS SKIPR connected".into(), is_input: false },
    ];
    window.set_console_lines(Rc::new(slint::VecModel::from(initial_console)).into());
    assert_eq!(window.get_console_lines().row_count(), 2);

    // 5. Test G-Code File List Model
    let files: Vec<GCodeFileInfo> = vec![
        GCodeFileInfo {
            name: "speed_test.gcode".into(),
            size: "3.2 MB".into(),
            is_dir: false,
            thumbnail: Default::default(),
        },
    ];
    window.set_gcode_files(Rc::new(slint::VecModel::from(files)).into());
    assert_eq!(window.get_gcode_files().row_count(), 1);

    // 6. Test UI Callbacks registration
    window.on_set_nozzle_target(|_temp| {});
    window.on_set_bed_target(|_temp| {});
    window.on_start_print(|_file| {});
    window.on_refresh_files(|| {});
    window.on_send_gcode_command(|_cmd| {});
}
