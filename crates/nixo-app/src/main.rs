#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod components;
mod flags;
mod home;
mod i18n;
mod icons;
mod input;
mod lucide;
mod osd;
mod osd_bar;
mod playlist;
mod popovers;
mod settings;
mod theme;
mod toast;
mod widgets;

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

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Nixo Media Player")
        .with_inner_size([1120.0, 680.0])
        .with_min_inner_size([360.0, 240.0])
        .with_transparent(true)
        .with_drag_and_drop(true);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Nixo Media Player",
        native_options,
        Box::new(move |cc| Ok(Box::new(NixoApp::new(cc, initial_file)))),
    )
}

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../../../assets/icon.png");
    let img = image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}
