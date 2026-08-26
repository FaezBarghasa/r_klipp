use dioxus::prelude::*;
use crate::ui::*;

#[component]
pub fn ConfigureView() -> Element {
    let mut config_text = use_signal(|| r#"[include mainsail.cfg]
[include fluidd.cfg]

[mcu]
serial: /dev/serial/by-id/usb-Klipper_stm32f407_MKS_SKIPR-if00
restart_method: command

[printer]
kinematics: corexy
max_velocity: 350
max_accel: 5000
max_z_velocity: 25
max_z_accel: 350
square_corner_velocity: 5.0

[stepper_x]
step_pin: PA14
dir_pin: !PA13
enable_pin: !PA15
microsteps: 32
rotation_distance: 40
endstop_pin: tmc2209_stepper_x:virtual_endstop
position_endstop: 350
position_max: 350
homing_speed: 60

[extruder]
step_pin: PB4
dir_pin: PB3
enable_pin: !PB5
microsteps: 16
rotation_distance: 22.67895
nozzle_diameter: 0.400
filament_diameter: 1.750
heater_pin: PB11
sensor_type: ATC Semitec 104GT-2
sensor_pin: PA2
min_temp: 0
max_temp: 300
"#.to_string());

    rsx! {
        div {
            class: "space-y-6",

            VCard {
                title: "Klipper Configuration Editor (printer.cfg)".to_string(),
                subtitle: "Monaco-Editor wrapper with syntax verification and live backup".to_string(),
                icon: "📝".to_string(),
                div {
                    class: "space-y-4",

                    // Editor Top Actions
                    div {
                        class: "flex flex-wrap items-center justify-between gap-3 bg-slate-800/60 p-3 rounded-lg border border-slate-700/60",
                        div {
                            class: "flex items-center space-x-2 text-xs font-mono text-slate-300",
                            span { "Active File:" }
                            span { class: "text-blue-400 font-bold bg-slate-900 px-2.5 py-1 rounded border border-slate-700", "printer.cfg" }
                        }
                        div {
                            class: "flex space-x-2",
                            VBtn {
                                label: "Save & Restart".to_string(),
                                icon: "💾".to_string(),
                                variant: "primary".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| {},
                            }
                            VBtn {
                                label: "Create Backup Snapshot".to_string(),
                                icon: "📦".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| {},
                            }
                        }
                    }

                    // Configuration Code Box
                    textarea {
                        class: "w-full h-96 bg-slate-950 text-slate-100 font-mono text-xs p-4 rounded-xl border border-slate-800 focus:border-blue-500 focus:outline-none leading-relaxed",
                        value: "{config_text}",
                        oninput: move |e| config_text.set(e.value()),
                    }
                }
            }
        }
    }
}
