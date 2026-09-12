//! instruction decoding: mnemonics, ModRM and the decoder itself

pub mod decode;
pub mod mnemonic;
pub mod modrm;

pub use decode::decode;
pub use mnemonic::{Instruction, Mnemonic, Operand};
pub use modrm::ModRm;
