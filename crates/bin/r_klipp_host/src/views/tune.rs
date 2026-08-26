use dioxus::prelude::*;
use crate::ui::*;

#[derive(Props, Clone, PartialEq)]
pub struct TuneProps {
    pub on_gcode: EventHandler<String>,
}

#[component]
pub fn TuneView(props: TuneProps) -> Element {
    let mut pid_target_temp = use_signal(|| 210);
    let mut pressure_advance = use_signal(|| 0.045f32);
    let mut shaper_freq_x = use_signal(|| 54.2f32);
    let mut shaper_freq_y = use_signal(|| 48.6f32);

    rsx! {
        div {
            class: "space-y-6",

            // 3D Bed Mesh Heightmap Visualizer
            VCard {
                title: "Bed Mesh Heightmap (ECharts-GL)".to_string(),
                subtitle: "Bed topography mesh (7x7 probed grid)".to_string(),
                icon: "🌐".to_string(),
                div {
                    class: "space-y-4",
                    div {
                        class: "flex justify-between items-center bg-slate-800/50 p-3 rounded-lg border border-slate-700/50 text-xs font-mono",
                        div {
                            span { class: "text-slate-400", "Mesh Range: " }
                            span { class: "text-blue-400 font-bold", "-0.045mm to +0.062mm" }
                        }
                        div {
                            span { class: "text-slate-400", "Variance: " }
                            span { class: "text-emerald-400 font-bold", "0.107mm" }
                        }
                        VBtn {
                            label: "Calibrate Mesh (G29)".to_string(),
                            icon: "🔄".to_string(),
                            size: "sm".to_string(),
                            onclick: move |_| props.on_gcode.call("BED_MESH_CALIBRATE".to_string()),
                        }
                    }

                    // 3D Visual Mesh Map (Matrix grid representation)
                    div {
                        class: "h-64 w-full bg-slate-950 rounded-xl p-4 border border-slate-800 flex flex-col justify-center items-center relative overflow-hidden",
                        div {
                            class: "grid grid-cols-7 gap-1.5 w-full max-w-md",
                            for row in 0..7 {
                                for col in 0..7 {
                                    {
                                        let val = ((row as f32 - 3.0).powi(2) + (col as f32 - 3.0).powi(2)) * 0.005 - 0.02;
                                        let bg = if val > 0.02 {
                                            "bg-rose-500/80"
                                        } else if val > 0.0 {
                                            "bg-amber-500/80"
                                        } else if val > -0.02 {
                                            "bg-emerald-500/80"
                                        } else {
                                            "bg-blue-500/80"
                                        };
                                        rsx! {
                                            div {
                                                class: "h-6 rounded flex items-center justify-center text-[9px] font-mono text-slate-950 font-bold {bg} hover:scale-110 transition-transform cursor-pointer shadow",
                                                title: format!("X: {}, Y: {}, Z: {:.3}mm", col, row, val),
                                                "{val:.2}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "absolute bottom-2 right-4 flex space-x-2 text-[10px] text-slate-400",
                            span { "Low (-0.05)" }
                            div { class: "w-16 h-2 bg-gradient-to-r from-blue-500 via-emerald-500 via-amber-500 to-rose-500 rounded my-auto" }
                            span { "High (+0.05)" }
                        }
                    }
                }
            }

            // Calibration & Tuning Wizards: PID & Input Shaper
            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-6",

                // PID Tuning Wizard
                VCard {
                    title: "PID Auto-Tune Wizard".to_string(),
                    icon: "🌡️".to_string(),
                    div {
                        class: "space-y-4",
                        div {
                            class: "flex items-center space-x-3",
                            span { class: "text-xs font-semibold text-slate-300", "Target Temp (°C):" }
                            input {
                                r#type: "number",
                                class: "w-28 bg-slate-900 border border-slate-700 text-slate-100 px-3 py-1.5 rounded text-sm font-mono focus:border-blue-500 focus:outline-none",
                                value: "{pid_target_temp}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        pid_target_temp.set(v);
                                    }
                                },
                            }
                        }
                        div {
                            class: "flex space-x-2",
                            VBtn {
                                label: "Tune Extruder (PID_CALIBRATE)".to_string(),
                                variant: "primary".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| props.on_gcode.call(format!("PID_CALIBRATE HEATER=extruder TARGET={}", *pid_target_temp.read())),
                            }
                            VBtn {
                                label: "Tune Bed".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| props.on_gcode.call(format!("PID_CALIBRATE HEATER=heater_bed TARGET=60")),
                            }
                        }
                    }
                }

                // Input Shaper & Pressure Advance
                VCard {
                    title: "Input Shaper & Pressure Advance".to_string(),
                    icon: "📈".to_string(),
                    div {
                        class: "space-y-4",
                        div {
                            class: "space-y-1.5",
                            div {
                                class: "flex justify-between text-xs text-slate-300 font-mono",
                                span { "Pressure Advance (PA)" }
                                span { class: "text-blue-400 font-bold", "{pressure_advance:.3}" }
                            }
                            input {
                                r#type: "range",
                                min: "0.0",
                                max: "0.15",
                                step: "0.005",
                                class: "w-full accent-blue-500",
                                value: "{pressure_advance}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<f32>() {
                                        pressure_advance.set(v);
                                        props.on_gcode.call(format!("SET_PRESSURE_ADVANCE ADVANCE={:.3}", v));
                                    }
                                },
                            }
                        }

                        div {
                            class: "grid grid-cols-2 gap-3 text-xs font-mono text-slate-300 pt-2",
                            div {
                                span { "Shaper Freq X: " }
                                span { class: "text-cyan-400 font-bold", "{shaper_freq_x:.1} Hz" }
                            }
                            div {
                                span { "Shaper Freq Y: " }
                                span { class: "text-cyan-400 font-bold", "{shaper_freq_y:.1} Hz" }
                            }
                        }
                    }
                }
            }
        }
    }
}
