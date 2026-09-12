//! disassembly panel: shows instructions around the instruction pointer

use eframe::egui;
use emulator::{Machine, instruction};

/// how many instructions to try to show
const WINDOW: usize = 8;

/// draws the disassembly view
pub fn show(ui: &mut egui::Ui, machine: &Machine) {
    ui.heading("disassembly");

    let mut address = machine.cpu.registers.rip();
    for _ in 0..WINDOW {
        match instruction::decode(&machine.memory, address) {
            Ok(decoded) => {
                ui.monospace(format!("{address:#010x}  {decoded}"));
                address = address.wrapping_add(decoded.length as u64);
            }
            Err(_) => {
                match machine.memory.read_u8(address) {
                    Ok(byte) => ui.monospace(format!("{address:#010x}  .byte {byte:#04x}")),
                    Err(_) => ui.monospace(format!("{address:#010x}  <out of bounds>")),
                };
                break;
            }
        }
    }
}
