//! CPU emulation for the Setun computer.
//!
//! This module implements the complete Setun (1958) architecture:
//! - 162 nine-trit memory cells
//! - 5 registers: S (accumulator), R (multiplier), F (index), C (PC), ω (sign)
//! - 24-instruction set with single-address architecture

pub mod memory;
pub mod registers;
pub mod decode;
pub mod execute;

pub use memory::Memory;
pub use registers::Registers;
pub use decode::{Instruction, AddrMode, DecodeError};
pub use execute::{Cpu, CpuError, CpuState};
