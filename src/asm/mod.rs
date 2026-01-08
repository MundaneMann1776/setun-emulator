//! Assembler and disassembler for Setun programs.
//!
//! This module provides:
//! - A simple two-pass assembler (text → TROM binary format)
//! - A disassembler (TROM → readable text)

pub mod assembler;
pub mod disasm;
pub mod trom;

pub use assembler::{assemble, AssemblerError};
pub use disasm::disassemble;
pub use trom::{TromFile, load_trom, save_trom};
