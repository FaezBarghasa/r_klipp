use dioxus::prelude::*;
use crate::state::*;
use crate::ui::*;

#[derive(Props, Clone, PartialEq)]
pub struct DashboardProps {
    pub nozzle: HeaterState,
    pub bed: HeaterState,
    pub toolhead: ToolheadState,
    pub print_job: PrintJobState,
    pub on_gcode: EventHandler<String>,
}

#[component]
pub fn DashboardView(props: DashboardProps) -> Element {
    let mut selected_jog_dist = use_signal(|| 10.0f32);
    let mut selected_extrude_dist = use_signal(|| 10.0f32);
    let mut custom_nozzle_target = use_signal(|| 215.0f32);
    let mut custom_bed_target = use_signal(|| 60.0f32);

    let progress_pct = props.print_job.progress;
    let format_time = |secs: u64| -> String {
        let hrs = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hrs, mins)
    };

    rsx! {
        div {
            class: "space-y-6",

            // Top Summary Bar / Status Card
            VCard {
                title: "Active Print Job".to_string(),
                subtitle: format!("File: {}", props.print_job.filename),
                icon: "🖨️".to_string(),
                class: "border-l-4 border-l-blue-500",
                div {
                    class: "grid grid-cols-1 md:grid-cols-4 gap-4 items-center",
                    // Progress ring/bar
                    div {
                        class: "md:col-span-2 space-y-2",
                        div {
                            class: "flex justify-between items-center text-sm font-semibold text-slate-300",
                            span { "Overall Progress" }
                            span { class: "text-blue-400 text-base font-bold", "{progress_pct:.1}%" }
                        }
                        div {
                            class: "w-full bg-slate-800 rounded-full h-3.5 overflow-hidden p-0.5 border border-slate-700",
                            div {
                                class: "bg-gradient-to-r from-blue-600 via-indigo-500 to-cyan-400 h-full rounded-full transition-all duration-300",
                                style: "width: {progress_pct}%",
                            }
                        }
                        div {
                            class: "flex justify-between text-xs text-slate-400 pt-1",
                            span { "Layer: {props.print_job.current_layer} / {props.print_job.total_layers}" }
                            span { "Filament Used: {props.print_job.filament_used_m:.1}m" }
                        }
                    }

                    // Print Timers
                    div {
                        class: "flex space-x-6 justify-around bg-slate-800/40 p-3 rounded-lg border border-slate-800",
                        div {
                            p { class: "text-xs text-slate-400", "Elapsed" }
                            p { class: "text-sm font-bold text-slate-200", "{format_time(props.print_job.print_time)}" }
                        }
                        div {
                            p { class: "text-xs text-slate-400", "Remaining" }
                            p { class: "text-sm font-bold text-blue-400", "{format_time(props.print_job.print_time_left)}" }
                        }
                    }

                    // Quick Action Buttons
                    div {
                        class: "flex items-center space-x-2 justify-end",
                        if props.print_job.status == "printing" {
                            VBtn {
                                label: "Pause".to_string(),
                                icon: "⏸".to_string(),
                                variant: "warning".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| props.on_gcode.call("PAUSE".to_string()),
                            }
                        } else {
                            VBtn {
                                label: "Resume".to_string(),
                                icon: "▶".to_string(),
                                variant: "success".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| props.on_gcode.call("RESUME".to_string()),
                            }
                        }
                        VBtn {
                            label: "Cancel".to_string(),
                            icon: "⏹".to_string(),
                            variant: "danger".to_string(),
                            size: "sm".to_string(),
                            onclick: move |_| props.on_gcode.call("CANCEL_PRINT".to_string()),
                        }
                    }
                }
            }

            // Main Grid Layout: Temperature, Controls, Macros
            div {
                class: "grid grid-cols-1 lg:grid-cols-3 gap-6",

                // Column 1 & 2: Temperatures and Motion Controls
                div {
                    class: "lg:col-span-2 space-y-6",

                    // Temperature Panel
                    VCard {
                        title: "Thermals & Heaters".to_string(),
                        icon: "🔥".to_string(),
                        div {
                            class: "space-y-6",
                            // Sparkline Visualization Placeholder / SVG Chart
                            div {
                                class: "h-36 w-full bg-slate-950/60 rounded-lg border border-slate-800/80 p-2 flex flex-col justify-between relative overflow-hidden",
                                svg {
                                    class: "w-full h-24 stroke-blue-500 fill-none stroke-2",
                                    view_box: "0 0 400 100",
                                    path {
                                        d: "M 0 80 Q 50 60 100 40 T 200 30 T 300 28 T 400 27",
                                    }
                                    path {
                                        class: "stroke-orange-500",
                                        d: "M 0 90 Q 70 80 150 50 T 300 45 T 400 45",
                                    }
                                }
                                div {
                                    class: "flex justify-between text-xs text-slate-400 font-mono px-2",
                                    span { "Extruder: {props.nozzle.current:.1}°C / {props.nozzle.target:.1}°C" }
                                    span { "Bed: {props.bed.current:.1}°C / {props.bed.target:.1}°C" }
                                }
                            }

                            // Temperature Targets & Presets
                            div {
                                class: "grid grid-cols-1 sm:grid-cols-2 gap-4",

                                // Extruder Target Card
                                div {
                                    class: "p-3 bg-slate-800/50 rounded-lg border border-slate-700/60 space-y-3",
                                    div {
                                        class: "flex justify-between items-center",
                                        span { class: "font-semibold text-slate-200 text-sm", "Nozzle Extruder" }
                                        VBadge { text: format!("{:.1}°C", props.nozzle.current), variant: "danger".to_string() }
                                    }
                                    div {
                                        class: "flex items-center space-x-2",
                                        input {
                                            r#type: "number",
                                            class: "w-24 bg-slate-900 border border-slate-700 text-slate-100 px-2.5 py-1.5 rounded text-sm font-mono focus:border-blue-500 focus:outline-none",
                                            value: "{custom_nozzle_target}",
                                            oninput: move |e| {
                                                if let Ok(v) = e.value().parse::<f32>() {
                                                    custom_nozzle_target.set(v);
                                                }
                                            },
                                        }
                                        VBtn {
                                            label: "Set".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| {
                                                let target = *custom_nozzle_target.read();
                                                props.on_gcode.call(format!("M104 S{}", target));
                                            },
                                        }
                                        VBtn {
                                            label: "Off".to_string(),
                                            variant: "secondary".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M104 S0".to_string()),
                                        }
                                    }
                                    // Presets
                                    div {
                                        class: "flex space-x-1.5 pt-1",
                                        VBtn {
                                            label: "PLA (210)".to_string(),
                                            variant: "ghost".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M104 S210".to_string()),
                                        }
                                        VBtn {
                                            label: "PETG (240)".to_string(),
                                            variant: "ghost".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M104 S240".to_string()),
                                        }
                                        VBtn {
                                            label: "ABS (255)".to_string(),
                                            variant: "ghost".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M104 S255".to_string()),
                                        }
                                    }
                                }

                                // Bed Target Card
                                div {
                                    class: "p-3 bg-slate-800/50 rounded-lg border border-slate-700/60 space-y-3",
                                    div {
                                        class: "flex justify-between items-center",
                                        span { class: "font-semibold text-slate-200 text-sm", "Heated Bed" }
                                        VBadge { text: format!("{:.1}°C", props.bed.current), variant: "warning".to_string() }
                                    }
                                    div {
                                        class: "flex items-center space-x-2",
                                        input {
                                            r#type: "number",
                                            class: "w-24 bg-slate-900 border border-slate-700 text-slate-100 px-2.5 py-1.5 rounded text-sm font-mono focus:border-blue-500 focus:outline-none",
                                            value: "{custom_bed_target}",
                                            oninput: move |e| {
                                                if let Ok(v) = e.value().parse::<f32>() {
                                                    custom_bed_target.set(v);
                                                }
                                            },
                                        }
                                        VBtn {
                                            label: "Set".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| {
                                                let target = *custom_bed_target.read();
                                                props.on_gcode.call(format!("M140 S{}", target));
                                            },
                                        }
                                        VBtn {
                                            label: "Off".to_string(),
                                            variant: "secondary".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M140 S0".to_string()),
                                        }
                                    }
                                    // Presets
                                    div {
                                        class: "flex space-x-1.5 pt-1",
                                        VBtn {
                                            label: "PLA (60)".to_string(),
                                            variant: "ghost".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M140 S60".to_string()),
                                        }
                                        VBtn {
                                            label: "PETG (80)".to_string(),
                                            variant: "ghost".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M140 S80".to_string()),
                                        }
                                        VBtn {
                                            label: "ABS (100)".to_string(),
                                            variant: "ghost".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call("M140 S100".to_string()),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Toolhead & Jog Control Panel
                    VCard {
                        title: "Toolhead & Motion".to_string(),
                        icon: "🧭".to_string(),
                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-6",

                            // Jog Pad
                            div {
                                class: "space-y-3 flex flex-col items-center",
                                div {
                                    class: "flex space-x-2 text-xs font-semibold mb-2",
                                    for &d in &[0.1f32, 1.0, 10.0, 50.0, 100.0] {
                                        button {
                                            class: if *selected_jog_dist.read() == d {
                                                "px-2.5 py-1 rounded bg-blue-600 text-white font-mono"
                                            } else {
                                                "px-2.5 py-1 rounded bg-slate-800 text-slate-300 font-mono hover:bg-slate-700"
                                            },
                                            onclick: move |_| selected_jog_dist.set(d),
                                            "{d}mm"
                                        }
                                    }
                                }

                                // 3x3 Jog Grid for X/Y
                                div {
                                    class: "grid grid-cols-3 gap-2 w-48 h-48",
                                    VBtn { label: "↖".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 X-{} Y{} F6000\nG90", *selected_jog_dist.read(), *selected_jog_dist.read())) }
                                    VBtn { label: "Y+".to_string(), variant: "primary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 Y{} F6000\nG90", *selected_jog_dist.read())) }
                                    VBtn { label: "↗".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 X{} Y{} F6000\nG90", *selected_jog_dist.read(), *selected_jog_dist.read())) }
                                    VBtn { label: "X-".to_string(), variant: "primary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 X-{} F6000\nG90", *selected_jog_dist.read())) }
                                    VBtn { label: "🏠 XY".to_string(), variant: "ghost".to_string(), onclick: move |_| props.on_gcode.call("G28 X Y".to_string()) }
                                    VBtn { label: "X+".to_string(), variant: "primary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 X{} F6000\nG90", *selected_jog_dist.read())) }
                                    VBtn { label: "↙".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 X-{} Y-{} F6000\nG90", *selected_jog_dist.read(), *selected_jog_dist.read())) }
                                    VBtn { label: "Y-".to_string(), variant: "primary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 Y-{} F6000\nG90", *selected_jog_dist.read())) }
                                    VBtn { label: "↘".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call(format!("G91\nG1 X{} Y-{} F6000\nG90", *selected_jog_dist.read(), *selected_jog_dist.read())) }
                                }
                            }

                            // Z Controls & Extruder
                            div {
                                class: "space-y-4 flex flex-col justify-between",
                                div {
                                    class: "space-y-2",
                                    span { class: "text-xs font-semibold text-slate-400 uppercase tracking-wider", "Z-Axis Controls" }
                                    div {
                                        class: "grid grid-cols-2 gap-2",
                                        VBtn { label: "Z +10".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call("G91\nG1 Z10 F600\nG90".to_string()) }
                                        VBtn { label: "Z +1".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call("G91\nG1 Z1 F600\nG90".to_string()) }
                                        VBtn { label: "Z -1".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call("G91\nG1 Z-1 F600\nG90".to_string()) }
                                        VBtn { label: "Z -10".to_string(), variant: "secondary".to_string(), onclick: move |_| props.on_gcode.call("G91\nG1 Z-10 F600\nG90".to_string()) }
                                    }
                                    VBtn {
                                        label: "Home All (G28)".to_string(),
                                        icon: "🏠".to_string(),
                                        variant: "primary".to_string(),
                                        class: "w-full mt-2".to_string(),
                                        onclick: move |_| props.on_gcode.call("G28".to_string()),
                                    }
                                }

                                // Extrusion controls
                                div {
                                    class: "p-3 bg-slate-800/40 rounded-lg border border-slate-700/50 space-y-2",
                                    span { class: "text-xs font-semibold text-slate-400 uppercase tracking-wider", "Extrude / Retract" }
                                    div {
                                        class: "flex space-x-2",
                                        VBtn {
                                            label: format!("Extrude {}mm", *selected_extrude_dist.read()),
                                            variant: "success".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call(format!("M83\nG1 E{} F300", *selected_extrude_dist.read())),
                                        }
                                        VBtn {
                                            label: format!("Retract {}mm", *selected_extrude_dist.read()),
                                            variant: "warning".to_string(),
                                            size: "sm".to_string(),
                                            onclick: move |_| props.on_gcode.call(format!("M83\nG1 E-{} F300", *selected_extrude_dist.read())),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Column 3: Macros & System Actions
                div {
                    class: "space-y-6",

                    // G-code Macros Panel
                    VCard {
                        title: "Macros & Commands".to_string(),
                        icon: "⚡".to_string(),
                        div {
                            class: "space-y-3",
                            div {
                                class: "grid grid-cols-1 gap-2",
                                VBtn {
                                    label: "BED_MESH_CALIBRATE".to_string(),
                                    variant: "secondary".to_string(),
                                    class: "justify-start text-xs font-mono".to_string(),
                                    onclick: move |_| props.on_gcode.call("BED_MESH_CALIBRATE".to_string()),
                                }
                                VBtn {
                                    label: "LOAD_FILAMENT".to_string(),
                                    variant: "secondary".to_string(),
                                    class: "justify-start text-xs font-mono".to_string(),
                                    onclick: move |_| props.on_gcode.call("LOAD_FILAMENT".to_string()),
                                }
                                VBtn {
                                    label: "UNLOAD_FILAMENT".to_string(),
                                    variant: "secondary".to_string(),
                                    class: "justify-start text-xs font-mono".to_string(),
                                    onclick: move |_| props.on_gcode.call("UNLOAD_FILAMENT".to_string()),
                                }
                                VBtn {
                                    label: "CLEAN_NOZZLE".to_string(),
                                    variant: "secondary".to_string(),
                                    class: "justify-start text-xs font-mono".to_string(),
                                    onclick: move |_| props.on_gcode.call("CLEAN_NOZZLE".to_string()),
                                }
                                VBtn {
                                    label: "QUAD_GANTRY_LEVEL".to_string(),
                                    variant: "secondary".to_string(),
                                    class: "justify-start text-xs font-mono".to_string(),
                                    onclick: move |_| props.on_gcode.call("QUAD_GANTRY_LEVEL".to_string()),
                                }
                            }
                        }
                    }

                    // Emergency & Power Guard
                    VCard {
                        title: "Emergency Stop".to_string(),
                        icon: "🛑".to_string(),
                        class: "border-red-900/50 bg-red-950/20".to_string(),
                        div {
                            class: "space-y-3 text-center",
                            p { class: "text-xs text-red-300", "Instantly halts all motor movements and cuts heater power immediately." }
                            VBtn {
                                label: "EMERGENCY STOP (M112)".to_string(),
                                variant: "danger".to_string(),
                                class: "w-full py-3 font-bold".to_string(),
                                onclick: move |_| props.on_gcode.call("M112".to_string()),
                            }
                            VBtn {
                                label: "Disable Steppers (M84)".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                class: "w-full".to_string(),
                                onclick: move |_| props.on_gcode.call("M84".to_string()),
                            }
                        }
                    }
                }
            }
        }
    }
}
