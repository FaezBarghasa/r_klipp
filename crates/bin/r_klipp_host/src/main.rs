
mod mcu_comms;
mod components;

use dioxus::prelude::*;
use components::dashboard::Dashboard;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    DashboardPage {},
    #[route("/files")]
    FileManager {},
    #[route("/settings")]
    Settings {},
}

#[component]
fn DashboardPage() -> Element {
    rsx! {
        Dashboard {}
    }
}

#[component]
fn FileManager() -> Element {
    rsx! {
        div { "File Manager" }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        div { "Settings" }
    }
}
