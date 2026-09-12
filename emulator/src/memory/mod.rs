//! flat, byte-addressable memory with bounds-checked typed accessors
//!
//! every accessor returns EmulatorError::MemoryOutOfBounds instead of
//! panicking when it reaches past the end of the address space

use std::ops::Range;

use crate::error::{EmulatorError, Result};

/// default address space size: 1 MiB
pub const DEFAULT_SIZE: usize = 1024 * 1024;

/// a flat block of memory addressed by u64
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memory {
    bytes: Vec<u8>,
}

impl Memory {
    /// creates a zeroed block of size bytes
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
    }

    /// number of addressable bytes
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// true when the address space is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// copies data into memory starting at address
    pub fn load(&mut self, address: u64, data: &[u8]) -> Result<()> {
        let range = self.range(address, data.len())?;
        self.bytes[range].copy_from_slice(data);
        Ok(())
    }

    /// borrows len bytes starting at address
    pub fn read_bytes(&self, address: u64, len: usize) -> Result<&[u8]> {
        let range = self.range(address, len)?;
        Ok(&self.bytes[range])
    }

    /// reads a single byte
    pub fn read_u8(&self, address: u64) -> Result<u8> {
        Ok(self.read_bytes(address, 1)?[0])
    }

    /// reads a little-endian u16
    pub fn read_u16(&self, address: u64) -> Result<u16> {
        let bytes = self.read_bytes(address, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// reads a little-endian u32
    pub fn read_u32(&self, address: u64) -> Result<u32> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// reads a little-endian u64
    pub fn read_u64(&self, address: u64) -> Result<u64> {
        let bytes = self.read_bytes(address, 8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// writes a single byte
    pub fn write_u8(&mut self, address: u64, value: u8) -> Result<()> {
        let range = self.range(address, 1)?;
        self.bytes[range][0] = value;
        Ok(())
    }

    /// writes a little-endian u16
    pub fn write_u16(&mut self, address: u64, value: u16) -> Result<()> {
        let range = self.range(address, 2)?;
        self.bytes[range].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// writes a little-endian u32
    pub fn write_u32(&mut self, address: u64, value: u32) -> Result<()> {
        let range = self.range(address, 4)?;
        self.bytes[range].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// writes a little-endian u64
    pub fn write_u64(&mut self, address: u64, value: u64) -> Result<()> {
        let range = self.range(address, 8)?;
        self.bytes[range].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// validates address..address + size and returns it as a usable range
    fn range(&self, address: u64, size: usize) -> Result<Range<usize>> {
        let end = address
            .checked_add(size as u64)
            .ok_or(EmulatorError::MemoryOutOfBounds { address, size })?;
        if end > self.bytes.len() as u64 {
            return Err(EmulatorError::MemoryOutOfBounds { address, size });
        }
        Ok(address as usize..end as usize)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new(DEFAULT_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_memory_is_zeroed() {
        let memory = Memory::new(16);
        assert_eq!(memory.len(), 16);
        assert!(!memory.is_empty());
        for address in 0..16 {
            assert_eq!(memory.read_u8(address).unwrap(), 0);
        }
    }

    #[test]
    fn default_uses_default_size() {
        assert_eq!(Memory::default().len(), DEFAULT_SIZE);
    }

    #[test]
    fn load_then_read_back() {
        let mut memory = Memory::new(16);
        memory.load(4, &[1, 2, 3]).unwrap();
        assert_eq!(memory.read_bytes(4, 3).unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn typed_accessors_are_little_endian() {
        let mut memory = Memory::new(16);
        memory.write_u32(0, 0xdead_beef).unwrap();
        assert_eq!(memory.read_bytes(0, 4).unwrap(), &[0xef, 0xbe, 0xad, 0xde]);
        assert_eq!(memory.read_u32(0).unwrap(), 0xdead_beef);
    }

    #[test]
    fn typed_accessors_round_trip() {
        let mut memory = Memory::new(32);
        memory.write_u8(0, 0xab).unwrap();
        memory.write_u16(1, 0x1234).unwrap();
        memory.write_u32(3, 0x89ab_cdef).unwrap();
        memory.write_u64(8, 0x0123_4567_89ab_cdef).unwrap();

        assert_eq!(memory.read_u8(0).unwrap(), 0xab);
        assert_eq!(memory.read_u16(1).unwrap(), 0x1234);
        assert_eq!(memory.read_u32(3).unwrap(), 0x89ab_cdef);
        assert_eq!(memory.read_u64(8).unwrap(), 0x0123_4567_89ab_cdef);
    }

    #[test]
    fn read_past_the_end_fails() {
        let memory = Memory::new(4);
        assert_eq!(
            memory.read_u8(4),
            Err(EmulatorError::MemoryOutOfBounds {
                address: 4,
                size: 1
            })
        );
        assert_eq!(
            memory.read_u32(2),
            Err(EmulatorError::MemoryOutOfBounds {
                address: 2,
                size: 4
            })
        );
    }

    #[test]
    fn write_past_the_end_fails() {
        let mut memory = Memory::new(4);
        assert_eq!(
            memory.write_u32(1, 0),
            Err(EmulatorError::MemoryOutOfBounds {
                address: 1,
                size: 4
            })
        );
    }

    #[test]
    fn load_past_the_end_fails() {
        let mut memory = Memory::new(4);
        assert!(memory.load(3, &[1, 2]).is_err());
    }

    #[test]
    fn overflowing_address_fails_instead_of_panicking() {
        let mut memory = Memory::new(4);
        assert!(memory.read_u64(u64::MAX).is_err());
        assert!(memory.write_u8(u64::MAX, 0).is_err());
    }

    #[test]
    fn zero_length_access_at_the_end_succeeds() {
        let memory = Memory::new(4);
        assert_eq!(memory.read_bytes(4, 0).unwrap(), &[] as &[u8]);
    }
}
