//! egui/eframe frontend for the emulator
//!
//! a barebones window with the register, disassembly and memory panels;
//! the panels are wired up to a real machine in a later phase

mod app;
mod panels;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "x86-64 emulator",
        options,
        Box::new(|_cc| Ok(Box::new(app::Visuals::default()))),
    )
}
