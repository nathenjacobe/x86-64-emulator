//! memory panel: hex dump of a window of memory

use eframe::egui;
use emulator::Machine;

/// bytes per row
const ROW: u64 = 16;
/// rows to display
const ROWS: u64 = 8;

/// draws the memory view, starting at the row containing rip
pub fn show(ui: &mut egui::Ui, machine: &Machine) {
    ui.heading("memory");

    let rip = machine.cpu.registers.rip();
    let start = rip & !(ROW - 1);

    for row in 0..ROWS {
        let base = start.wrapping_add(row * ROW);
        let mut bytes = String::with_capacity(ROW as usize * 3);
        for offset in 0..ROW {
            match machine.memory.read_u8(base.wrapping_add(offset)) {
                Ok(byte) => bytes.push_str(&format!("{byte:02x} ")),
                Err(_) => bytes.push_str("?? "),
            }
        }
        ui.monospace(format!("{base:#010x}  {bytes}"));
    }
}
