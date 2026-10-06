//! disassembly panel: shows instructions around the instruction pointer

use eframe::egui::{self, RichText};
use emulator::{Machine, instruction};

use crate::theme;

/// how many instructions to try to show
const WINDOW: usize = 8;

/// draws the disassembly view, highlighting the instruction at rip
pub fn show(ui: &mut egui::Ui, machine: &Machine) {
    ui.heading("disassembly");

    let rip = machine.cpu.registers.rip();
    let mut address = rip;

    for _ in 0..WINDOW {
        match instruction::decode(&machine.memory, address) {
            Ok(decoded) => {
                line(ui, address, &decoded.to_string(), address == rip);
                address = address.wrapping_add(decoded.length as u64);
            }
            Err(_) => {
                let text = match machine.memory.read_u8(address) {
                    Ok(byte) => format!(".byte {byte:#04x}"),
                    Err(_) => "<out of bounds>".to_owned(),
                };
                line(ui, address, &text, address == rip);
                break;
            }
        }
    }
}

/// draws one address plus its decoded text, tinted when it is the current one
fn line(ui: &mut egui::Ui, address: u64, text: &str, current: bool) {
    let mut rich = RichText::new(format!("{address:#010x}  {text}")).monospace();
    if current {
        rich = rich
            .strong()
            .color(theme::ACCENT)
            .background_color(theme::POINTER_BG);
    }
    ui.label(rich);
}
