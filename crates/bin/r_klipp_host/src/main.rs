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
    let toolhead_state = use_signal(ToolheadState::default);
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

                // Logo & Printer Identity
                div {
                    class: "flex items-center space-x-4",
                    div {
                        class: "flex items-center space-x-2",
                        span { class: "text-2xl", "⚡" }
                        div {
                            span { class: "font-black tracking-wider text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-indigo-300 to-purple-400 text-lg", "r_klipp" }
                            span { class: "text-xs font-mono bg-blue-900/60 text-blue-300 px-1.5 py-0.5 rounded ml-1.5 border border-blue-700/50", "HOST" }
                        }
                    }

                    div { class: "h-5 w-px bg-slate-800" }

                    // Emergency Stop & Reset Buttons
                    div {
                        class: "flex items-center space-x-2",
                        button {
                            class: "px-3 py-1 bg-red-600/90 hover:bg-red-600 active:scale-95 transition text-white font-bold text-xs uppercase tracking-wider rounded shadow-sm border border-red-500/80 flex items-center space-x-1.5",
                            onclick: move |_| on_gcode("M112".to_string()),
                            span { "🛑" }
                            span { "Emergency Stop" }
                        }
                        button {
                            class: "px-2.5 py-1 bg-slate-800 hover:bg-slate-700 active:scale-95 transition text-slate-300 text-xs font-semibold rounded border border-slate-700",
                            onclick: move |_| on_gcode("FIRMWARE_RESTART".to_string()),
                            "Restart"
                        }
                    }
                }

                // Global Status Gauges & Machine State
                div {
                    class: "flex items-center space-x-6",
                    // Extruder gauge
                    div {
                        class: "flex items-center space-x-2 text-xs",
                        span { class: "text-red-400 text-sm", "♨" }
                        div {
                            div { class: "text-slate-400 font-medium", "Nozzle" }
                            div { class: "font-mono font-bold text-slate-200", "{nozzle_state.read().current:.1}°C / {nozzle_state.read().target:.0}°C" }
                        }
                    }

                    // Bed gauge
                    div {
                        class: "flex items-center space-x-2 text-xs",
                        span { class: "text-amber-400 text-sm", "♨" }
                        div {
                            div { class: "text-slate-400 font-medium", "Bed" }
                            div { class: "font-mono font-bold text-slate-200", "{bed_state.read().current:.1}°C / {bed_state.read().target:.0}°C" }
                        }
                    }

                    // Printing State Badge
                    VBadge {
                        variant: match print_job.read().status.as_str() {
                            "printing" => "success".to_string(),
                            "paused" => "warning".to_string(),
                            "error" => "danger".to_string(),
                            _ => "neutral".to_string(),
                        },
                        text: print_job.read().status.to_uppercase(),
                    }
                }
            }

            // Main Body (Sidebar + Content Workspace)
            div {
                class: "flex-1 flex overflow-hidden",

                // Navigation Sidebar
                aside {
                    class: "w-64 bg-slate-900 border-r border-slate-800 p-4 flex flex-col justify-between shrink-0",
                    nav {
                        class: "space-y-1.5 font-medium text-sm",
                        {
                            let tabs = [
                                (AppTab::Dashboard, "📊", "Dashboard"),
                                (AppTab::Files, "📁", "G-Code Files"),
                                (AppTab::Console, "💻", "Terminal Console"),
                                (AppTab::Hardware, "🔧", "Hardware & Tools"),
                                (AppTab::Webcams, "📷", "Webcam Feeds"),
                                (AppTab::Tune, "🎛️", "Tune & Shaping"),
                                (AppTab::Farm, "🚜", "Print Farm"),
                                (AppTab::History, "📜", "Print History"),
                                (AppTab::Configure, "⚙️", "Configuration"),
                            ];

                            tabs.into_iter().map(|(tab, icon, label)| {
                                let is_active = active_tab() == tab;
                                rsx! {
                                    button {
                                        key: "{label}",
                                        class: if is_active {
                                            "w-full flex items-center space-x-3 px-3.5 py-2.5 rounded-lg bg-blue-600/20 text-blue-400 border border-blue-500/30 font-semibold shadow-inner"
                                        } else {
                                            "w-full flex items-center space-x-3 px-3.5 py-2.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition"
                                        },
                                        onclick: move |_| active_tab.set(tab),
                                        span { class: "text-lg", "{icon}" }
                                        span { "{label}" }
                                    }
                                }
                            })
                        }
                    }

                    // System Resource Summary Footer in Sidebar
                    div {
                        class: "bg-slate-950/60 p-3 rounded-lg border border-slate-800/80 space-y-2 text-xs text-slate-400 font-mono",
                        div { class: "flex justify-between", span { "Host CPU:" } span { class: "text-slate-200 font-bold", "12.4%" } }
                        div { class: "flex justify-between", span { "RAM Usage:" } span { class: "text-slate-200 font-bold", "412 MB / 2 GB" } }
                        div { class: "flex justify-between", span { "MCU Load:" } span { class: "text-emerald-400 font-bold", "4.2%" } }
                    }
                }

                // Dynamic Workspace Content View
                main {
                    class: "flex-1 overflow-y-auto p-6 bg-slate-950/80",
                    div {
                        class: "max-w-7xl mx-auto space-y-6",
                        match active_tab() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dioxus_app_state_initialization() {
        let nozzle = HeaterState::default();
        assert_eq!(nozzle.name, "Extruder");
        assert_eq!(nozzle.current, 215.0);
        assert_eq!(nozzle.target, 220.0);

        let bed = HeaterState {
            name: "Heater Bed".to_string(),
            current: 60.2,
            target: 60.0,
            power: 0.22,
            history: vec![58.0, 59.0, 59.5, 60.0, 60.2],
        };
        assert_eq!(bed.target, 60.0);

        let toolhead = ToolheadState::default();
        assert_eq!(toolhead.homed_axes, "xyz");
        assert_eq!(toolhead.feedrate, 150.0);

        let job = PrintJobState::default();
        assert_eq!(job.status, "printing");
        assert_eq!(job.progress, 68.4);
    }

    #[test]
    fn test_dioxus_vdom_lifecycle() {
        let mut vdom = VirtualDom::new(App);
        vdom.rebuild_in_place();
        // Virtual DOM rendered root correctly without panicking
    }

    #[test]
    fn test_dioxus_views_instantiation() {
        let mut vdom_dash = VirtualDom::new(|| rsx! {
            DashboardView {
                nozzle: HeaterState::default(),
                bed: HeaterState::default(),
                toolhead: ToolheadState::default(),
                print_job: PrintJobState::default(),
                on_gcode: EventHandler::new(|_| {}),
            }
        });
        vdom_dash.rebuild_in_place();

        let mut vdom_files = VirtualDom::new(|| rsx! {
            FilesView {
                on_print: EventHandler::new(|_| {}),
            }
        });
        vdom_files.rebuild_in_place();

        let mut vdom_console = VirtualDom::new(|| rsx! {
            ConsoleView {
                on_send: EventHandler::new(|_| {}),
            }
        });
        vdom_console.rebuild_in_place();

        let mut vdom_hw = VirtualDom::new(|| rsx! {
            HardwareView {
                on_gcode: EventHandler::new(|_| {}),
            }
        });
        vdom_hw.rebuild_in_place();

        let mut vdom_tune = VirtualDom::new(|| rsx! {
            TuneView {
                on_gcode: EventHandler::new(|_| {}),
            }
        });
        vdom_tune.rebuild_in_place();

        let mut vdom_webcam = VirtualDom::new(|| rsx! {
            WebcamView {}
        });
        vdom_webcam.rebuild_in_place();

        let mut vdom_farm = VirtualDom::new(|| rsx! {
            FarmView {}
        });
        vdom_farm.rebuild_in_place();

        let mut vdom_history = VirtualDom::new(|| rsx! {
            HistoryView {}
        });
        vdom_history.rebuild_in_place();

        let mut vdom_config = VirtualDom::new(|| rsx! {
            ConfigureView {}
        });
        vdom_config.rebuild_in_place();
    }
}
