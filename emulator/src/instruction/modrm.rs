//! decoding a ModRM byte
//!
//! only register-direct operands (mod == 0b11) are modelled for now; the
//! decoder rejects anything else before it reaches this type

use crate::cpu::Reg64;

/// a decoded ModRM byte
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModRm {
    /// addressing mode (top two bits)
    pub mode: u8,
    /// register / opcode-extension field (middle three bits)
    pub reg: u8,
    /// register / memory field (low three bits)
    pub rm: u8,
}

impl ModRm {
    /// splits a ModRM byte into its three fields
    pub fn decode(byte: u8) -> Self {
        Self {
            mode: byte >> 6,
            reg: (byte >> 3) & 0b111,
            rm: byte & 0b111,
        }
    }

    /// true when both operands are registers
    pub fn is_register_direct(&self) -> bool {
        self.mode == 0b11
    }

    /// the register encoded in the reg field
    pub fn reg_register(&self) -> Option<Reg64> {
        Reg64::from_index(self.reg)
    }

    /// the register encoded in the rm field
    pub fn rm_register(&self) -> Option<Reg64> {
        Reg64::from_index(self.rm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_three_fields() {
        // 11 001 010 -> mod=3, reg=1 (rcx), rm=2 (rdx)
        let modrm = ModRm::decode(0b1100_1010);
        assert_eq!(modrm.mode, 0b11);
        assert_eq!(modrm.reg, 1);
        assert_eq!(modrm.rm, 2);
        assert_eq!(modrm.reg_register(), Some(Reg64::Rcx));
        assert_eq!(modrm.rm_register(), Some(Reg64::Rdx));
    }

    #[test]
    fn recognises_register_direct_mode() {
        assert!(ModRm::decode(0b1100_0000).is_register_direct());
        assert!(!ModRm::decode(0b0000_0000).is_register_direct());
        assert!(!ModRm::decode(0b0100_0000).is_register_direct());
    }

    #[test]
    fn reg_register_never_none_for_three_bit_field() {
        for byte in 0u8..=0xff {
            let modrm = ModRm::decode(byte);
            assert!(modrm.reg_register().is_some());
            assert!(modrm.rm_register().is_some());
        }
    }
}
