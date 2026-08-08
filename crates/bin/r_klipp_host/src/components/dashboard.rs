
use dioxus::prelude::*;
use r_klipp_api::LinkHealth;

#[component]
pub fn Dashboard(cx: Scope) -> Element {
    let link_health = use_signal(cx, || LinkHealth {
        rtt_us: 0,
        buffer_fill_percent: 100,
        dropped_packets: 0,
    });

    let (mode_text, mode_color) = {
        let health = link_health.read();
        if health.rtt_us < 2000 && health.buffer_fill_percent > 50 {
            ("PREDICTIVE MODE (Tier 1)", "bg-green-500")
        } else if health.rtt_us > 5000 || health.buffer_fill_percent < 30 {
            ("BASIC MODE (Tier 2)", "bg-yellow-500")
        } else {
            ("LINK DEGRADED", "bg-red-500")
        }
    };

    cx.render(rsx! {
        div {
            class: "p-4",
            h1 { class: "text-2xl font-bold", "r_klipp Dashboard" },
            div {
                class: "mt-4 p-2 rounded {mode_color} text-white",
                "{mode_text}"
            }
            div {
                class: "mt-4",
                p { "RTT: {link_health.read().rtt_us} µs" },
                p { "Buffer Fill: {link_health.read().buffer_fill_percent}%" }
            }
        }
    })
}
