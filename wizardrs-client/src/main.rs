#![windows_subsystem = "windows"]

use crate::config::Config;
use crate::error::*;
use crate::gui::{App, APPLICATION, ORGANIZATION, QUALIFIER};
use chrono::Local;
use directories::ProjectDirs;
use egui::ViewportBuilder;
use egui_extras::install_image_loaders;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, Level};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, FmtSubscriber, Registry};

pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod gui;
pub(crate) mod image_cache;
pub(crate) mod interaction;
pub(crate) mod state;

const MAX_LOGS: usize = 20;

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic_hook();
    setup_logger()?;
    debug!("started logger");

    // clean old logs
    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        let log_dir = proj_dirs.data_dir().join("logs");
        clean_old_logs(&log_dir, MAX_LOGS)?;
    }

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

    info!(?config, "starting GUI");
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

#[instrument(level = "debug")]
fn clean_old_logs(dir: &Path, max_logs: usize) -> Result<()> {
    // get all paths of log files in dir
    let mut file_paths = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && path.extension().is_some_and(|x| x == "log"))
        .collect::<Vec<_>>();

    // sort paths newest to oldest
    file_paths.sort_by_key(|path| path.metadata().and_then(|meta| meta.modified()).ok());
    file_paths.reverse();

    while file_paths.len() > max_logs {
        if let Some(oldest) = file_paths.pop() {
            debug!(log_file = ?oldest, "removing log file");
            fs::remove_file(oldest)?;
        }
    }

    Ok(())
}

fn setup_logger() -> Result<()> {
    // setup logger
    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        let log_dir = proj_dirs.data_dir().join("logs");
        fs::create_dir_all(&log_dir)?;

        let mut latest_file = log_dir.clone();
        latest_file.push("latest.log");
        let latest_log = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(latest_file)?;

        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let log_filename = format!("log-{}.log", timestamp);
        let mut log_file = log_dir.clone();
        log_file.push(log_filename);
        let datetime_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        let (latest_writer, latest_guard) = tracing_appender::non_blocking(latest_log);
        let (datetime_writer, datetime_guard) = tracing_appender::non_blocking(datetime_log);

        let console_layer = fmt::layer()
            .compact()
            .with_line_number(true)
            .with_thread_names(true);
        let latest_file_layer = fmt::layer()
            .compact()
            .with_line_number(true)
            .with_thread_names(true)
            .with_writer(latest_writer)
            .with_ansi(false);
        let datetime_file_layer = fmt::layer()
            .compact()
            .with_line_number(true)
            .with_thread_names(true)
            .with_writer(datetime_writer)
            .with_ansi(false);

        let subscriber = Registry::default()
            .with(console_layer)
            .with(latest_file_layer)
            .with(datetime_file_layer);
        tracing::subscriber::set_global_default(subscriber).expect("Failed to setup logger");

        // prevent log file guard from being dropped
        std::mem::forget(latest_guard);
        std::mem::forget(datetime_guard);
    } else {
        println!("couldn't create log files");
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .with_line_number(true)
            .compact()
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("Failed to setup logger");
    }

    Ok(())
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column())).unwrap_or("unknown".to_string());
        let message = info.payload().downcast_ref::<&str>().map_or_else(|| "unknown panic".to_string(), |s| s.to_string());

        error!("Panic occurred: {} at {}", message, location);
    }));
}