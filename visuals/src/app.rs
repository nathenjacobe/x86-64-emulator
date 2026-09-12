//! the eframe application: owns the machine and drives the update loop

use demos::Demo;
use eframe::egui;
use emulator::Machine;

use crate::panels;

/// maximum steps a single "Run" click will execute before giving up
const RUN_STEP_LIMIT: u64 = 10_000;

/// top-level UI state
pub struct Visuals {
    machine: Machine,
    demo: &'static Demo,
    status: String,
}

impl Default for Visuals {
    fn default() -> Self {
        let demo = demos::default_demo();
        Self {
            machine: demos::machine(demo),
            demo,
            status: format!("loaded {}", demo.name),
        }
    }
}

impl Visuals {
    /// reloads the currently selected demo from scratch
    fn reload(&mut self) {
        self.machine = demos::machine(self.demo);
        self.status = format!("reset {}", self.demo.name);
    }

    /// executes a single instruction
    fn step(&mut self) {
        match self.machine.step() {
            Ok(info) => self.status = format!("{:#x}: {}", info.address, info.instruction),
            Err(err) => self.status = err.to_string(),
        }
    }

    /// runs until halt or the step limit
    fn run(&mut self) {
        match self.machine.run(RUN_STEP_LIMIT) {
            Ok(steps) => self.status = format!("ran {steps} step(s)"),
            Err(err) => self.status = err.to_string(),
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Step").clicked() {
                self.step();
            }
            if ui.button("Run").clicked() {
                self.run();
            }
            if ui.button("Reset").clicked() {
                self.reload();
            }
            ui.label(&self.status);
        });

        ui.horizontal(|ui| {
            ui.label("demo:");
            for demo in demos::demos() {
                let selected = std::ptr::eq(demo, self.demo);
                if ui.selectable_label(selected, demo.name).clicked() && !selected {
                    self.demo = demo;
                    self.reload();
                }
            }
        });
    }
}

impl eframe::App for Visuals {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            self.controls(ui);
            ui.separator();
            panels::registers::show(ui, &self.machine);
            ui.separator();
            panels::disassembly::show(ui, &self.machine);
            ui.separator();
            panels::memory::show(ui, &self.machine);
        });
    }
}
