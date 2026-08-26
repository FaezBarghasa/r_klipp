use dioxus::prelude::*;

#[component]
pub fn DigitalTwin() -> Element {
    rsx! {
        div {
            class: "p-4",
            h2 { class: "text-xl font-bold", "Digital Twin" },
            div {
                class: "mt-2 border p-6 text-center text-slate-400 bg-slate-900 rounded",
                "3D Viewport Simulation"
            }
        }
    }
}
