mod app;
mod dashboard;
mod detail;
mod icons;
mod invoke;
mod models;
mod palette;
mod repolist;
mod shared;
mod sidebar;

use app::App;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}
