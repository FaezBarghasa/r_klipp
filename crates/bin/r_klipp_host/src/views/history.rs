use dioxus::prelude::*;
use crate::ui::*;

#[derive(Clone, PartialEq, Debug)]
pub struct HistoryRecord {
    pub id: u64,
    pub filename: String,
    pub date: String,
    pub duration: String,
    pub filament_used_m: f32,
    pub status: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct MaintenanceTask {
    pub name: String,
    pub current_hours: f32,
    pub max_hours: f32,
    pub icon: String,
}

#[component]
pub fn HistoryView() -> Element {
    let records = use_signal(|| vec![
        HistoryRecord { id: 101, filename: "Voron_Cube_V2.gcode".to_string(), date: "2026-08-26 14:10".to_string(), duration: "48m 20s".to_string(), filament_used_m: 12.8, status: "Completed".to_string() },
        HistoryRecord { id: 102, filename: "Benchy_HighSpeed.gcode".to_string(), date: "2026-08-25 18:22".to_string(), duration: "32m 10s".to_string(), filament_used_m: 18.2, status: "Completed".to_string() },
        HistoryRecord { id: 103, filename: "Fan_Duct_5015.gcode".to_string(), date: "2026-08-24 11:05".to_string(), duration: "1h 14m".to_string(), filament_used_m: 24.5, status: "Completed".to_string() },
        HistoryRecord { id: 104, filename: "Failed_Spool_Runout.gcode".to_string(), date: "2026-08-23 09:40".to_string(), duration: "2h 45m".to_string(), filament_used_m: 54.1, status: "Interrupted".to_string() },
    ]);

    let maintenance = use_signal(|| vec![
        MaintenanceTask { name: "Linear Rails Lubrication".to_string(), current_hours: 84.0, max_hours: 100.0, icon: "🛢️".to_string() },
        MaintenanceTask { name: "Check Gates & Belt Tension".to_string(), current_hours: 140.0, max_hours: 250.0, icon: "📏".to_string() },
        MaintenanceTask { name: "Clean Extruder Drive Gears".to_string(), current_hours: 45.0, max_hours: 150.0, icon: "⚙️".to_string() },
    ]);

    rsx! {
        div {
            class: "space-y-6",

            // Stats Overview
            div {
                class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4",
                div {
                    class: "bg-slate-900/80 p-4 rounded-xl border border-slate-800 space-y-1 shadow",
                    p { class: "text-xs text-slate-400 font-semibold uppercase", "Total Prints" }
                    p { class: "text-2xl font-extrabold text-blue-400", "42" }
                }
                div {
                    class: "bg-slate-900/80 p-4 rounded-xl border border-slate-800 space-y-1 shadow",
                    p { class: "text-xs text-slate-400 font-semibold uppercase", "Success Rate" }
                    p { class: "text-2xl font-extrabold text-emerald-400", "95.2%" }
                }
                div {
                    class: "bg-slate-900/80 p-4 rounded-xl border border-slate-800 space-y-1 shadow",
                    p { class: "text-xs text-slate-400 font-semibold uppercase", "Print Time" }
                    p { class: "text-2xl font-extrabold text-slate-100", "128h 40m" }
                }
                div {
                    class: "bg-slate-900/80 p-4 rounded-xl border border-slate-800 space-y-1 shadow",
                    p { class: "text-xs text-slate-400 font-semibold uppercase", "Filament Consumed" }
                    p { class: "text-2xl font-extrabold text-cyan-400", "1.84 kg" }
                }
            }

            // Maintenance Reminders Card
            VCard {
                title: "Hardware Maintenance Reminders".to_string(),
                subtitle: "Automated runtime interval tracking for 3D printer mechanics".to_string(),
                icon: "🔧".to_string(),
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    for task in maintenance.read().iter() {
                        div {
                            key: "{task.name}",
                            class: "p-4 bg-slate-800/40 rounded-xl border border-slate-700/60 space-y-2.5",
                            div {
                                class: "flex items-center space-x-2",
                                span { class: "text-lg", "{task.icon}" }
                                h4 { class: "font-semibold text-slate-200 text-sm", "{task.name}" }
                            }
                            div {
                                class: "space-y-1",
                                div {
                                    class: "flex justify-between text-xs text-slate-400 font-mono",
                                    span { "Interval" }
                                    span { "{task.current_hours:.0}h / {task.max_hours:.0}h" }
                                }
                                div {
                                    class: "w-full bg-slate-900 rounded-full h-2 overflow-hidden",
                                    div {
                                        class: if task.current_hours / task.max_hours > 0.8 {
                                            "bg-amber-500 h-full rounded-full"
                                        } else {
                                            "bg-blue-500 h-full rounded-full"
                                        },
                                        style: "width: {(task.current_hours / task.max_hours) * 100.0}%",
                                    }
                                }
                            }
                            VBtn {
                                label: "Mark Completed".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                class: "w-full mt-2".to_string(),
                                onclick: move |_| {},
                            }
                        }
                    }
                }
            }

            // Historical Records Table
            VCard {
                title: "Print History Logs".to_string(),
                icon: "📜".to_string(),
                div {
                    class: "space-y-2",
                    for record in records.read().iter() {
                        div {
                            key: "{record.id}",
                            class: "flex flex-col sm:flex-row justify-between items-start sm:items-center p-3.5 bg-slate-800/30 hover:bg-slate-800/60 rounded-lg border border-slate-800 transition-all gap-2",
                            div {
                                class: "space-y-1",
                                p { class: "font-semibold text-slate-100 text-sm", "{record.filename}" }
                                p { class: "text-xs text-slate-400 font-mono", "Date: {record.date} | Duration: {record.duration} | Filament: {record.filament_used_m:.1}m" }
                            }
                            VBadge {
                                text: record.status.clone(),
                                variant: if record.status == "Completed" { "success".to_string() } else { "danger".to_string() },
                            }
                        }
                    }
                }
            }
        }
    }
}
