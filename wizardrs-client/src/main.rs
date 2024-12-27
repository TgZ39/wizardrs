//#![windows_subsystem = "windows"]

use crate::error::*;
use crate::gui::App;
use egui::{ViewportBuilder, Visuals};
use egui_extras::install_image_loaders;
use std::sync::Arc;
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
            // setup fonts
            let mut fonts = egui::FontDefinitions::default();
            // install monospace font
            fonts.font_data.insert(
                "CaskaydiaCoveNerdFontMono-Regular".to_owned(),
                Arc::from(egui::FontData::from_static(include_bytes!(
                    "../../assets/fonts/CaskaydiaCoveNerdFontMono-Regular.ttf"
                ))),
            );
            // set font as first
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "CaskaydiaCoveNerdFontMono-Regular".to_owned());
            // set monospace font first
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "CaskaydiaCoveNerdFontMono-Regular".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            // set dark mode default
            cc.egui_ctx
                .style_mut(|style| style.visuals = Visuals::dark());

            // set zoom
            cc.egui_ctx.set_zoom_factor(1.2);

            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )?;

    Ok(())
}
