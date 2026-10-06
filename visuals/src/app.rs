//! the eframe application: owns the machine and drives the update loop

use demos::Demo;
use eframe::egui::{self, Key};
use emulator::Machine;

use crate::panels::{self, Before};

/// maximum steps a single "Run" click will execute before giving up
const RUN_STEP_LIMIT: u64 = 10_000;

/// top-level UI state
pub struct Visuals {
    machine: Machine,
    demo: &'static Demo,
    status: String,
    /// register and flag state before the last step, for change highlighting
    before: Before,
}

impl Default for Visuals {
    fn default() -> Self {
        let demo = demos::default_demo();
        let machine = demos::machine(demo);
        let before = Before::capture(&machine);
        Self {
            machine,
            demo,
            status: format!("loaded {}", demo.name),
            before,
        }
    }
}

impl Visuals {
    /// reloads the currently selected demo from scratch
    fn reload(&mut self) {
        self.machine = demos::machine(self.demo);
        self.before = Before::capture(&self.machine);
        self.status = format!("reset {}", self.demo.name);
    }

    /// executes a single instruction
    fn step(&mut self) {
        self.before = Before::capture(&self.machine);
        match self.machine.step() {
            Ok(info) => self.status = format!("{:#x}: {}", info.address, info.instruction),
            Err(err) => self.status = err.to_string(),
        }
    }

    /// runs until halt or the step limit
    fn run(&mut self) {
        self.before = Before::capture(&self.machine);
        match self.machine.run(RUN_STEP_LIMIT) {
            Ok(steps) => self.status = format!("ran {steps} step(s)"),
            Err(err) => self.status = err.to_string(),
        }
    }

    /// the toolbar across the top of the window
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Step").on_hover_text("F10").clicked() {
                self.step();
            }
            if ui.button("Run").on_hover_text("F5").clicked() {
                self.run();
            }
            if ui.button("Reset").on_hover_text("F9").clicked() {
                self.reload();
            }
            ui.separator();
            let state = if self.machine.cpu.halted {
                "halted"
            } else {
                "running"
            };
            ui.monospace(state);
        });

        ui.horizontal(|ui| {
            ui.label("demo:");
            for demo in demos::demos() {
                let selected = std::ptr::eq(demo, self.demo);
                let label = ui.selectable_label(selected, demo.name);
                if label.on_hover_text(demo.description).clicked() && !selected {
                    self.demo = demo;
                    self.reload();
                }
            }
        });
    }

    /// the status bar along the bottom of the window
    fn status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.monospace(self.status.as_str());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.monospace(format!("rip {:#x}", self.machine.cpu.registers.rip()));
            });
        });
    }

    /// handles the keyboard shortcuts for the transport buttons
    fn shortcuts(&mut self, ui: &egui::Ui) {
        let (step, run, reset) = ui.input(|input| {
            (
                input.key_pressed(Key::F10),
                input.key_pressed(Key::F5),
                input.key_pressed(Key::F9),
            )
        });
        if step {
            self.step();
        }
        if run {
            self.run();
        }
        if reset {
            self.reload();
        }
    }
}

impl eframe::App for Visuals {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shortcuts(ui);

        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        egui::Panel::bottom("memory")
            .resizable(true)
            .default_size(240.0)
            .min_size(140.0)
            .show(ui, |ui| panels::memory::show(ui, &self.machine));
        egui::Panel::left("registers")
            .resizable(true)
            .default_size(340.0)
            .min_size(240.0)
            .show(ui, |ui| panels::registers::show(ui, &self.machine, &self.before));
        egui::CentralPanel::default().show(ui, |ui| panels::disassembly::show(ui, &self.machine));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emulator::Reg64;

    /// a headless input with a plausible window size
    fn input() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            ..Default::default()
        }
    }

    #[test]
    fn step_records_the_state_it_replaced() {
        let mut app = Visuals::default();
        let rip = app.machine.cpu.registers.rip();

        app.step();

        assert_ne!(app.machine.cpu.registers.rip(), rip);
        assert_eq!(app.before.registers().rip(), rip);
    }

    #[test]
    fn step_highlights_the_register_it_wrote() {
        let mut app = Visuals::default();

        app.step();

        // the first arithmetic instruction is mov rax, 5
        assert_eq!(app.before.registers().get(Reg64::Rax), 0);
        assert_eq!(app.machine.cpu.registers.get(Reg64::Rax), 5);
        assert_eq!(app.before.flags(), app.machine.cpu.flags);
    }

    #[test]
    fn reload_clears_every_highlight() {
        let mut app = Visuals::default();
        app.step();

        app.reload();

        assert_eq!(
            app.before.registers().rip(),
            app.machine.cpu.registers.rip()
        );
        assert_eq!(app.before.flags(), app.machine.cpu.flags);
    }

    #[test]
    fn every_panel_renders_after_a_step() {
        let mut app = Visuals::default();
        app.step();

        let ctx = egui::Context::default();
        let output = ctx.run_ui(input(), |ui| {
            app.toolbar(ui);
            app.status_bar(ui);
            panels::registers::show(ui, &app.machine, &app.before);
            panels::disassembly::show(ui, &app.machine);
            panels::memory::show(ui, &app.machine);
        });
        output.drop_without_applying_deltas();
    }
}
