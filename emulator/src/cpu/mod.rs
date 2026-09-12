//! the CPU state: registers, flags and instruction execution

pub mod execute;
pub mod flags;
pub mod registers;

pub use flags::Flags;
pub use registers::{Reg64, Registers};

use crate::error::Result;
use crate::instruction::Instruction;

/// the processor state: register file, flags and whether it has halted
#[derive(Debug, Clone, Default)]
pub struct Cpu {
    /// general purpose registers and the instruction pointer
    pub registers: Registers,
    /// status flags
    pub flags: Flags,
    /// set once a hlt has executed
    pub halted: bool,
}

impl Cpu {
    /// a reset CPU
    pub fn new() -> Self {
        Self::default()
    }

    /// executes one already-decoded instruction
    pub fn execute(&mut self, instruction: &Instruction) -> Result<()> {
        execute::execute(self, instruction)
    }
}
