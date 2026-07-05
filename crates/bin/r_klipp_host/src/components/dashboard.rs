
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn Dashboard() -> Element {
    let temp = use_signal(|| 25.0);

    // In a real application, you would establish a WebSocket connection here
    // and update the `temp` signal with incoming data.

    rsx! {
        div {
            h1 { "Dashboard" }
            div {
                "Temperature: {temp}°C"
            }
            svg {
                width: "400",
                height: "200",
                rect {
                    width: "400",
                    height: "200",
                    fill: "#f0f0f0",
                }
                // A simple line representing temperature history would go here
            }
        }
    }
}
