use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    pub text: String,
    #[props(default = "info".to_string())]
    pub variant: String, // "success", "warning", "danger", "info", "neutral"
}

#[component]
pub fn VBadge(props: BadgeProps) -> Element {
    let style = match props.variant.as_str() {
        "success" => "bg-emerald-500/15 text-emerald-400 border-emerald-500/30",
        "warning" => "bg-amber-500/15 text-amber-400 border-amber-500/30",
        "danger" => "bg-rose-500/15 text-rose-400 border-rose-500/30",
        "neutral" => "bg-slate-700/30 text-slate-400 border-slate-700/50",
        _ => "bg-blue-500/15 text-blue-400 border-blue-500/30",
    };

    rsx! {
        span {
            class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border {style}",
            "{props.text}"
        }
    }
}
