use dioxus::prelude::*;
use crate::ui::*;

#[derive(Clone, PartialEq, Debug)]
pub struct TerminalLine {
    pub id: u64,
    pub timestamp: String,
    pub text: String,
    pub is_command: bool,
    pub is_error: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct ConsoleProps {
    pub on_send: EventHandler<String>,
}

#[component]
pub fn ConsoleView(props: ConsoleProps) -> Element {
    let mut command_input = use_signal(String::new);
    let mut lines = use_signal(|| vec![
        TerminalLine { id: 1, timestamp: "19:04:12".to_string(), text: "Klipper state: Ready".to_string(), is_command: false, is_error: false },
        TerminalLine { id: 2, timestamp: "19:04:15".to_string(), text: "G28".to_string(), is_command: true, is_error: false },
        TerminalLine { id: 3, timestamp: "19:04:18".to_string(), text: "Homing X Y Z axes completed in 3.42s".to_string(), is_command: false, is_error: false },
        TerminalLine { id: 4, timestamp: "19:05:01".to_string(), text: "M105".to_string(), is_command: true, is_error: false },
        TerminalLine { id: 5, timestamp: "19:05:01".to_string(), text: "ok T:215.0 /220.0 B:60.0 /60.0 @:45 B@:0".to_string(), is_command: false, is_error: false },
    ]);

    let mut send_cmd = move || {
        let cmd = command_input.read().trim().to_string();
        if !cmd.is_empty() {
            let id = lines.read().len() as u64 + 1;
            lines.write().push(TerminalLine {
                id,
                timestamp: "19:05:22".to_string(),
                text: cmd.clone(),
                is_command: true,
                is_error: false,
            });
            props.on_send.call(cmd);
            command_input.set(String::new());
        }
    };


    rsx! {
        div {
            class: "space-y-4",

            VCard {
                title: "G-Code Terminal & Debug Console".to_string(),
                icon: "💻".to_string(),
                div {
                    class: "space-y-4",

                    // Quick Command Chips
                    div {
                        class: "flex flex-wrap gap-2 text-xs",
                        for &quick in &["G28", "M105", "M114", "M84", "BED_MESH_CALIBRATE", "DUMP_TMC STEPPER=stepper_x", "RESTART", "FIRMWARE_RESTART"] {
                            button {
                                class: "px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono border border-slate-700 transition-colors",
                                onclick: move |_| {
                                    command_input.set(quick.to_string());
                                },
                                "{quick}"
                            }
                        }
                    }

                    // Terminal Log Output Box
                    div {
                        class: "h-96 w-full bg-slate-950 rounded-lg p-4 font-mono text-xs overflow-y-auto border border-slate-800 space-y-1.5",
                        for line in lines.read().iter() {
                            div {
                                key: "{line.id}",
                                class: "flex items-start space-x-2.5",
                                span { class: "text-slate-600 select-none", "[{line.timestamp}]" }
                                if line.is_command {
                                    span { class: "text-cyan-400 font-bold select-none", ">" }
                                    span { class: "text-slate-100 font-semibold", "{line.text}" }
                                } else if line.is_error {
                                    span { class: "text-rose-400 font-bold select-none", "!!" }
                                    span { class: "text-rose-300", "{line.text}" }
                                } else {
                                    span { class: "text-slate-400", "{line.text}" }
                                }
                            }
                        }
                    }

                    // Input Form
                    div {
                        class: "flex items-center space-x-2 pt-2",
                        input {
                            r#type: "text",
                            class: "flex-1 bg-slate-900 border border-slate-700 text-slate-100 px-4 py-2 rounded-lg font-mono text-sm focus:border-blue-500 focus:outline-none",
                            placeholder: "Send G-Code command (e.g. G28, M104 S200)...",
                            value: "{command_input}",
                            oninput: move |e| command_input.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    send_cmd();
                                }
                            },
                        }
                        VBtn {
                            label: "Send".to_string(),
                            icon: "↵".to_string(),
                            variant: "primary".to_string(),
                            onclick: move |_| send_cmd(),
                        }
                        VBtn {
                            label: "Clear".to_string(),
                            variant: "secondary".to_string(),
                            onclick: move |_| lines.set(Vec::new()),
                        }
                    }
                }
            }
        }
    }
}
