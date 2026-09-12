//! fetches and decodes the next instruction from memory
//!
//! only the 64-bit register forms of a small instruction subset are supported;
//! anything else (memory operands, the extended r8..r15 registers, opcodes
//! outside the table) decodes to EmulatorError::UnimplementedOpcode

use crate::cpu::Reg64;
use crate::error::{EmulatorError, Result};
use crate::instruction::mnemonic::{Instruction, Mnemonic, Operand};
use crate::instruction::modrm::ModRm;
use crate::memory::Memory;

/// decodes the instruction located at address
pub fn decode(memory: &Memory, address: u64) -> Result<Instruction> {
    let mut cursor = Cursor::new(memory, address);
    let first = cursor.read_u8()?;

    let (rex, opcode) = if is_rex(first) {
        (Some(Rex::from_byte(first)), cursor.read_u8()?)
    } else {
        (None, first)
    };

    // only the eight base registers are modelled, so the REX bits that select
    // an extended register cannot be honoured
    if rex.is_some_and(|rex| rex.r || rex.b) {
        return Err(EmulatorError::UnimplementedOpcode { opcode, address });
    }
    let wide = rex.is_some_and(|rex| rex.w);

    match opcode {
        0x90 => Ok(finish(Mnemonic::Nop, None, None, &cursor)),
        0xf4 => Ok(finish(Mnemonic::Hlt, None, None, &cursor)),

        // mov r32/r64, imm
        0xb8..=0xbf => {
            let reg = Reg64::from_index(opcode - 0xb8).expect("opcode maps to a register");
            let value = if wide {
                cursor.read_u64()?
            } else {
                cursor.read_u32()? as u64
            };
            Ok(finish(
                Mnemonic::Mov,
                Some(Operand::Reg(reg)),
                Some(Operand::Imm(value)),
                &cursor,
            ))
        }

        // reg field is the destination, rm field the source
        0x8b => decode_reg_dst(&mut cursor, opcode, address, wide, Mnemonic::Mov),
        0x03 => decode_reg_dst(&mut cursor, opcode, address, wide, Mnemonic::Add),
        0x2b => decode_reg_dst(&mut cursor, opcode, address, wide, Mnemonic::Sub),
        0x3b => decode_reg_dst(&mut cursor, opcode, address, wide, Mnemonic::Cmp),
        0x33 => decode_reg_dst(&mut cursor, opcode, address, wide, Mnemonic::Xor),

        // rm field is the destination, reg field the source
        0x89 => decode_rm_dst(&mut cursor, opcode, address, wide, Mnemonic::Mov),
        0x01 => decode_rm_dst(&mut cursor, opcode, address, wide, Mnemonic::Add),
        0x29 => decode_rm_dst(&mut cursor, opcode, address, wide, Mnemonic::Sub),
        0x39 => decode_rm_dst(&mut cursor, opcode, address, wide, Mnemonic::Cmp),
        0x31 => decode_rm_dst(&mut cursor, opcode, address, wide, Mnemonic::Xor),

        0x83 => decode_group1(&mut cursor, opcode, address, wide, ImmSize::Byte),
        0x81 => decode_group1(&mut cursor, opcode, address, wide, ImmSize::Dword),
        0xff => decode_group5(&mut cursor, opcode, address, wide),

        0xeb => {
            let rel = cursor.read_u8()? as i8 as i64;
            Ok(finish(Mnemonic::Jmp, Some(Operand::Rel(rel)), None, &cursor))
        }
        0xe9 => {
            let rel = cursor.read_u32()? as i32 as i64;
            Ok(finish(Mnemonic::Jmp, Some(Operand::Rel(rel)), None, &cursor))
        }
        0x74 => {
            let rel = cursor.read_u8()? as i8 as i64;
            Ok(finish(Mnemonic::Je, Some(Operand::Rel(rel)), None, &cursor))
        }
        0x75 => {
            let rel = cursor.read_u8()? as i8 as i64;
            Ok(finish(Mnemonic::Jne, Some(Operand::Rel(rel)), None, &cursor))
        }

        _ => Err(EmulatorError::UnimplementedOpcode { opcode, address }),
    }
}

/// width of the immediate that follows a group-1 opcode
#[derive(Debug, Clone, Copy)]
enum ImmSize {
    /// 0x83, a sign-extended byte
    Byte,
    /// 0x81, a sign-extended dword
    Dword,
}

fn decode_reg_dst(
    cursor: &mut Cursor,
    opcode: u8,
    address: u64,
    wide: bool,
    mnemonic: Mnemonic,
) -> Result<Instruction> {
    let modrm = read_register_modrm(cursor, opcode, address, wide)?;
    Ok(finish(
        mnemonic,
        Some(Operand::Reg(modrm.reg_register().expect("reg field"))),
        Some(Operand::Reg(modrm.rm_register().expect("rm field"))),
        cursor,
    ))
}

fn decode_rm_dst(
    cursor: &mut Cursor,
    opcode: u8,
    address: u64,
    wide: bool,
    mnemonic: Mnemonic,
) -> Result<Instruction> {
    let modrm = read_register_modrm(cursor, opcode, address, wide)?;
    Ok(finish(
        mnemonic,
        Some(Operand::Reg(modrm.rm_register().expect("rm field"))),
        Some(Operand::Reg(modrm.reg_register().expect("reg field"))),
        cursor,
    ))
}

fn decode_group1(
    cursor: &mut Cursor,
    opcode: u8,
    address: u64,
    wide: bool,
    imm_size: ImmSize,
) -> Result<Instruction> {
    let modrm = read_register_modrm(cursor, opcode, address, wide)?;
    let mnemonic = match modrm.reg {
        0 => Mnemonic::Add,
        5 => Mnemonic::Sub,
        7 => Mnemonic::Cmp,
        _ => return Err(EmulatorError::UnimplementedOpcode { opcode, address }),
    };
    let value = match imm_size {
        ImmSize::Byte => cursor.read_u8()? as i8 as i64 as u64,
        ImmSize::Dword => cursor.read_u32()? as i32 as i64 as u64,
    };
    Ok(finish(
        mnemonic,
        Some(Operand::Reg(modrm.rm_register().expect("rm field"))),
        Some(Operand::Imm(value)),
        cursor,
    ))
}

fn decode_group5(
    cursor: &mut Cursor,
    opcode: u8,
    address: u64,
    wide: bool,
) -> Result<Instruction> {
    let modrm = read_register_modrm(cursor, opcode, address, wide)?;
    let mnemonic = match modrm.reg {
        0 => Mnemonic::Inc,
        1 => Mnemonic::Dec,
        _ => return Err(EmulatorError::UnimplementedOpcode { opcode, address }),
    };
    Ok(finish(
        mnemonic,
        Some(Operand::Reg(modrm.rm_register().expect("rm field"))),
        None,
        cursor,
    ))
}

/// reads a ModRM byte, requiring a 64-bit register-direct operand
fn read_register_modrm(
    cursor: &mut Cursor,
    opcode: u8,
    address: u64,
    wide: bool,
) -> Result<ModRm> {
    if !wide {
        // only the 64-bit operand size is modelled
        return Err(EmulatorError::UnimplementedOpcode { opcode, address });
    }
    let modrm = ModRm::decode(cursor.read_u8()?);
    if !modrm.is_register_direct() {
        return Err(EmulatorError::UnimplementedOpcode { opcode, address });
    }
    Ok(modrm)
}

fn finish(
    mnemonic: Mnemonic,
    dst: Option<Operand>,
    src: Option<Operand>,
    cursor: &Cursor,
) -> Instruction {
    Instruction::new(mnemonic, dst, src, cursor.offset)
}

fn is_rex(byte: u8) -> bool {
    matches!(byte, 0x40..=0x4f)
}

/// the REX prefix bits this decoder cares about
#[derive(Debug, Clone, Copy)]
struct Rex {
    w: bool,
    r: bool,
    b: bool,
}

impl Rex {
    fn from_byte(byte: u8) -> Self {
        Self {
            w: byte & 0x08 != 0,
            r: byte & 0x04 != 0,
            b: byte & 0x01 != 0,
        }
    }
}

/// sequential reader over memory that tracks the instruction length
struct Cursor<'a> {
    memory: &'a Memory,
    address: u64,
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(memory: &'a Memory, address: u64) -> Self {
        Self {
            memory,
            address,
            offset: 0,
        }
    }

    /// address of the next field, or an out-of-bounds error on overflow
    fn at(&self, size: usize) -> Result<u64> {
        self.address
            .checked_add(self.offset as u64)
            .ok_or(EmulatorError::MemoryOutOfBounds {
                address: self.address,
                size,
            })
    }

    fn read_u8(&mut self) -> Result<u8> {
        let value = self.memory.read_u8(self.at(1)?)?;
        self.offset += 1;
        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32> {
        let value = self.memory.read_u32(self.at(4)?)?;
        self.offset += 4;
        Ok(value)
    }

    fn read_u64(&mut self) -> Result<u64> {
        let value = self.memory.read_u64(self.at(8)?)?;
        self.offset += 8;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_bytes(bytes: &[u8]) -> Result<Instruction> {
        let mut memory = Memory::new(64);
        memory.load(0, bytes).unwrap();
        decode(&memory, 0)
    }

    #[test]
    fn decodes_nop_and_hlt() {
        assert_eq!(
            decode_bytes(&[0x90]).unwrap(),
            Instruction::new(Mnemonic::Nop, None, None, 1)
        );
        assert_eq!(
            decode_bytes(&[0xf4]).unwrap(),
            Instruction::new(Mnemonic::Hlt, None, None, 1)
        );
    }

    #[test]
    fn decodes_mov_imm64_and_records_length() {
        let bytes = [0x48, 0xb8, 0x05, 0, 0, 0, 0, 0, 0, 0];
        let instruction = decode_bytes(&bytes).unwrap();
        assert_eq!(
            instruction,
            Instruction::new(
                Mnemonic::Mov,
                Some(Operand::Reg(Reg64::Rax)),
                Some(Operand::Imm(5)),
                10
            )
        );
    }

    #[test]
    fn decodes_mov_imm32_without_rex_and_zero_extends() {
        let bytes = [0xb9, 0xff, 0xff, 0xff, 0xff];
        let instruction = decode_bytes(&bytes).unwrap();
        assert_eq!(
            instruction,
            Instruction::new(
                Mnemonic::Mov,
                Some(Operand::Reg(Reg64::Rcx)),
                Some(Operand::Imm(0xffff_ffff)),
                5
            )
        );
    }

    #[test]
    fn decodes_reg_register_forms_in_both_directions() {
        // add rax, rbx  (0x03 /r)
        assert_eq!(
            decode_bytes(&[0x48, 0x03, 0xc3]).unwrap(),
            Instruction::new(
                Mnemonic::Add,
                Some(Operand::Reg(Reg64::Rax)),
                Some(Operand::Reg(Reg64::Rbx)),
                3
            )
        );
        // add rbx, rax  (0x01 /r)
        assert_eq!(
            decode_bytes(&[0x48, 0x01, 0xc3]).unwrap(),
            Instruction::new(
                Mnemonic::Add,
                Some(Operand::Reg(Reg64::Rbx)),
                Some(Operand::Reg(Reg64::Rax)),
                3
            )
        );
    }

    #[test]
    fn decodes_group1_add_and_sub_and_cmp() {
        // add rax, 3
        assert_eq!(
            decode_bytes(&[0x48, 0x83, 0xc0, 0x03]).unwrap(),
            Instruction::new(
                Mnemonic::Add,
                Some(Operand::Reg(Reg64::Rax)),
                Some(Operand::Imm(3)),
                4
            )
        );
        // sub rax, 3
        assert_eq!(
            decode_bytes(&[0x48, 0x83, 0xe8, 0x03]).unwrap(),
            Instruction::new(
                Mnemonic::Sub,
                Some(Operand::Reg(Reg64::Rax)),
                Some(Operand::Imm(3)),
                4
            )
        );
        // cmp rax, 1
        assert_eq!(
            decode_bytes(&[0x48, 0x83, 0xf8, 0x01]).unwrap(),
            Instruction::new(
                Mnemonic::Cmp,
                Some(Operand::Reg(Reg64::Rax)),
                Some(Operand::Imm(1)),
                4
            )
        );
    }

    #[test]
    fn sign_extends_group1_immediates() {
        // sub rax, -1
        let instruction = decode_bytes(&[0x48, 0x83, 0xe8, 0xff]).unwrap();
        assert_eq!(instruction.src, Some(Operand::Imm(u64::MAX)));
    }

    #[test]
    fn decodes_inc_and_dec() {
        assert_eq!(
            decode_bytes(&[0x48, 0xff, 0xc0]).unwrap(),
            Instruction::new(Mnemonic::Inc, Some(Operand::Reg(Reg64::Rax)), None, 3)
        );
        assert_eq!(
            decode_bytes(&[0x48, 0xff, 0xc9]).unwrap(),
            Instruction::new(Mnemonic::Dec, Some(Operand::Reg(Reg64::Rcx)), None, 3)
        );
    }

    #[test]
    fn decodes_xor_for_register_zeroing() {
        assert_eq!(
            decode_bytes(&[0x48, 0x31, 0xc0]).unwrap(),
            Instruction::new(
                Mnemonic::Xor,
                Some(Operand::Reg(Reg64::Rax)),
                Some(Operand::Reg(Reg64::Rax)),
                3
            )
        );
    }

    #[test]
    fn decodes_relative_jumps() {
        assert_eq!(
            decode_bytes(&[0x75, 0xf8]).unwrap(),
            Instruction::new(Mnemonic::Jne, Some(Operand::Rel(-8)), None, 2)
        );
        assert_eq!(
            decode_bytes(&[0x74, 0x0a]).unwrap(),
            Instruction::new(Mnemonic::Je, Some(Operand::Rel(10)), None, 2)
        );
        assert_eq!(
            decode_bytes(&[0xe9, 0x00, 0x01, 0x00, 0x00]).unwrap(),
            Instruction::new(Mnemonic::Jmp, Some(Operand::Rel(0x100)), None, 5)
        );
    }

    #[test]
    fn rejects_memory_operands() {
        // add [rax], rbx -> modrm 0x18 is not register-direct
        assert_eq!(
            decode_bytes(&[0x48, 0x01, 0x18]),
            Err(EmulatorError::UnimplementedOpcode {
                opcode: 0x01,
                address: 0
            })
        );
    }

    #[test]
    fn rejects_extended_registers() {
        // the REX.B bit selects r8..r15, which we do not model
        assert_eq!(
            decode_bytes(&[0x49, 0x01, 0xc0]),
            Err(EmulatorError::UnimplementedOpcode {
                opcode: 0x01,
                address: 0
            })
        );
    }

    #[test]
    fn rejects_32_bit_alu_forms() {
        // 32-bit add is not modelled
        assert_eq!(
            decode_bytes(&[0x01, 0xc0]),
            Err(EmulatorError::UnimplementedOpcode {
                opcode: 0x01,
                address: 0
            })
        );
    }

    #[test]
    fn rejects_unknown_opcodes() {
        assert_eq!(
            decode_bytes(&[0x0f]),
            Err(EmulatorError::UnimplementedOpcode {
                opcode: 0x0f,
                address: 0
            })
        );
    }

    #[test]
    fn truncation_at_the_end_of_memory_is_an_error() {
        // an empty address space cannot even supply the opcode byte
        assert!(matches!(
            decode(&Memory::new(0), 0),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));

        // mov needs four immediate bytes that are not present
        let mut memory = Memory::new(1);
        memory.load(0, &[0xb8]).unwrap();
        assert!(matches!(
            decode(&memory, 0),
            Err(EmulatorError::MemoryOutOfBounds { .. })
        ));
    }

    #[test]
    fn decodes_at_a_nonzero_address() {
        let mut memory = Memory::new(64);
        memory.load(0x20, &[0x48, 0xff, 0xc0]).unwrap();
        assert_eq!(
            decode(&memory, 0x20).unwrap(),
            Instruction::new(Mnemonic::Inc, Some(Operand::Reg(Reg64::Rax)), None, 3)
        );
    }
}
