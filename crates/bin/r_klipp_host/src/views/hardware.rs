use dioxus::prelude::*;
use crate::state::*;
use crate::ui::*;

#[derive(Props, Clone, PartialEq)]
pub struct HardwareProps {
    pub on_gcode: EventHandler<String>,
}

#[component]
pub fn HardwareView(props: HardwareProps) -> Element {
    let mut spools = use_signal(|| vec![
        SpoolInfo { id: 1, name: "Prusament Galaxy Black".to_string(), material: "PLA".to_string(), color_hex: "#111827".to_string(), remaining_weight_g: 680.0, total_weight_g: 1000.0, temperature_nozzle: 215, temperature_bed: 60 },
        SpoolInfo { id: 2, name: "Polymaker PolyLite Teal".to_string(), material: "PETG".to_string(), color_hex: "#0d9488".to_string(), remaining_weight_g: 420.0, total_weight_g: 1000.0, temperature_nozzle: 240, temperature_bed: 80 },
        SpoolInfo { id: 3, name: "eSUN Fire Red".to_string(), material: "ABS+".to_string(), color_hex: "#dc2626".to_string(), remaining_weight_g: 910.0, total_weight_g: 1000.0, temperature_nozzle: 255, temperature_bed: 100 },
    ]);

    let mut mmu_gates = use_signal(|| vec![
        MmuGate { gate_id: 1, filament_name: "Prusament Galaxy Black".to_string(), color_hex: "#111827".to_string(), is_loaded: true, status: "In Toolhead".to_string() },
        MmuGate { gate_id: 2, filament_name: "Polymaker Teal".to_string(), color_hex: "#0d9488".to_string(), is_loaded: false, status: "Pre-loaded".to_string() },
        MmuGate { gate_id: 3, filament_name: "eSUN Red".to_string(), color_hex: "#dc2626".to_string(), is_loaded: false, status: "Pre-loaded".to_string() },
        MmuGate { gate_id: 4, filament_name: "Empty Gate".to_string(), color_hex: "#64748b".to_string(), is_loaded: false, status: "Empty".to_string() },
    ]);

    let afc_lanes = use_signal(|| vec![
        AfcLane { lane_id: 1, spool_name: "Spool #1 (Black)".to_string(), material: "PLA".to_string(), color_hex: "#111827".to_string(), status: "Hub Active".to_string(), buffer_length_mm: 120.0 },
        AfcLane { lane_id: 2, spool_name: "Spool #2 (Teal)".to_string(), material: "PETG".to_string(), color_hex: "#0d9488".to_string(), status: "Ready".to_string(), buffer_length_mm: 140.0 },
        AfcLane { lane_id: 3, spool_name: "Spool #3 (Red)".to_string(), material: "ABS+".to_string(), color_hex: "#dc2626".to_string(), status: "Ready".to_string(), buffer_length_mm: 135.0 },
        AfcLane { lane_id: 4, spool_name: "Spool #4 (White)".to_string(), material: "PLA".to_string(), color_hex: "#f8fafc".to_string(), status: "Empty".to_string(), buffer_length_mm: 0.0 },
    ]);

    rsx! {
        div {
            class: "space-y-6",

            // Spoolman Integration Header
            VCard {
                title: "Spoolman Filament Inventory".to_string(),
                subtitle: "Active spool telemetry and weight tracking".to_string(),
                icon: "🧵".to_string(),
                div {
                    class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    for spool in spools.read().iter() {
                        div {
                            key: "{spool.id}",
                            class: "p-4 bg-slate-800/50 rounded-xl border border-slate-700/60 space-y-3",
                            div {
                                class: "flex items-center justify-between",
                                div {
                                    class: "flex items-center space-x-2.5",
                                    span {
                                        class: "w-4 h-4 rounded-full border border-slate-600 inline-block shadow",
                                        style: "background-color: {spool.color_hex}",
                                    }
                                    h4 { class: "font-semibold text-slate-100 text-sm", "{spool.name}" }
                                }
                                VBadge { text: spool.material.clone(), variant: "info".to_string() }
                            }
                            // Weight remaining bar
                            div {
                                class: "space-y-1",
                                div {
                                    class: "flex justify-between text-xs text-slate-400 font-mono",
                                    span { "Remaining" }
                                    span { "{spool.remaining_weight_g:.0}g / {spool.total_weight_g:.0}g" }
                                }
                                div {
                                    class: "w-full bg-slate-900 rounded-full h-2 overflow-hidden",
                                    div {
                                        class: "bg-blue-500 h-full rounded-full",
                                        style: "width: {(spool.remaining_weight_g / spool.total_weight_g) * 100.0}%",
                                    }
                                }
                            }
                            div {
                                class: "flex justify-between text-xs text-slate-400 pt-1 font-mono",
                                span { "Nozzle: {spool.temperature_nozzle}°C" }
                                span { "Bed: {spool.temperature_bed}°C" }
                            }
                            VBtn {
                                label: "Select as Active Spool".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                class: "w-full mt-2".to_string(),
                                onclick: move |_| {},
                            }
                        }
                    }
                }
            }

            // MMU (Multi-Material Unit) State & Controls
            VCard {
                title: "MMU / Happy Hare Dashboard".to_string(),
                subtitle: "Multi-Material multi-gate selector and encoder telemetry".to_string(),
                icon: "🎛️".to_string(),
                div {
                    class: "space-y-4",
                    div {
                        class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4",
                        for gate in mmu_gates.read().iter() {
                            div {
                                key: "{gate.gate_id}",
                                class: if gate.is_loaded {
                                    "p-4 bg-blue-950/40 rounded-xl border-2 border-blue-500/80 space-y-3"
                                } else {
                                    "p-4 bg-slate-800/40 rounded-xl border border-slate-700/60 space-y-3"
                                },
                                div {
                                    class: "flex justify-between items-center",
                                    span { class: "font-bold text-slate-200 text-sm", "Gate #{gate.gate_id}" }
                                    VBadge {
                                        text: gate.status.clone(),
                                        variant: if gate.is_loaded { "success".to_string() } else { "neutral".to_string() },
                                    }
                                }
                                div {
                                    class: "flex items-center space-x-2 text-xs text-slate-300",
                                    span {
                                        class: "w-3 h-3 rounded-full inline-block",
                                        style: "background-color: {gate.color_hex}",
                                    }
                                    span { "{gate.filament_name}" }
                                }
                                div {
                                    class: "flex space-x-2 pt-2",
                                    {
                                        let gid = gate.gate_id;
                                        rsx! {
                                            VBtn {
                                                label: "Select".to_string(),
                                                variant: "primary".to_string(),
                                                size: "sm".to_string(),
                                                class: "flex-1".to_string(),
                                                onclick: move |_| props.on_gcode.call(format!("MMU_SELECT GATE={}", gid)),
                                            }
                                        }
                                    }
                                    {
                                        let gid = gate.gate_id;
                                        rsx! {
                                            VBtn {
                                                label: "Eject".to_string(),
                                                variant: "secondary".to_string(),
                                                size: "sm".to_string(),
                                                onclick: move |_| props.on_gcode.call(format!("MMU_EJECT GATE={}", gid)),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // AFC (Automated Filament Changer) Panel
            VCard {
                title: "AFC (Automated Filament Changer)".to_string(),
                subtitle: "Hub extruder mapping and lane tension status".to_string(),
                icon: "🔄".to_string(),
                div {
                    class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                    for lane in afc_lanes.read().iter() {
                        div {
                            key: "{lane.lane_id}",
                            class: "p-4 bg-slate-800/40 rounded-xl border border-slate-700/60 space-y-2.5",
                            div {
                                class: "flex justify-between items-center",
                                span { class: "font-semibold text-slate-200 text-sm", "Lane #{lane.lane_id}" }
                                VBadge {
                                    text: lane.status.clone(),
                                    variant: if lane.status == "Hub Active" { "success".to_string() } else { "neutral".to_string() },
                                }
                            }
                            p { class: "text-xs text-slate-400 font-mono", "{lane.spool_name} ({lane.material})" }
                            div {
                                class: "text-xs text-slate-400 font-mono flex justify-between",
                                span { "Buffer Length:" }
                                span { class: "text-blue-400 font-bold", "{lane.buffer_length_mm:.1} mm" }
                            }
                        }
                    }
                }
            }
        }
    }
}
