use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct BtnProps {
    #[props(default = String::new())]
    pub label: String,
    #[props(default = String::new())]
    pub icon: String,
    #[props(default = "primary".to_string())]
    pub variant: String, // "primary", "secondary", "danger", "success", "warning", "ghost"
    #[props(default = "md".to_string())]
    pub size: String, // "sm", "md", "lg"
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = String::new())]
    pub class: String,
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Option<Element>,
}

#[component]
pub fn VBtn(props: BtnProps) -> Element {
    let size_cls = match props.size.as_str() {
        "sm" => "px-2.5 py-1.5 text-xs font-medium",
        "lg" => "px-5 py-3 text-base font-semibold",
        _ => "px-4 py-2 text-sm font-medium",
    };

    let variant_cls = match props.variant.as_str() {
        "secondary" => "bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 active:bg-slate-900",
        "danger" => "bg-rose-600 hover:bg-rose-500 text-white shadow-rose-900/20 shadow-lg active:bg-rose-700",
        "success" => "bg-emerald-600 hover:bg-emerald-500 text-white shadow-emerald-900/20 shadow-lg active:bg-emerald-700",
        "warning" => "bg-amber-600 hover:bg-amber-500 text-white shadow-amber-900/20 shadow-lg active:bg-amber-700",
        "ghost" => "bg-transparent hover:bg-slate-800 text-slate-300 active:bg-slate-900",
        _ => "bg-blue-600 hover:bg-blue-500 text-white shadow-blue-900/20 shadow-lg active:bg-blue-700",
    };

    let disabled_cls = if props.disabled {
        "opacity-50 cursor-not-allowed pointer-events-none"
    } else {
        "cursor-pointer active:scale-[0.98]"
    };

    rsx! {
        button {
            class: "inline-flex items-center justify-center space-x-2 rounded-lg transition-all duration-150 select-none {size_cls} {variant_cls} {disabled_cls} {props.class}",
            disabled: props.disabled,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            if !props.icon.is_empty() {
                span { class: "text-base", "{props.icon}" }
            }
            if !props.label.is_empty() {
                span { "{props.label}" }
            }
            if let Some(children) = props.children {
                {children}
            }
        }
    }
}
