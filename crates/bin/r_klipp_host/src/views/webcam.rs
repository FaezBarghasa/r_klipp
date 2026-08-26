use dioxus::prelude::*;
use crate::ui::*;

#[component]
pub fn WebcamView() -> Element {
    let mut selected_stream = use_signal(|| "MJPEG Stream (Default)".to_string());
    let mut fps_target = use_signal(|| 30);

    rsx! {
        div {
            class: "space-y-6",

            VCard {
                title: "Adaptive Live Webcams".to_string(),
                subtitle: "Multi-stream camera hub with HLS, WebRTC, and MJPEG support".to_string(),
                icon: "📷".to_string(),
                div {
                    class: "space-y-4",

                    // Controls Bar
                    div {
                        class: "flex flex-wrap items-center justify-between gap-3 bg-slate-800/60 p-3 rounded-lg border border-slate-700/60",
                        div {
                            class: "flex items-center space-x-3",
                            span { class: "text-xs font-semibold text-slate-300", "Stream Protocol:" }
                            select {
                                class: "bg-slate-900 border border-slate-700 text-slate-200 text-xs rounded px-3 py-1.5 focus:border-blue-500 focus:outline-none",
                                value: "{selected_stream}",
                                onchange: move |e| selected_stream.set(e.value()),
                                option { "MJPEG Stream (Default)" }
                                option { "WebRTC (Low-Latency)" }
                                option { "HLS Stream (Adaptive)" }
                                option { "Janus Gateway" }
                            }
                        }
                        div {
                            class: "flex items-center space-x-2",
                            VBtn {
                                label: "Take Snapshot".to_string(),
                                icon: "📸".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| {},
                            }
                            VBtn {
                                label: "Full Screen".to_string(),
                                icon: "⛶".to_string(),
                                variant: "secondary".to_string(),
                                size: "sm".to_string(),
                                onclick: move |_| {},
                            }
                        }
                    }

                    // Simulated Video Feed Player Container
                    div {
                        class: "relative aspect-video w-full bg-slate-950 rounded-xl overflow-hidden border border-slate-800 flex flex-col items-center justify-center shadow-inner group",
                        
                        // Video stream placeholder / overlay
                        div {
                            class: "absolute top-4 left-4 z-10 flex space-x-2",
                            VBadge { text: "LIVE".to_string(), variant: "danger".to_string() }
                            VBadge { text: format!("{} FPS", *fps_target.read()), variant: "neutral".to_string() }
                            VBadge { text: "1080p @ 6000kbps".to_string(), variant: "info".to_string() }
                        }

                        div {
                            class: "text-center space-y-2",
                            span { class: "text-5xl block animate-pulse", "📹" }
                            p { class: "text-sm text-slate-300 font-semibold", "Bed Chamber Main Camera (v4l2src)" }
                            p { class: "text-xs text-slate-500 font-mono", "Protocol: {selected_stream} | Latency: 42ms" }
                        }

                        // Stream footer
                        div {
                            class: "absolute bottom-0 inset-x-0 bg-gradient-to-t from-slate-950/90 to-transparent p-4 flex justify-between items-center opacity-0 group-hover:opacity-100 transition-opacity",
                            span { class: "text-xs text-slate-300 font-mono", "Camera 1/1: Voron StealthBurner Nozzle Cam" }
                            div {
                                class: "flex space-x-2",
                                VBtn { label: "Flip H".to_string(), variant: "ghost".to_string(), size: "sm".to_string() }
                                VBtn { label: "Flip V".to_string(), variant: "ghost".to_string(), size: "sm".to_string() }
                            }
                        }
                    }
                }
            }
        }
    }
}
