mod api;
mod components;
mod mcu_comms;
mod state;
mod ui;
mod views;

use dioxus::prelude::*;
use state::*;
use ui::*;
use views::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppTab {
    Dashboard,
    Files,
    Console,
    Hardware,
    Webcams,
    Tune,
    Farm,
    History,
    Configure,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut active_tab = use_signal(|| AppTab::Dashboard);
    let mut nozzle_state = use_signal(HeaterState::default);
    let mut bed_state = use_signal(|| HeaterState {
        name: "Heater Bed".to_string(),
        current: 60.2,
        target: 60.0,
        power: 0.22,
        history: vec![58.0, 59.0, 59.5, 60.0, 60.2],
    });
    let mut toolhead_state = use_signal(ToolheadState::default);
    let mut print_job = use_signal(PrintJobState::default);
    let mut last_command = use_signal(|| "Ready".to_string());

    // Live telemetry simulation / background update ticker
    use_future(move || async move {
        let mut count = 0u64;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            count += 1;

            // Small fluctuation in nozzle temp around target
            let noise = ((count % 5) as f32 - 2.0) * 0.15;
            let target = nozzle_state.read().target;
            let cur = (target + noise).max(20.0);
            nozzle_state.write().current = cur;

            // If printing, advance timers and progress
            if print_job.read().status == "printing" {
                let mut job = print_job.write();
                job.print_time += 1;
                if job.print_time_left > 1 {
                    job.print_time_left -= 1;
                }
                job.progress = ((job.print_time as f32) / (job.print_time + job.print_time_left) as f32) * 100.0;
            }
        }
    });

    let mut on_gcode = move |cmd: String| {
        last_command.set(format!("Executed: {}", cmd));
        if cmd.starts_with("M104 S") {
            if let Ok(temp) = cmd[6..].trim().parse::<f32>() {
                nozzle_state.write().target = temp;
            }
        } else if cmd.starts_with("M140 S") {
            if let Ok(temp) = cmd[6..].trim().parse::<f32>() {
                bed_state.write().target = temp;
            }
        } else if cmd == "PAUSE" {
            print_job.write().status = "paused".to_string();
        } else if cmd == "RESUME" {
            print_job.write().status = "printing".to_string();
        } else if cmd == "CANCEL_PRINT" {
            print_job.write().status = "standby".to_string();
        } else if cmd == "M112" {
            nozzle_state.write().target = 0.0;
            bed_state.write().target = 0.0;
            print_job.write().status = "error".to_string();
        }
    };


    rsx! {
        div {
            class: "min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-blue-500 selection:text-white",

            // Top Header Bar
            header {
                class: "sticky top-0 z-50 bg-slate-900/90 backdrop-blur-md border-b border-slate-800 px-6 py-3 flex items-center justify-between shadow-md",
                
                // Branding
                div {
                    class: "flex items-center space-x-3",
                    div {
                        class: "w-8 h-8 rounded-lg bg-gradient-to-tr from-blue-600 to-cyan-400 flex items-center justify-center font-black text-white text-lg shadow-lg shadow-blue-500/20",
                        "⚡"
                    }
                    div {
                        h1 { class: "text-lg font-black tracking-tight text-white flex items-center gap-1.5",
                            "FluiddSail"
                            span { class: "text-[10px] uppercase font-bold tracking-widest text-cyan-400 bg-cyan-950/80 px-1.5 py-0.5 rounded border border-cyan-800/60", "r_klipp" }
                        }
                        p { class: "text-[10px] text-slate-400 font-mono", "Unified High-Speed Host UI" }
                    }
                }

                // Center Telemetry Quick Status
                div {
                    class: "hidden md:flex items-center space-x-6 text-xs font-mono bg-slate-950/60 px-4 py-1.5 rounded-full border border-slate-800",
                    div {
                        span { class: "text-slate-400", "Nozzle: " }
                        span { class: "text-rose-400 font-bold", "{nozzle_state.read().current:.1}°C" }
                    }
                    div {
                        span { class: "text-slate-400", "Bed: " }
                        span { class: "text-amber-400 font-bold", "{bed_state.read().current:.1}°C" }
                    }
                    div {
                        span { class: "text-slate-400", "State: " }
                        span { class: "text-blue-400 font-bold uppercase", "{print_job.read().status}" }
                    }
                }

                // Right Actions
                div {
                    class: "flex items-center space-x-3",
                    VBadge { text: "Moonraker 7125 (Connected)".to_string(), variant: "success".to_string() }
                    VBtn {
                        label: "E-STOP".to_string(),
                        icon: "🛑".to_string(),
                        variant: "danger".to_string(),
                        size: "sm".to_string(),
                        onclick: move |_| on_gcode("M112".to_string()),
                    }
                }
            }

            // Main Layout Body: Sidebar Navigation + Content View
            div {
                class: "flex flex-1 overflow-hidden",

                // Sidebar
                aside {
                    class: "w-64 bg-slate-900/60 backdrop-blur-md border-r border-slate-800 p-4 space-y-1.5 hidden md:block",
                    div {
                        class: "text-[11px] font-bold text-slate-500 uppercase tracking-wider px-3 py-1",
                        "Navigation"
                    }
                    for (tab, icon, label) in &[
                        (AppTab::Dashboard, "📊", "Dashboard"),
                        (AppTab::Files, "📁", "Files & Queue"),
                        (AppTab::Console, "💻", "G-Code Console"),
                        (AppTab::Hardware, "🎛️", "Hardware (MMU/Spool)"),
                        (AppTab::Webcams, "📷", "Live Webcams"),
                        (AppTab::Tune, "📈", "Heightmap & Tune"),
                        (AppTab::Farm, "🏭", "Farm Mode"),
                        (AppTab::History, "📜", "Print History"),
                        (AppTab::Configure, "📝", "Configuration"),
                    ] {
                        {
                            let t = *tab;
                            let is_active = *active_tab.read() == t;
                            let active_cls = if is_active {
                                "bg-blue-600/20 text-blue-400 border border-blue-500/40 shadow-sm font-semibold"
                            } else {
                                "text-slate-300 hover:bg-slate-800/60 hover:text-slate-100 font-medium"
                            };
                            rsx! {
                                button {
                                    key: "{label}",
                                    class: "w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm transition-all text-left {active_cls}",
                                    onclick: move |_| active_tab.set(t),
                                    span { class: "text-lg", "{icon}" }
                                    span { "{label}" }
                                }
                            }
                        }
                    }
                }

                // Main Page View Container
                main {
                    class: "flex-1 overflow-y-auto p-6 bg-slate-950",
                    div {
                        class: "max-w-7xl mx-auto space-y-6",

                        // Render Active Tab View
                        match *active_tab.read() {
                            AppTab::Dashboard => rsx! {
                                DashboardView {
                                    nozzle: nozzle_state.read().clone(),
                                    bed: bed_state.read().clone(),
                                    toolhead: toolhead_state.read().clone(),
                                    print_job: print_job.read().clone(),
                                    on_gcode: on_gcode,
                                }
                            },
                            AppTab::Files => rsx! {
                                FilesView {
                                    on_print: move |f| {
                                        print_job.write().filename = f;
                                        print_job.write().status = "printing".to_string();
                                        active_tab.set(AppTab::Dashboard);
                                    }
                                }
                            },
                            AppTab::Console => rsx! {
                                ConsoleView {
                                    on_send: on_gcode,
                                }
                            },
                            AppTab::Hardware => rsx! {
                                HardwareView {
                                    on_gcode: on_gcode,
                                }
                            },
                            AppTab::Webcams => rsx! {
                                WebcamView {}
                            },
                            AppTab::Tune => rsx! {
                                TuneView {
                                    on_gcode: on_gcode,
                                }
                            },
                            AppTab::Farm => rsx! {
                                FarmView {}
                            },
                            AppTab::History => rsx! {
                                HistoryView {}
                            },
                            AppTab::Configure => rsx! {
                                ConfigureView {}
                            },
                        }
                    }
                }
            }

            // Bottom Status Bar
            footer {
                class: "bg-slate-900/90 border-t border-slate-800 px-6 py-2 flex items-center justify-between text-xs text-slate-400 font-mono",
                div {
                    class: "flex items-center space-x-4",
                    span { "Kinematics: CoreXY" }
                    span { "•" }
                    span { "MCU: STM32F407 (MKS SKIPR)" }
                    span { "•" }
                    span { "{last_command}" }
                }
                div {
                    span { "FluiddSail v0.1.0 • r_klipp" }
                }
            }
        }
    }
}
