//! WebAssembly bindings for the Setun emulator.
//!
//! This module provides JavaScript-friendly wrappers around the core emulator.

use wasm_bindgen::prelude::*;
use crate::{Cpu, Tryte9, CpuState};
use crate::asm::assembler::assemble;
use crate::asm::disasm::disassemble_instruction;
use crate::cpu::decode::encode;

/// Initialize panic hook for better error messages in console.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WebAssembly-friendly CPU wrapper.
#[wasm_bindgen]
pub struct WasmCpu {
    cpu: Cpu,
    program: Vec<Tryte9>,
}

#[wasm_bindgen]
impl WasmCpu {
    /// Create a new CPU instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            program: Vec::new(),
        }
    }
    
    /// Load a program from assembly source code.
    #[wasm_bindgen]
    pub fn load_asm(&mut self, source: &str) -> Result<usize, JsError> {
        let instructions = assemble(source)
            .map_err(|e| JsError::new(&format!("{}", e)))?;
        
        let len = instructions.len();
        self.program = instructions.clone();
        self.cpu = Cpu::new();
        self.cpu.load_program(&instructions)
            .map_err(|e| JsError::new(&format!("{}", e)))?;
        
        Ok(len)
    }
    
    /// Step one instruction. Returns the disassembled instruction.
    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<String, JsError> {
        if !self.cpu.is_running() {
            return Err(JsError::new("CPU is halted"));
        }
        
        let instr = self.cpu.step()
            .map_err(|e| JsError::new(&format!("{}", e)))?;
        
        Ok(disassemble_instruction(encode(&instr)))
    }
    
    /// Run until halt or max cycles.
    #[wasm_bindgen]
    pub fn run(&mut self, max_cycles: u32) -> u64 {
        let _ = self.cpu.run_limited(max_cycles as u64);
        self.cpu.cycles
    }
    
    /// Reset CPU to initial state with loaded program.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
        if !self.program.is_empty() {
            let _ = self.cpu.load_program(&self.program);
        }
    }
    
    /// Check if CPU is running.
    #[wasm_bindgen]
    pub fn is_running(&self) -> bool {
        self.cpu.is_running()
    }
    
    /// Check if CPU is halted.
    #[wasm_bindgen]
    pub fn is_halted(&self) -> bool {
        self.cpu.is_halted()
    }
    
    /// Get cycle count.
    #[wasm_bindgen]
    pub fn cycles(&self) -> u64 {
        self.cpu.cycles
    }
    
    /// Get program counter.
    #[wasm_bindgen]
    pub fn pc(&self) -> i32 {
        self.cpu.regs.c.to_i32()
    }
    
    /// Get accumulator value (S register).
    #[wasm_bindgen]
    pub fn accumulator(&self) -> i64 {
        self.cpu.regs.s.to_i64()
    }
    
    /// Get accumulator as ternary string.
    #[wasm_bindgen]
    pub fn accumulator_ternary(&self) -> String {
        format!("{}", self.cpu.regs.s)
    }
    
    /// Get multiplier register value (R register).
    #[wasm_bindgen]
    pub fn multiplier(&self) -> i64 {
        self.cpu.regs.r.to_i64()
    }
    
    /// Get index register value (F register).
    #[wasm_bindgen]
    pub fn index(&self) -> i32 {
        self.cpu.regs.f.to_i32()
    }
    
    /// Get omega (sign) register as string.
    #[wasm_bindgen]
    pub fn omega(&self) -> String {
        format!("{:?}", self.cpu.regs.omega)
    }
    
    /// Get state as string.
    #[wasm_bindgen]
    pub fn state(&self) -> String {
        format!("{:?}", self.cpu.state)
    }
    
    /// Get memory cell value at index (0-161).
    #[wasm_bindgen]
    pub fn memory_at(&self, index: usize) -> i32 {
        if index < 162 {
            self.cpu.mem.read(index).to_i32()
        } else {
            0
        }
    }
    
    /// Get memory cell as ternary string.
    #[wasm_bindgen]
    pub fn memory_ternary_at(&self, index: usize) -> String {
        if index < 162 {
            format!("{}", self.cpu.mem.read(index))
        } else {
            "OOOOOOOOO".to_string()
        }
    }
    
    /// Get all memory as JSON array of values.
    #[wasm_bindgen]
    pub fn memory_all(&self) -> Vec<i32> {
        (0..162).map(|i| self.cpu.mem.read(i).to_i32()).collect()
    }
    
    /// Get registers as JSON string.
    #[wasm_bindgen]
    pub fn registers_json(&self) -> String {
        format!(r#"{{"s":{},"r":{},"f":{},"c":{},"omega":"{}","cycles":{}}}"#,
            self.cpu.regs.s.to_i64(),
            self.cpu.regs.r.to_i64(),
            self.cpu.regs.f.to_i32(),
            self.cpu.regs.c.to_i32(),
            format!("{:?}", self.cpu.regs.omega),
            self.cpu.cycles
        )
    }
}

impl Default for WasmCpu {
    fn default() -> Self {
        Self::new()
    }
}

/// Assemble source code and return instruction count.
#[wasm_bindgen]
pub fn wasm_assemble(source: &str) -> Result<usize, JsError> {
    let instructions = assemble(source)
        .map_err(|e| JsError::new(&format!("{}", e)))?;
    Ok(instructions.len())
}

/// Disassemble a single 9-trit value.
#[wasm_bindgen]
pub fn wasm_disassemble(value: i32) -> String {
    let tryte = Tryte9::from_i32(value);
    disassemble_instruction(tryte)
}
