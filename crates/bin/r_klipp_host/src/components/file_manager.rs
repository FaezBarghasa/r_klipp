
use dioxus::prelude::*;

#[component]
pub fn FileManager(cx: Scope) -> Element {
    let files = use_resource(cx, || async {
        // Mock file loading
        vec!["file1.gcode".to_string(), "file2.gcode".to_string()]
    });

    cx.render(rsx! {
        div {
            class: "p-4",
            h2 { class: "text-xl font-bold", "File Manager" },
            div {
                class: "mt-2",
                files.read().iter().map(|file| rsx! {
                    p { "{file}" }
                })
            }
        }
    })
}
