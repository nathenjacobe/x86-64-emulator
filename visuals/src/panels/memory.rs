//! memory panel: hex dump of a window of memory

use eframe::egui::{self, RichText};
use emulator::Machine;

use crate::theme;

/// bytes per row
const ROW: u64 = 16;
/// rows to display
const ROWS: u64 = 8;

/// draws the memory view, starting at the row containing rip
pub fn show(ui: &mut egui::Ui, machine: &Machine) {
    ui.heading("memory");

    let rip = machine.cpu.registers.rip();
    let start = rip & !(ROW - 1);

    egui::Grid::new("memory")
        .num_columns(ROW as usize + 2)
        .spacing([6.0, 3.0])
        .show(ui, |ui| {
            header(ui);
            ui.end_row();

            for row in 0..ROWS {
                let base = start.wrapping_add(row * ROW);
                ui.monospace(dim(format!("{base:#010x}")));
                for offset in 0..ROW {
                    ui.monospace(byte_text(machine, base.wrapping_add(offset), rip));
                }
                ui.monospace(dim(ascii(machine, base)));
                ui.end_row();
            }
        });

    ui.add_space(6.0);
    ui.monospace(dim("highlighted byte is the instruction pointer"));
}

/// the offset ruler shown above the hex bytes
fn header(ui: &mut egui::Ui) {
    ui.monospace(dim("address"));
    for column in 0..ROW {
        ui.monospace(dim(format!("{column:02x}")));
    }
    ui.monospace(dim("ascii"));
}

/// one hex byte, highlighted when it is the instruction pointer
fn byte_text(machine: &Machine, address: u64, rip: u64) -> RichText {
    match machine.memory.read_u8(address) {
        Ok(byte) if address == rip => RichText::new(format!("{byte:02x}"))
            .strong()
            .color(theme::ACCENT)
            .background_color(theme::POINTER_BG),
        Ok(byte) => RichText::new(format!("{byte:02x}")),
        Err(_) => dim("??"),
    }
}

/// the printable rendering of one row
fn ascii(machine: &Machine, base: u64) -> String {
    let mut text = String::with_capacity(ROW as usize);
    for offset in 0..ROW {
        match machine.memory.read_u8(base.wrapping_add(offset)) {
            Ok(byte) if byte.is_ascii_graphic() || byte == b' ' => text.push(byte as char),
            Ok(_) => text.push('.'),
            Err(_) => text.push(' '),
        }
    }
    text
}

/// dim secondary text
fn dim(text: impl Into<String>) -> RichText {
    RichText::new(text).color(theme::DIM)
}
