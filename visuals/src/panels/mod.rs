//! individual ui panels

use emulator::{Flags, Machine, Registers};

pub mod disassembly;
pub mod memory;
pub mod registers;

/// a snapshot of the machine state taken before a step, so the panels can
/// highlight exactly what that step changed
#[derive(Debug, Clone, Default)]
pub struct Before {
    registers: Registers,
    flags: Flags,
}

impl Before {
    /// captures the current register file and flags
    pub fn capture(machine: &Machine) -> Self {
        Self {
            registers: machine.cpu.registers.clone(),
            flags: machine.cpu.flags,
        }
    }

    /// the register file as it was before the last step
    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    /// the flags as they were before the last step
    pub fn flags(&self) -> Flags {
        self.flags
    }
}
