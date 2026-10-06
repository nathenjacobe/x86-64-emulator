//! egui/eframe frontend for the emulator
//!
//! shows the registers, disassembly and memory panels for the selected demo,
//! with a toolbar, status bar and keyboard shortcuts for stepping and running

mod app;
mod panels;
mod theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "x86-64 emulator",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::Visuals::default()))
        }),
    )
}
