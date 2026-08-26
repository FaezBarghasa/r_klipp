use dioxus::prelude::*;
use crate::ui::*;

#[derive(Clone, PartialEq, Debug)]
pub struct GCodeItem {
    pub name: String,
    pub size_mb: f32,
    pub estimated_time: String,
    pub filament_m: f32,
    pub layer_height: f32,
    pub modified_date: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct FilesViewProps {
    pub on_print: EventHandler<String>,
}

#[component]
pub fn FilesView(props: FilesViewProps) -> Element {
    let mut files = use_signal(|| vec![
        GCodeItem {
            name: "Voron_Cube_V2_PETG.gcode".to_string(),
            size_mb: 2.4,
            estimated_time: "48m 20s".to_string(),
            filament_m: 12.8,
            layer_height: 0.2,
            modified_date: "2026-08-26 18:30".to_string(),
        },
        GCodeItem {
            name: "Benchy_HighSpeed_PLA.gcode".to_string(),
            size_mb: 4.8,
            estimated_time: "32m 10s".to_string(),
            filament_m: 18.2,
            layer_height: 0.2,
            modified_date: "2026-08-26 15:12".to_string(),
        },
        GCodeItem {
            name: "StealthBurner_Front_Cover_ABS.gcode".to_string(),
            size_mb: 8.1,
            estimated_time: "2h 15m".to_string(),
            filament_m: 45.6,
            layer_height: 0.2,
            modified_date: "2026-08-25 21:04".to_string(),
        },
    ]);

    let mut queue = use_signal(|| vec![
        "Benchy_HighSpeed_PLA.gcode".to_string(),
        "StealthBurner_Front_Cover_ABS.gcode".to_string(),
    ]);

    rsx! {
        div {
            class: "space-y-6",

            // Upload Bar & Search
            div {
                class: "flex flex-wrap items-center justify-between gap-4 bg-slate-900/90 backdrop-blur-md p-4 rounded-xl border border-slate-800",
                div {
                    class: "flex items-center space-x-3",
                    span { class: "text-2xl", "📁" }
                    h2 { class: "text-lg font-bold text-slate-100", "G-Code File Browser & Job Queue" }
                }
                div {
                    class: "flex space-x-3",
                    VBtn {
                        label: "Upload File".to_string(),
                        icon: "📤".to_string(),
                        variant: "primary".to_string(),
                        onclick: move |_| {},
                    }
                    VBtn {
                        label: "New Directory".to_string(),
                        icon: "➕".to_string(),
                        variant: "secondary".to_string(),
                        onclick: move |_| {},
                    }
                }
            }

            // Two Columns: Files List and Print Queue
            div {
                class: "grid grid-cols-1 lg:grid-cols-3 gap-6",

                // G-Code Files List (2 cols)
                div {
                    class: "lg:col-span-2 space-y-4",
                    VCard {
                        title: "Available Print Files".to_string(),
                        subtitle: format!("{} items", files.read().len()),
                        icon: "📄".to_string(),
                        div {
                            class: "space-y-3",
                            for file in files.read().iter() {
                                div {
                                    key: "{file.name}",
                                    class: "flex flex-col sm:flex-row justify-between items-start sm:items-center p-3.5 bg-slate-800/40 hover:bg-slate-800/80 rounded-lg border border-slate-700/50 transition-all gap-3",
                                    div {
                                        class: "space-y-1",
                                        div {
                                            class: "flex items-center space-x-2",
                                            span { class: "text-blue-400 font-semibold text-sm", "{file.name}" }
                                            VBadge { text: format!("{:.1} MB", file.size_mb), variant: "neutral".to_string() }
                                        }
                                        div {
                                            class: "flex space-x-4 text-xs text-slate-400 font-mono",
                                            span { "Est: {file.estimated_time}" }
                                            span { "Filament: {file.filament_m:.1}m" }
                                            span { "Layer: {file.layer_height}mm" }
                                        }
                                    }

                                    // Action buttons
                                    div {
                                        class: "flex items-center space-x-2 self-end sm:self-center",
                                        {
                                            let fname = file.name.clone();
                                            rsx! {
                                                VBtn {
                                                    label: "Print".to_string(),
                                                    icon: "▶".to_string(),
                                                    variant: "success".to_string(),
                                                    size: "sm".to_string(),
                                                    onclick: move |_| props.on_print.call(fname.clone()),
                                                }
                                            }
                                        }
                                        {
                                            let fname = file.name.clone();
                                            rsx! {
                                                VBtn {
                                                    label: "+ Queue".to_string(),
                                                    variant: "secondary".to_string(),
                                                    size: "sm".to_string(),
                                                    onclick: move |_| queue.write().push(fname.clone()),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Print Job Queue (1 col)
                div {
                    class: "space-y-4",
                    VCard {
                        title: "Print Job Queue".to_string(),
                        subtitle: format!("{} jobs queued", queue.read().len()),
                        icon: "📋".to_string(),
                        div {
                            class: "space-y-3",
                            if queue.read().is_empty() {
                                p { class: "text-xs text-slate-500 italic text-center py-6", "Queue is empty. Add G-code files to queue next prints." }
                            } else {
                                for (idx, job) in queue.read().iter().enumerate() {
                                    div {
                                        key: "{idx}",
                                        class: "flex items-center justify-between p-3 bg-slate-800/60 rounded-lg border border-slate-700/60 text-xs font-mono",
                                        div {
                                            class: "flex items-center space-x-2",
                                            span { class: "w-5 h-5 rounded-full bg-blue-600/30 text-blue-400 flex items-center justify-center font-bold", "{idx + 1}" }
                                            span { class: "text-slate-200 truncate max-w-[140px]", "{job}" }
                                        }
                                        button {
                                            class: "text-slate-400 hover:text-red-400 p-1",
                                            onclick: move |_| {
                                                queue.write().remove(idx);
                                            },
                                            "✕"
                                        }
                                    }
                                }
                                VBtn {
                                    label: "Start Queue".to_string(),
                                    variant: "primary".to_string(),
                                    class: "w-full mt-2".to_string(),
                                    onclick: move |_| {},
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
