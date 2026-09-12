//! decoded mnemonics and operands, used by execution and disassembly

use std::fmt;

use crate::cpu::Reg64;

/// an operand of a decoded instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    /// a general purpose register
    Reg(Reg64),
    /// an immediate value, already sign-extended where the encoding says so
    Imm(u64),
    /// a signed displacement relative to the end of the instruction (jumps)
    Rel(i64),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Reg(reg) => write!(f, "{reg}"),
            Operand::Imm(value) => write!(f, "{value:#x}"),
            Operand::Rel(rel) if *rel < 0 => write!(f, "-{:#x}", rel.unsigned_abs()),
            Operand::Rel(rel) => write!(f, "+{rel:#x}"),
        }
    }
}

/// the mnemonic of a decoded instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mnemonic {
    /// no operation
    Nop,
    /// halt the CPU
    Hlt,
    /// move
    Mov,
    /// add
    Add,
    /// subtract
    Sub,
    /// compare (sets flags only)
    Cmp,
    /// increment by one
    Inc,
    /// decrement by one
    Dec,
    /// unconditional jump
    Jmp,
    /// jump if zero flag set
    Je,
    /// jump if zero flag clear
    Jne,
    /// bitwise exclusive or
    Xor,
}

impl Mnemonic {
    /// lower-case mnemonic text
    pub fn name(self) -> &'static str {
        match self {
            Mnemonic::Nop => "nop",
            Mnemonic::Hlt => "hlt",
            Mnemonic::Mov => "mov",
            Mnemonic::Add => "add",
            Mnemonic::Sub => "sub",
            Mnemonic::Cmp => "cmp",
            Mnemonic::Inc => "inc",
            Mnemonic::Dec => "dec",
            Mnemonic::Jmp => "jmp",
            Mnemonic::Je => "je",
            Mnemonic::Jne => "jne",
            Mnemonic::Xor => "xor",
        }
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// a fully decoded instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    /// what the instruction does
    pub mnemonic: Mnemonic,
    /// destination / first operand, if any
    pub dst: Option<Operand>,
    /// source / second operand, if any
    pub src: Option<Operand>,
    /// total encoded length in bytes
    pub length: usize,
}

impl Instruction {
    /// builds a decoded instruction
    pub fn new(
        mnemonic: Mnemonic,
        dst: Option<Operand>,
        src: Option<Operand>,
        length: usize,
    ) -> Self {
        Self {
            mnemonic,
            dst,
            src,
            length,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.mnemonic, self.dst, self.src) {
            (Mnemonic::Nop | Mnemonic::Hlt, ..) => write!(f, "{}", self.mnemonic),
            (mnemonic, Some(dst), Some(src)) => write!(f, "{mnemonic} {dst}, {src}"),
            (mnemonic, Some(dst), None) => write!(f, "{mnemonic} {dst}"),
            (mnemonic, None, _) => write!(f, "{mnemonic}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_two_operand_instructions() {
        let instruction = Instruction::new(
            Mnemonic::Add,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Reg(Reg64::Rbx)),
            3,
        );
        assert_eq!(instruction.to_string(), "add rax, rbx");
    }

    #[test]
    fn displays_immediates_in_hex() {
        let instruction = Instruction::new(
            Mnemonic::Mov,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Imm(5)),
            10,
        );
        assert_eq!(instruction.to_string(), "mov rax, 0x5");
    }

    #[test]
    fn displays_signed_relative_offsets() {
        let forward = Instruction::new(Mnemonic::Jne, Some(Operand::Rel(8)), None, 2);
        assert_eq!(forward.to_string(), "jne +0x8");

        let backward = Instruction::new(Mnemonic::Jmp, Some(Operand::Rel(-8)), None, 2);
        assert_eq!(backward.to_string(), "jmp -0x8");
    }

    #[test]
    fn displays_single_operand_and_nullary() {
        let inc = Instruction::new(Mnemonic::Inc, Some(Operand::Reg(Reg64::Rcx)), None, 3);
        assert_eq!(inc.to_string(), "inc rcx");

        let hlt = Instruction::new(Mnemonic::Hlt, None, None, 1);
        assert_eq!(hlt.to_string(), "hlt");
    }
}
