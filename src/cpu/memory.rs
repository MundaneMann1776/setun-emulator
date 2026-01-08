//! Setun memory subsystem.
//!
//! The original Setun had 162 nine-trit memory cells organized as
//! 3 pages of 54 cells each, with magnetic drum backup.

use crate::ternary::Tryte9;
use serde::{Serialize, Deserialize};

/// The number of memory cells in the Setun.
pub const MEMORY_SIZE: usize = 162;

/// Setun memory: 162 nine-trit cells.
#[derive(Clone, Serialize, Deserialize)]
pub struct Memory {
    cells: Vec<Tryte9>,
}

impl Memory {
    /// Create a new memory with all cells zeroed.
    pub fn new() -> Self {
        Self {
            cells: vec![Tryte9::zero(); MEMORY_SIZE],
        }
    }
    
    /// Read a cell by address (0-161).
    /// 
    /// # Panics
    /// Panics if address is out of range.
    #[inline]
    pub fn read(&self, addr: usize) -> Tryte9 {
        assert!(addr < MEMORY_SIZE, "Memory address {} out of range (0-{})", addr, MEMORY_SIZE - 1);
        self.cells[addr]
    }
    
    /// Write a cell by address (0-161).
    ///
    /// # Panics
    /// Panics if address is out of range.
    #[inline]
    pub fn write(&mut self, addr: usize, value: Tryte9) {
        assert!(addr < MEMORY_SIZE, "Memory address {} out of range (0-{})", addr, MEMORY_SIZE - 1);
        self.cells[addr] = value;
    }
    
    /// Read using a ternary address.
    /// Converts the balanced ternary value to an unsigned index.
    pub fn read_ternary(&self, addr: Tryte9) -> Result<Tryte9, MemoryError> {
        let index = self.addr_to_index(addr)?;
        Ok(self.cells[index])
    }
    
    /// Write using a ternary address.
    pub fn write_ternary(&mut self, addr: Tryte9, value: Tryte9) -> Result<(), MemoryError> {
        let index = self.addr_to_index(addr)?;
        self.cells[index] = value;
        Ok(())
    }
    
    /// Convert a ternary address to a memory index.
    /// 
    /// The Setun used addresses from approximately -81 to +80 (162 values).
    /// We map this to 0-161 by adding 81.
    fn addr_to_index(&self, addr: Tryte9) -> Result<usize, MemoryError> {
        let signed_addr = addr.to_i32();
        // Map balanced ternary range to 0-based index
        // Addresses -81 to +80 map to indices 0 to 161
        let index = (signed_addr + 81) as usize;
        if index >= MEMORY_SIZE {
            return Err(MemoryError::AddressOutOfRange(signed_addr));
        }
        Ok(index)
    }
    
    /// Convert a memory index to a ternary address.
    pub fn index_to_addr(&self, index: usize) -> Tryte9 {
        let signed_addr = (index as i32) - 81;
        Tryte9::from_i32(signed_addr)
    }
    
    /// Clear all memory to zeros.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Tryte9::zero();
        }
    }
    
    /// Load a program into memory starting at the given address.
    pub fn load_program(&mut self, start_addr: usize, program: &[Tryte9]) -> Result<(), MemoryError> {
        if start_addr + program.len() > MEMORY_SIZE {
            return Err(MemoryError::ProgramTooLarge {
                size: program.len(),
                available: MEMORY_SIZE - start_addr,
            });
        }
        
        for (i, &word) in program.iter().enumerate() {
            self.cells[start_addr + i] = word;
        }
        
        Ok(())
    }
    
    /// Dump memory contents (for debugging).
    pub fn dump(&self, start: usize, count: usize) -> Vec<(usize, Tryte9)> {
        let end = (start + count).min(MEMORY_SIZE);
        (start..end)
            .map(|i| (i, self.cells[i]))
            .collect()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Only show non-zero cells
        let non_zero: Vec<_> = self.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| !cell.is_zero())
            .collect();
        
        f.debug_struct("Memory")
            .field("non_zero_cells", &non_zero.len())
            .field("total_cells", &MEMORY_SIZE)
            .finish()
    }
}

/// Errors that can occur during memory operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    /// Address is outside valid memory range.
    AddressOutOfRange(i32),
    /// Program is too large to fit in memory.
    ProgramTooLarge { size: usize, available: usize },
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::AddressOutOfRange(addr) => {
                write!(f, "memory address {} out of range (-81 to +80)", addr)
            }
            MemoryError::ProgramTooLarge { size, available } => {
                write!(f, "program size {} exceeds available space {}", size, available)
            }
        }
    }
}

impl std::error::Error for MemoryError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_read_write() {
        let mut mem = Memory::new();
        let value = Tryte9::from_i32(42);
        
        mem.write(10, value);
        assert_eq!(mem.read(10).to_i32(), 42);
    }
    
    #[test]
    fn test_memory_ternary_addr() {
        let mut mem = Memory::new();
        let value = Tryte9::from_i32(123);
        let addr = Tryte9::from_i32(0); // Middle of memory
        
        mem.write_ternary(addr, value).unwrap();
        assert_eq!(mem.read_ternary(addr).unwrap().to_i32(), 123);
    }
    
    #[test]
    fn test_memory_bounds() {
        let mem = Memory::new();
        
        // Valid addresses: -81 to +80
        assert!(mem.read_ternary(Tryte9::from_i32(-81)).is_ok());
        assert!(mem.read_ternary(Tryte9::from_i32(80)).is_ok());
        
        // Invalid addresses
        assert!(mem.read_ternary(Tryte9::from_i32(-82)).is_err());
        assert!(mem.read_ternary(Tryte9::from_i32(81)).is_err());
    }
    
    #[test]
    fn test_load_program() {
        let mut mem = Memory::new();
        let program = vec![
            Tryte9::from_i32(1),
            Tryte9::from_i32(2),
            Tryte9::from_i32(3),
        ];
        
        mem.load_program(0, &program).unwrap();
        
        assert_eq!(mem.read(0).to_i32(), 1);
        assert_eq!(mem.read(1).to_i32(), 2);
        assert_eq!(mem.read(2).to_i32(), 3);
    }
}
