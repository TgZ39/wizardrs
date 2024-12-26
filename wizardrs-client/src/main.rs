#![windows_subsystem = "windows"]

use crate::error::*;
use crate::gui::App;
use egui::ViewportBuilder;
use egui_extras::install_image_loaders;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub(crate) mod client;
pub(crate) mod error;
pub(crate) mod gui;
pub(crate) mod interaction;
pub(crate) mod state;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        ..Default::default()
    };

    info!("starting GUI");
    eframe::run_native(
        "Wizardrs",
        options,
        Box::new(|cc| {
            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )?;

    Ok(())
}
