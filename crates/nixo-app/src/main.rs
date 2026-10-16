#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use std::path::PathBuf;

use app::NixoApp;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,wgpu_hal=warn,wgpu_core=warn")),
        )
        .init();

    let initial_file = std::env::args().nth(1).map(PathBuf::from);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nixo Media Player")
            .with_inner_size([1120.0, 680.0])
            .with_min_inner_size([640.0, 400.0])
            .with_visible(true)
            .with_active(true)
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Nixo Media Player",
        native_options,
        Box::new(move |cc| Ok(Box::new(NixoApp::new(cc, initial_file)))),
    )
}
