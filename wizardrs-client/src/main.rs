#![windows_subsystem = "windows"]

use crate::error::*;
use crate::gui::App;
use egui::ViewportBuilder;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub(crate) mod client;
pub(crate) mod error;
pub(crate) mod gui;
pub(crate) mod state;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "Wizardrs",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )?;

    Ok(())
}
