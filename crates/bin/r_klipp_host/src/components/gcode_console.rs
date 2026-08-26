use dioxus::prelude::*;

#[component]
pub fn GcodeConsole() -> Element {
    let mut command = use_signal(String::new);

    rsx! {
        div {
            class: "p-4",
            h2 { class: "text-xl font-bold", "G-code Console" },
            div {
                class: "mt-2 flex",
                input {
                    class: "border p-1 flex-grow bg-slate-900 text-white",
                    value: "{command}",
                    oninput: move |evt| command.set(evt.value()),
                }
                button {
                    class: "bg-blue-500 text-white p-1 ml-2",
                    onclick: move |_| {
                        command.set(String::new());
                    },
                    "Send"
                }
            }
        }
    }
}
