fn main() -> eframe::Result {
    eframe::run_native(
        "Rustcord",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(rustcord_client::RustcordApp::new(cc)))),
    )
}
