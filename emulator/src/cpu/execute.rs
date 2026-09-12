//! execution of a single decoded instruction
//!
//! execution assumes the instruction was decoded from the current rip; rip
//! is advanced by the encoded length, or redirected for jumps

use crate::cpu::{Cpu, Flags};
use crate::error::{EmulatorError, Result};
use crate::instruction::{Instruction, Mnemonic, Operand};

/// executes instruction against cpu
pub(crate) fn execute(cpu: &mut Cpu, instruction: &Instruction) -> Result<()> {
    let address = cpu.registers.rip();

    match instruction.mnemonic {
        Mnemonic::Nop => {}
        Mnemonic::Hlt => cpu.halted = true,

        Mnemonic::Mov => {
            let value = read_value(cpu, instruction.src, address)?;
            write_register(cpu, instruction.dst, value, address)?;
        }

        Mnemonic::Add => {
            let lhs = read_value(cpu, instruction.dst, address)?;
            let rhs = read_value(cpu, instruction.src, address)?;
            let result = lhs.wrapping_add(rhs);
            cpu.flags = Flags::from_add(lhs, rhs, result);
            write_register(cpu, instruction.dst, result, address)?;
        }

        Mnemonic::Sub => {
            let lhs = read_value(cpu, instruction.dst, address)?;
            let rhs = read_value(cpu, instruction.src, address)?;
            let result = lhs.wrapping_sub(rhs);
            cpu.flags = Flags::from_sub(lhs, rhs, result);
            write_register(cpu, instruction.dst, result, address)?;
        }

        Mnemonic::Cmp => {
            let lhs = read_value(cpu, instruction.dst, address)?;
            let rhs = read_value(cpu, instruction.src, address)?;
            cpu.flags = Flags::from_sub(lhs, rhs, lhs.wrapping_sub(rhs));
        }

        Mnemonic::Xor => {
            let lhs = read_value(cpu, instruction.dst, address)?;
            let rhs = read_value(cpu, instruction.src, address)?;
            let result = lhs ^ rhs;
            cpu.flags.apply_logic(result);
            write_register(cpu, instruction.dst, result, address)?;
        }

        Mnemonic::Inc => {
            let operand = read_value(cpu, instruction.dst, address)?;
            let result = operand.wrapping_add(1);
            cpu.flags.apply_inc(operand, result);
            write_register(cpu, instruction.dst, result, address)?;
        }

        Mnemonic::Dec => {
            let operand = read_value(cpu, instruction.dst, address)?;
            let result = operand.wrapping_sub(1);
            cpu.flags.apply_dec(operand, result);
            write_register(cpu, instruction.dst, result, address)?;
        }

        Mnemonic::Jmp => return jump(cpu, instruction, address),
        Mnemonic::Je if cpu.flags.zero => return jump(cpu, instruction, address),
        Mnemonic::Jne if !cpu.flags.zero => return jump(cpu, instruction, address),
        Mnemonic::Je | Mnemonic::Jne => {}
    }

    cpu.registers.advance_rip(instruction.length as u64);
    Ok(())
}

/// applies a relative jump: rip = address + length + displacement
fn jump(cpu: &mut Cpu, instruction: &Instruction, address: u64) -> Result<()> {
    let displacement = read_relative(instruction.dst, address)?;
    let target = (address as i64)
        .wrapping_add(instruction.length as i64)
        .wrapping_add(displacement);
    cpu.registers.set_rip(target as u64);
    Ok(())
}

fn read_value(cpu: &Cpu, operand: Option<Operand>, address: u64) -> Result<u64> {
    match operand {
        Some(Operand::Reg(reg)) => Ok(cpu.registers.get(reg)),
        Some(Operand::Imm(value)) => Ok(value),
        _ => Err(EmulatorError::InvalidInstruction { address }),
    }
}

fn write_register(
    cpu: &mut Cpu,
    operand: Option<Operand>,
    value: u64,
    address: u64,
) -> Result<()> {
    match operand {
        Some(Operand::Reg(reg)) => {
            cpu.registers.set(reg, value);
            Ok(())
        }
        _ => Err(EmulatorError::InvalidInstruction { address }),
    }
}

fn read_relative(operand: Option<Operand>, address: u64) -> Result<i64> {
    match operand {
        Some(Operand::Rel(rel)) => Ok(rel),
        _ => Err(EmulatorError::InvalidInstruction { address }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Reg64;

    fn cpu_at(rip: u64) -> Cpu {
        let mut cpu = Cpu::new();
        cpu.registers.set_rip(rip);
        cpu
    }

    #[test]
    fn mov_immediate_writes_register_and_advances() {
        let mut cpu = cpu_at(0x1000);
        let instruction = Instruction::new(
            Mnemonic::Mov,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Imm(42)),
            10,
        );
        cpu.execute(&instruction).unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rax), 42);
        assert_eq!(cpu.registers.rip(), 0x100a);
    }

    #[test]
    fn add_sets_flags_and_result() {
        let mut cpu = cpu_at(0);
        cpu.registers.set(Reg64::Rax, 5);
        cpu.registers.set(Reg64::Rbx, 3);
        let instruction = Instruction::new(
            Mnemonic::Add,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Reg(Reg64::Rbx)),
            3,
        );
        cpu.execute(&instruction).unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rax), 8);
        assert!(!cpu.flags.zero);
        assert!(!cpu.flags.carry);
    }

    #[test]
    fn add_to_max_sets_zero_and_carry() {
        let mut cpu = cpu_at(0);
        cpu.registers.set(Reg64::Rax, u64::MAX);
        let instruction = Instruction::new(
            Mnemonic::Add,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Imm(1)),
            4,
        );
        cpu.execute(&instruction).unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rax), 0);
        assert!(cpu.flags.zero);
        assert!(cpu.flags.carry);
    }

    #[test]
    fn sub_subtracts() {
        let mut cpu = cpu_at(0);
        cpu.registers.set(Reg64::Rax, 10);
        cpu.registers.set(Reg64::Rbx, 4);
        let instruction = Instruction::new(
            Mnemonic::Sub,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Reg(Reg64::Rbx)),
            3,
        );
        cpu.execute(&instruction).unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rax), 6);
    }

    #[test]
    fn cmp_sets_flags_without_writing() {
        let mut cpu = cpu_at(0);
        cpu.registers.set(Reg64::Rax, 7);
        let instruction = Instruction::new(
            Mnemonic::Cmp,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Imm(7)),
            4,
        );
        cpu.execute(&instruction).unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rax), 7);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn xor_of_self_zeroes_and_preserves_rip_advance() {
        let mut cpu = cpu_at(0);
        cpu.registers.set(Reg64::Rax, 0xdead_beef);
        let instruction = Instruction::new(
            Mnemonic::Xor,
            Some(Operand::Reg(Reg64::Rax)),
            Some(Operand::Reg(Reg64::Rax)),
            3,
        );
        cpu.execute(&instruction).unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rax), 0);
        assert!(cpu.flags.zero);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.overflow);
    }

    #[test]
    fn inc_and_dec_leave_carry_alone() {
        let mut cpu = cpu_at(0);
        cpu.flags.carry = true;
        cpu.registers.set(Reg64::Rcx, 2);

        cpu.execute(&Instruction::new(
            Mnemonic::Inc,
            Some(Operand::Reg(Reg64::Rcx)),
            None,
            3,
        ))
        .unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rcx), 3);
        assert!(cpu.flags.carry);

        cpu.execute(&Instruction::new(
            Mnemonic::Dec,
            Some(Operand::Reg(Reg64::Rcx)),
            None,
            3,
        ))
        .unwrap();
        assert_eq!(cpu.registers.get(Reg64::Rcx), 2);
        assert!(cpu.flags.carry);
    }

    #[test]
    fn hlt_sets_halted() {
        let mut cpu = cpu_at(0);
        cpu.execute(&Instruction::new(Mnemonic::Hlt, None, None, 1))
            .unwrap();
        assert!(cpu.halted);
    }

    #[test]
    fn unconditional_jump_redirects_rip() {
        let mut cpu = cpu_at(0x1000);
        cpu.execute(&Instruction::new(
            Mnemonic::Jmp,
            Some(Operand::Rel(-8)),
            None,
            2,
        ))
        .unwrap();
        assert_eq!(cpu.registers.rip(), 0x1000 + 2 - 8);
    }

    #[test]
    fn conditional_jumps_follow_the_zero_flag() {
        let mut taken = cpu_at(0x100);
        taken.flags.zero = true;
        taken
            .execute(&Instruction::new(
                Mnemonic::Je,
                Some(Operand::Rel(0x10)),
                None,
                2,
            ))
            .unwrap();
        assert_eq!(taken.registers.rip(), 0x100 + 2 + 0x10);

        let mut not_taken = cpu_at(0x100);
        not_taken.flags.zero = false;
        not_taken
            .execute(&Instruction::new(
                Mnemonic::Je,
                Some(Operand::Rel(0x10)),
                None,
                2,
            ))
            .unwrap();
        assert_eq!(not_taken.registers.rip(), 0x102);
    }

    #[test]
    fn jne_jumps_when_zero_clear() {
        let mut cpu = cpu_at(0x200);
        cpu.flags.zero = false;
        cpu.execute(&Instruction::new(
            Mnemonic::Jne,
            Some(Operand::Rel(-2)),
            None,
            2,
        ))
        .unwrap();
        assert_eq!(cpu.registers.rip(), 0x200);
    }

    #[test]
    fn writing_to_an_immediate_is_rejected() {
        let mut cpu = cpu_at(0);
        let result = cpu.execute(&Instruction::new(
            Mnemonic::Mov,
            Some(Operand::Imm(1)),
            Some(Operand::Imm(2)),
            1,
        ));
        assert_eq!(result, Err(EmulatorError::InvalidInstruction { address: 0 }));
    }
}
