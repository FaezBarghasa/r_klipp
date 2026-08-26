use dioxus::prelude::*;

#[component]
pub fn FileManager() -> Element {
    let files = use_signal(|| vec!["file1.gcode".to_string(), "file2.gcode".to_string()]);

    rsx! {
        div {
            class: "p-4",
            h2 { class: "text-xl font-bold", "File Manager" },
            div {
                class: "mt-2 space-y-1",
                for file in files.read().iter() {
                    p { key: "{file}", "{file}" }
                }
            }
        }
    }
}
