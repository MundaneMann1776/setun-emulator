//! # Setun Emulator
//!
//! A balanced ternary emulator of the Soviet Setun (1958) computer.
//!
//! The Setun was the first (and only) balanced ternary computer ever built
//! for practical use. This emulator faithfully recreates its architecture
//! for educational purposes.

pub mod ternary;
pub mod cpu;
pub mod asm;

#[cfg(feature = "tui")]
pub mod tui;

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export commonly used types
pub use ternary::{Trit, Tryte9, Word18};
pub use cpu::{Cpu, CpuState, CpuError, Memory, Registers, Instruction};
pub use asm::{assemble, disassemble, AssemblerError, TromFile, load_trom, save_trom};

#[cfg(feature = "tui")]
pub use tui::run_debugger;
