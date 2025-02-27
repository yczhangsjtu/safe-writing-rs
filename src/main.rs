#![windows_subsystem = "windows"]

use safe_writing_rs::app::MyApp;

use eframe::egui;

#[cfg(target_os = "windows")]
const ICON: &[u8] = include_bytes!("..\\assets\\icon.png");

#[cfg(target_os = "macos")]
const ICON: &[u8] = include_bytes!("../assets/icon.png");

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(egui::IconData {
                rgba: ICON.to_vec(),
                width: 32,
                height: 32,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "Safe Writing",
        options,
        Box::new(|cc: &eframe::CreationContext<'_>| {
            egui_material_icons::initialize(&cc.egui_ctx);
            Ok(Box::<MyApp>::new(MyApp::new(cc)))
        }),
    )
}
