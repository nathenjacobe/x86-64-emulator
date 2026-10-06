//! register panel: shows the general purpose registers and flags

use eframe::egui::{self, RichText};
use emulator::Machine;

use super::Before;
use crate::theme;

/// the status flags in the order they are displayed
const FLAGS: [&str; 4] = ["cf", "zf", "sf", "of"];

/// draws the register view, highlighting values changed by the last step
pub fn show(ui: &mut egui::Ui, machine: &Machine, before: &Before) {
    ui.heading("registers");

    let previous = before.registers();
    egui::Grid::new("registers")
        .num_columns(3)
        .spacing([14.0, 3.0])
        .min_col_width(0.0)
        .show(ui, |ui| {
            for (reg, value) in machine.cpu.registers.iter() {
                let changed = previous.get(reg) != value;
                ui.monospace(changed_text(reg.name(), changed));
                ui.monospace(changed_text(format!("{value:#018x}"), changed));
                ui.monospace(dim(format!("{value}")));
                ui.end_row();
            }

            let rip = machine.cpu.registers.rip();
            let changed = previous.rip() != rip;
            ui.monospace(changed_text("rip", changed));
            ui.monospace(changed_text(format!("{rip:#018x}"), changed));
            ui.monospace(dim(format!("{rip}")));
            ui.end_row();
        });

    ui.add_space(8.0);
    flags(ui, machine, before);
}

/// draws the four status flags, bolding the ones that are set
fn flags(ui: &mut egui::Ui, machine: &Machine, before: &Before) {
    let now = machine.cpu.flags;
    let was = before.flags();
    let set = [now.carry, now.zero, now.sign, now.overflow];
    let was_set = [was.carry, was.zero, was.sign, was.overflow];

    ui.horizontal(|ui| {
        ui.monospace(dim("flags"));
        for (index, name) in FLAGS.iter().enumerate() {
            let changed = set[index] != was_set[index];
            let mut text = RichText::new(format!("{name}={}", set[index] as u8));
            text = if changed {
                text.strong().color(theme::ACCENT)
            } else if set[index] {
                text.strong()
            } else {
                text.color(theme::DIM)
            };
            ui.monospace(text);
        }
    });
}

/// bolds and tints text when the value it shows changed
fn changed_text(text: impl Into<String>, changed: bool) -> RichText {
    let text = RichText::new(text);
    if changed {
        text.strong().color(theme::ACCENT)
    } else {
        text
    }
}

/// dim secondary text
fn dim(text: impl Into<String>) -> RichText {
    RichText::new(text).color(theme::DIM)
}
