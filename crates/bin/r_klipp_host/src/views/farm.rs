use dioxus::prelude::*;
use crate::state::*;
use crate::ui::*;

#[component]
pub fn FarmView() -> Element {
    let printers = use_signal(|| vec![
        FarmPrinter {
            id: "p1".to_string(),
            name: "Voron 2.4 (350mm)".to_string(),
            ip_address: "192.168.1.101".to_string(),
            status: "printing".to_string(),
            progress: 68.4,
            current_file: "Voron_Cube_V2.gcode".to_string(),
            nozzle_temp: 215.0,
            bed_temp: 60.0,
        },
        FarmPrinter {
            id: "p2".to_string(),
            name: "RatRig V-Core 3".to_string(),
            ip_address: "192.168.1.102".to_string(),
            status: "printing".to_string(),
            progress: 89.2,
            current_file: "Dragon_Articulated.gcode".to_string(),
            nozzle_temp: 240.0,
            bed_temp: 80.0,
        },
        FarmPrinter {
            id: "p3".to_string(),
            name: "Voron V0.2 (Mini)".to_string(),
            ip_address: "192.168.1.103".to_string(),
            status: "standby".to_string(),
            progress: 100.0,
            current_file: "None".to_string(),
            nozzle_temp: 24.0,
            bed_temp: 23.0,
        },
    ]);

    rsx! {
        div {
            class: "space-y-6",

            // Farm Header
            div {
                class: "flex flex-wrap items-center justify-between gap-4 bg-slate-900/90 backdrop-blur-md p-4 rounded-xl border border-slate-800",
                div {
                    class: "flex items-center space-x-3",
                    span { class: "text-2xl", "🏭" }
                    div {
                        h2 { class: "text-lg font-bold text-slate-100", "Multi-Printer Farm Mode" }
                        p { class: "text-xs text-slate-400 font-mono", "Managing {printers.read().len()} active printers on local network" }
                    }

                }
                div {
                    class: "flex space-x-3",
                    VBtn {
                        label: "Add Printer".to_string(),
                        icon: "➕".to_string(),
                        variant: "primary".to_string(),
                        onclick: move |_| {},
                    }
                    VBtn {
                        label: "Emergency Stop All".to_string(),
                        icon: "🛑".to_string(),
                        variant: "danger".to_string(),
                        onclick: move |_| {},
                    }
                }
            }

            // Printers Grid
            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for printer in printers.read().iter() {
                    div {
                        key: "{printer.id}",
                        class: "bg-slate-900/90 rounded-xl p-5 border border-slate-800 shadow-lg space-y-4 hover:border-slate-700 transition-all",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                h3 { class: "font-bold text-slate-100 text-base", "{printer.name}" }
                                p { class: "text-xs text-slate-400 font-mono", "{printer.ip_address}" }
                            }
                            VBadge {
                                text: printer.status.to_uppercase(),
                                variant: if printer.status == "printing" { "success".to_string() } else { "neutral".to_string() },
                            }
                        }

                        // Progress Bar
                        div {
                            class: "space-y-1.5",
                            div {
                                class: "flex justify-between text-xs text-slate-300 font-mono",
                                span { class: "truncate max-w-[180px]", "{printer.current_file}" }
                                span { class: "text-blue-400 font-bold", "{printer.progress:.1}%" }
                            }
                            div {
                                class: "w-full bg-slate-800 rounded-full h-2.5 overflow-hidden",
                                div {
                                    class: "bg-blue-500 h-full rounded-full transition-all duration-300",
                                    style: "width: {printer.progress}%",
                                }
                            }
                        }

                        // Quick telemetry stats
                        div {
                            class: "flex justify-between text-xs text-slate-400 font-mono bg-slate-800/40 p-2.5 rounded-lg",
                            span { "🔥 Nozzle: {printer.nozzle_temp}°C" }
                            span { "🛏️ Bed: {printer.bed_temp}°C" }
                        }

                        // Actions
                        div {
                            class: "flex space-x-2 pt-2",
                            VBtn {
                                label: "Open UI".to_string(),
                                variant: "primary".to_string(),
                                size: "sm".to_string(),
                                class: "flex-1".to_string(),
                                onclick: move |_| {},
                            }
                            VBtn {
                                label: "Pause".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| {},
                            }
                        }
                    }
                }
            }
        }
    }
}
