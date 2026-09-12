//! a barebones x86-64 emulator core
//!
//! the crate is intentionally split into small modules:
//!
//! - cpu: registers, flags and instruction execution
//! - instruction: decoding mnemonics and ModRM bytes
//! - memory: flat, byte-addressable memory
//! - machine: ties a CPU and memory together into a steppable machine
//! - error: the shared error type used across the crate

pub mod cpu;
pub mod error;
pub mod instruction;
pub mod machine;
pub mod memory;

pub use cpu::{Cpu, Flags, Reg64, Registers};
pub use error::{EmulatorError, Result};
pub use instruction::{Instruction, Mnemonic, Operand};
pub use machine::{Machine, StepInfo};
pub use memory::Memory;
