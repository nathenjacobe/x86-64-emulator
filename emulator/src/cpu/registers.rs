//! general purpose registers and the instruction pointer

use std::fmt;

/// the eight general purpose registers addressable by the base encoding
///
/// the variants are listed in encoding order (0..=7), matching the reg/rm
/// fields of a ModRM byte
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reg64 {
    /// accumulator
    Rax,
    /// counter
    Rcx,
    /// data
    Rdx,
    /// base
    Rbx,
    /// stack pointer
    Rsp,
    /// base pointer
    Rbp,
    /// source index
    Rsi,
    /// destination index
    Rdi,
}

impl Reg64 {
    /// all registers, indexed by their encoding
    pub const ALL: [Reg64; 8] = [
        Reg64::Rax,
        Reg64::Rcx,
        Reg64::Rdx,
        Reg64::Rbx,
        Reg64::Rsp,
        Reg64::Rbp,
        Reg64::Rsi,
        Reg64::Rdi,
    ];

    /// maps a 3-bit encoding field to a register
    pub fn from_index(index: u8) -> Option<Self> {
        Self::ALL.get(index as usize).copied()
    }

    /// the 3-bit encoding index of this register
    pub fn index(self) -> u8 {
        self as u8
    }

    /// lower-case AT&T-style name, e.g. "rax"
    pub fn name(self) -> &'static str {
        match self {
            Reg64::Rax => "rax",
            Reg64::Rcx => "rcx",
            Reg64::Rdx => "rdx",
            Reg64::Rbx => "rbx",
            Reg64::Rsp => "rsp",
            Reg64::Rbp => "rbp",
            Reg64::Rsi => "rsi",
            Reg64::Rdi => "rdi",
        }
    }
}

impl fmt::Display for Reg64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// the register file: eight 64-bit values plus the instruction pointer
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Registers {
    values: [u64; 8],
    rip: u64,
}

impl Registers {
    /// a zeroed register file
    pub fn new() -> Self {
        Self::default()
    }

    /// reads a register
    pub fn get(&self, reg: Reg64) -> u64 {
        self.values[reg.index() as usize]
    }

    /// writes a register
    pub fn set(&mut self, reg: Reg64, value: u64) {
        self.values[reg.index() as usize] = value;
    }

    /// the instruction pointer
    pub fn rip(&self) -> u64 {
        self.rip
    }

    /// sets the instruction pointer
    pub fn set_rip(&mut self, value: u64) {
        self.rip = value;
    }

    /// moves the instruction pointer forward by delta bytes
    pub fn advance_rip(&mut self, delta: u64) {
        self.rip = self.rip.wrapping_add(delta);
    }

    /// iterates over every register and its value, in encoding order
    pub fn iter(&self) -> impl Iterator<Item = (Reg64, u64)> + '_ {
        Reg64::ALL.into_iter().map(move |reg| (reg, self.get(reg)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_round_trips_through_from_index() {
        for (index, reg) in Reg64::ALL.into_iter().enumerate() {
            assert_eq!(reg.index(), index as u8);
            assert_eq!(Reg64::from_index(index as u8), Some(reg));
        }
    }

    #[test]
    fn from_index_rejects_out_of_range() {
        assert_eq!(Reg64::from_index(8), None);
        assert_eq!(Reg64::from_index(255), None);
    }

    #[test]
    fn registers_start_zeroed() {
        let registers = Registers::new();
        for (_, value) in registers.iter() {
            assert_eq!(value, 0);
        }
        assert_eq!(registers.rip(), 0);
    }

    #[test]
    fn set_and_get_are_independent_per_register() {
        let mut registers = Registers::new();
        registers.set(Reg64::Rax, 0x1111);
        registers.set(Reg64::Rcx, 0x2222);
        assert_eq!(registers.get(Reg64::Rax), 0x1111);
        assert_eq!(registers.get(Reg64::Rcx), 0x2222);
        assert_eq!(registers.get(Reg64::Rdx), 0);
    }

    #[test]
    fn rip_advances() {
        let mut registers = Registers::new();
        registers.set_rip(0x10);
        registers.advance_rip(4);
        assert_eq!(registers.rip(), 0x14);
        registers.set_rip(0);
        assert_eq!(registers.rip(), 0);
    }

    #[test]
    fn display_uses_lower_case_names() {
        assert_eq!(Reg64::Rax.to_string(), "rax");
        assert_eq!(Reg64::Rdi.to_string(), "rdi");
    }
}
