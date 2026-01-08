//! TUI debugger for Setun emulator.
//!
//! Provides an interactive terminal-based debugger with:
//! - Real-time register visualization
//! - Memory view with trit coloring
//! - Step/run/breakpoint controls
//! - Disassembly view

mod app;
mod ui;

pub use app::{DebuggerApp, run_debugger};
