
use dioxus::prelude::*;

#[server]
async fn send_gcode(command: String) -> Result<(), ServerFnError> {
    println!("Received G-code: {}", command);
    // In a real app, this would send the command to the G-code parser
    Ok(())
}

#[component]
pub fn GcodeConsole(cx: Scope) -> Element {
    let command = use_state(cx, String::new);

    cx.render(rsx! {
        div {
            class: "p-4",
            h2 { class: "text-xl font-bold", "G-code Console" },
            div {
                class: "mt-2 flex",
                input {
                    class: "border p-1 flex-grow",
                    value: "{command}",
                    oninput: move |evt| command.set(evt.value.clone()),
                },
                button {
                    class: "bg-blue-500 text-white p-1 ml-2",
                    onclick: move |_| {
                        let cmd = command.get().clone();
                        spawn(async move {
                            send_gcode(cmd).await.ok();
                        });
                        command.set(String::new());
                    },
                    "Send"
                }
            }
        }
    })
}
