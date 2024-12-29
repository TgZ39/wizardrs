#![windows_subsystem = "windows"]

use crate::config::Config;
use crate::error::*;
use crate::gui::{App, APPLICATION, ORGANIZATION, QUALIFIER};
use directories::ProjectDirs;
use egui::ViewportBuilder;
use egui_extras::install_image_loaders;
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod gui;
pub(crate) mod image_cache;
pub(crate) mod interaction;
pub(crate) mod state;

#[tokio::main]
async fn main() -> Result<()> {
    // setup logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_line_number(true)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // open config
    let config = match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        Some(proj_dirs) => {
            let mut config_path = proj_dirs.config_dir().to_path_buf();
            config_path.push("config.json");

            Config::load(&config_path)?
        }
        None => {
            error!("unable to find app dir");
            return Ok(());
        }
    };

    // GUI
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

            // set theme preference
            cc.egui_ctx.set_theme(config.theme);

            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(config)))
        }),
    )?;

    Ok(())
}
