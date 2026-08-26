use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default = String::new())]
    pub title: String,
    #[props(default = String::new())]
    pub subtitle: String,
    #[props(default = String::new())]
    pub icon: String,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = true)]
    pub elevation: bool,
    pub children: Element,
}

#[component]
pub fn VCard(props: CardProps) -> Element {
    let elevation_cls = if props.elevation {
        "shadow-lg border border-slate-800/80"
    } else {
        "border border-slate-800"
    };

    rsx! {
        div {
            class: "bg-slate-900/90 backdrop-blur-md rounded-xl overflow-hidden transition-all duration-200 hover:border-slate-700/80 {elevation_cls} {props.class}",
            if !props.title.is_empty() {
                div {
                    class: "px-5 py-3.5 border-b border-slate-800/80 flex items-center justify-between bg-slate-900/50",
                    div {
                        class: "flex items-center space-x-2.5",
                        if !props.icon.is_empty() {
                            span { class: "text-blue-400 text-lg", "{props.icon}" }
                        }
                        div {
                            h3 { class: "font-semibold text-slate-100 text-sm tracking-wide uppercase", "{props.title}" }
                            if !props.subtitle.is_empty() {
                                p { class: "text-xs text-slate-400 font-normal", "{props.subtitle}" }
                            }
                        }
                    }
                }
            }
            div {
                class: "p-4",
                {props.children}
            }
        }
    }
}
