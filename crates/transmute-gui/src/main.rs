#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console on Windows

use transmute_gui::TransmuteApp;

fn main() -> eframe::Result<()> {
    // Initialize logging with debug level for development
    tracing_subscriber::fmt()
        .with_env_filter("transmute=debug")
        .with_target(false)
        .without_time()
        .init();

    println!("===========================================");
    println!("    Transmute GUI Starting");
    println!("    Debug output enabled");
    println!("===========================================");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Transmute - Media Converter",
        options,
        Box::new(|cc| Ok(Box::new(TransmuteApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    // Placeholder: 32x32 purple icon
    let icon_data = vec![99, 102, 241, 255]; // RGBA for primary color
    let icon_pixels = vec![icon_data; 32 * 32].into_iter().flatten().collect();

    egui::IconData {
        rgba: icon_pixels,
        width: 32,
        height: 32,
    }
}
