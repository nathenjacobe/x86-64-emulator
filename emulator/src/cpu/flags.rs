//! the status flags register (rflags) and helpers for updating it
//!
//! only the four flags this emulator actually models are stored: carry, zero,
//! sign and overflow

use std::fmt;

/// the subset of rflags this emulator tracks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    /// carry flag (CF): unsigned overflow / borrow
    pub carry: bool,
    /// zero flag (ZF): result was zero
    pub zero: bool,
    /// sign flag (SF): bit 63 of the result
    pub sign: bool,
    /// overflow flag (OF): signed overflow
    pub overflow: bool,
}

impl Flags {
    /// all flags cleared
    pub const fn new() -> Self {
        Self {
            carry: false,
            zero: false,
            sign: false,
            overflow: false,
        }
    }

    /// flags produced by lhs + rhs == result
    pub fn from_add(lhs: u64, rhs: u64, result: u64) -> Self {
        Self {
            carry: result < lhs,
            zero: result == 0,
            sign: result >> 63 != 0,
            overflow: ((lhs ^ result) & (rhs ^ result)) >> 63 != 0,
        }
    }

    /// flags produced by lhs - rhs == result; also used by cmp, which only
    /// reports the difference through the flags
    pub fn from_sub(lhs: u64, rhs: u64, result: u64) -> Self {
        Self {
            carry: lhs < rhs,
            zero: result == 0,
            sign: result >> 63 != 0,
            overflow: ((lhs ^ rhs) & (lhs ^ result)) >> 63 != 0,
        }
    }

    /// updates the flags for inc, which leaves the carry flag untouched
    pub fn apply_inc(&mut self, operand: u64, result: u64) {
        self.zero = result == 0;
        self.sign = result >> 63 != 0;
        self.overflow = operand == i64::MAX as u64;
    }

    /// updates the flags for dec, which leaves the carry flag untouched
    pub fn apply_dec(&mut self, operand: u64, result: u64) {
        self.zero = result == 0;
        self.sign = result >> 63 != 0;
        self.overflow = operand == i64::MIN as u64;
    }

    /// updates the flags for a bitwise operation (xor): CF and OF cleared,
    /// ZF and SF set from the result
    pub fn apply_logic(&mut self, result: u64) {
        self.carry = false;
        self.overflow = false;
        self.zero = result == 0;
        self.sign = result >> 63 != 0;
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CF={} ZF={} SF={} OF={}",
            self.carry as u8, self.zero as u8, self.sign as u8, self.overflow as u8
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_without_carry() {
        let flags = Flags::from_add(1, 1, 2);
        assert_eq!(
            flags,
            Flags {
                carry: false,
                zero: false,
                sign: false,
                overflow: false
            }
        );
    }

    #[test]
    fn add_sets_carry_and_zero_on_wraparound() {
        let flags = Flags::from_add(u64::MAX, 1, 0);
        assert!(flags.carry);
        assert!(flags.zero);
        assert!(!flags.sign);
        assert!(!flags.overflow);
    }

    #[test]
    fn add_sets_overflow_at_signed_max() {
        let flags = Flags::from_add(i64::MAX as u64, 1, i64::MIN as u64);
        assert!(flags.overflow);
        assert!(flags.sign);
        assert!(!flags.carry);
    }

    #[test]
    fn sub_sets_carry_on_borrow() {
        let flags = Flags::from_sub(0, 1, u64::MAX);
        assert!(flags.carry);
        assert!(flags.sign);
        assert!(!flags.zero);
    }

    #[test]
    fn sub_sets_zero_when_equal() {
        let flags = Flags::from_sub(7, 7, 0);
        assert!(flags.zero);
        assert!(!flags.carry);
        assert!(!flags.sign);
        assert!(!flags.overflow);
    }

    #[test]
    fn sub_sets_overflow_at_signed_min() {
        let flags = Flags::from_sub(i64::MIN as u64, 1, i64::MAX as u64);
        assert!(flags.overflow);
        assert!(!flags.sign);
    }

    #[test]
    fn inc_preserves_carry() {
        let mut flags = Flags {
            carry: true,
            ..Flags::new()
        };
        flags.apply_inc(1, 2);
        assert!(flags.carry, "inc must not touch CF");
        assert!(!flags.zero);
        assert!(!flags.overflow);
    }

    #[test]
    fn inc_sets_overflow_from_signed_max() {
        let mut flags = Flags::new();
        flags.apply_inc(i64::MAX as u64, i64::MIN as u64);
        assert!(flags.overflow);
        assert!(flags.sign);
    }

    #[test]
    fn inc_sets_zero_from_all_ones() {
        let mut flags = Flags::new();
        flags.apply_inc(u64::MAX, 0);
        assert!(flags.zero);
        assert!(!flags.overflow);
    }

    #[test]
    fn dec_preserves_carry() {
        let mut flags = Flags {
            carry: true,
            ..Flags::new()
        };
        flags.apply_dec(2, 1);
        assert!(flags.carry, "dec must not touch CF");
        assert!(!flags.zero);
        assert!(!flags.overflow);
    }

    #[test]
    fn dec_sets_overflow_from_signed_min() {
        let mut flags = Flags::new();
        flags.apply_dec(i64::MIN as u64, i64::MAX as u64);
        assert!(flags.overflow);
        assert!(!flags.sign);
    }

    #[test]
    fn logic_clears_carry_and_overflow() {
        let mut flags = Flags {
            carry: true,
            overflow: true,
            ..Flags::new()
        };
        flags.apply_logic(0);
        assert!(!flags.carry);
        assert!(!flags.overflow);
        assert!(flags.zero);
        assert!(!flags.sign);
    }

    #[test]
    fn display_lists_all_flags() {
        let flags = Flags {
            carry: true,
            zero: false,
            sign: true,
            overflow: false,
        };
        assert_eq!(flags.to_string(), "CF=1 ZF=0 SF=1 OF=0");
    }
}
