//! error type shared by every part of the emulator

use std::fmt;

/// errors that can occur while decoding or executing code
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmulatorError {
    /// a memory access fell outside the address space
    MemoryOutOfBounds {
        /// first address of the access
        address: u64,
        /// number of bytes the access tried to touch
        size: usize,
    },
    /// the opcode is not implemented by this emulator
    UnimplementedOpcode {
        /// the offending opcode byte
        opcode: u8,
        /// address the opcode was decoded from
        address: u64,
    },
    /// a decoded instruction is malformed for execution (e.g. a write to an
    /// immediate operand); reaching this indicates a decoder bug
    InvalidInstruction {
        /// address of the instruction
        address: u64,
    },
    /// the CPU is halted and cannot execute further instructions
    Halted,
    /// crate::Machine::run hit its step budget before the CPU halted
    StepLimitReached {
        /// the budget that was exhausted
        limit: u64,
    },
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmulatorError::MemoryOutOfBounds { address, size } => write!(
                f,
                "memory access of {size} byte(s) at {address:#x} is out of bounds"
            ),
            EmulatorError::UnimplementedOpcode { opcode, address } => {
                write!(f, "unimplemented opcode {opcode:#04x} at {address:#x}")
            }
            EmulatorError::InvalidInstruction { address } => {
                write!(f, "invalid instruction at {address:#x}")
            }
            EmulatorError::Halted => write!(f, "the CPU is halted"),
            EmulatorError::StepLimitReached { limit } => {
                write!(f, "step limit of {limit} reached before halt")
            }
        }
    }
}

impl std::error::Error for EmulatorError {}

/// convenience alias used across the crate
pub type Result<T> = std::result::Result<T, EmulatorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_mentions_relevant_details() {
        let err = EmulatorError::MemoryOutOfBounds {
            address: 0x10,
            size: 4,
        };
        assert_eq!(err.to_string(), "memory access of 4 byte(s) at 0x10 is out of bounds");

        let err = EmulatorError::UnimplementedOpcode {
            opcode: 0x0f,
            address: 0xff,
        };
        assert_eq!(err.to_string(), "unimplemented opcode 0x0f at 0xff");

        assert_eq!(EmulatorError::Halted.to_string(), "the CPU is halted");
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<EmulatorError>();
    }
}
