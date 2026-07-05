use dioxus::prelude::*;
use std::fs;
use std::path::PathBuf;

#[component]
pub fn FileManager(cx: Scope) -> Element {
    let files = use_state(cx, || {
        fs::read_dir("./gcode")
            .map(|res| {
                res.filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .collect::<Vec<PathBuf>>()
            })
            .unwrap_or_default()
    });

    render! {
        div {
            h2 { "File Manager" }
            div {
                class: "border p-4",
                ul {
                    for path in files.get() {
                        li { "{path.to_str().unwrap_or(\"Invalid path\")}" }
                    }
                }
            }
            div {
                h3 { "G-code Console" }
                textarea {
                    class: "w-full border",
                    rows: "10",
                    placeholder: "Enter G-code here",
                }
                button {
                    class: "p-2 bg-blue-500 text-white rounded",
                    "Send"
                }
            }
        }
    }
}
