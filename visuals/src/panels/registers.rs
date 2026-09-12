//! register panel: shows the general purpose registers and flags

use eframe::egui;
use emulator::Machine;

/// draws the register view
pub fn show(ui: &mut egui::Ui, machine: &Machine) {
    ui.heading("registers");
    ui.monospace(format!("flags: {}", machine.cpu.flags));
    ui.monospace(format!("  rip: {:#018x}", machine.cpu.registers.rip()));
    for (reg, value) in machine.cpu.registers.iter() {
        ui.monospace(format!("{reg:>4}: {value:#018x}"));
    }
}
