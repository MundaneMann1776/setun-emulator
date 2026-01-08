//! CPU execution engine for the Setun.
//!
//! Implements the fetch-decode-execute cycle and all instruction behaviors.

use crate::ternary::{Trit, Tryte9, Word18, arith};
use crate::cpu::{Memory, Registers};
use crate::cpu::decode::{self, Instruction, AddrMode, DecodeError};
use crate::cpu::registers::Tryte5;
use crate::cpu::memory::MemoryError;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// CPU execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuState {
    /// CPU is running normally.
    Running,
    /// CPU has halted (executed HLT instruction).
    Halted,
    /// CPU encountered an error.
    Error,
}

/// The Setun CPU.
#[derive(Clone, Serialize, Deserialize)]
pub struct Cpu {
    /// CPU registers.
    pub regs: Registers,
    /// Main memory.
    pub mem: Memory,
    /// Current execution state.
    pub state: CpuState,
    /// Instruction count (for profiling).
    pub cycles: u64,
    /// Last executed instruction (for debugging).
    last_instr: Option<Instruction>,
}

impl Cpu {
    /// Create a new CPU with zeroed state.
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            mem: Memory::new(),
            state: CpuState::Running,
            cycles: 0,
            last_instr: None,
        }
    }
    
    /// Reset the CPU to initial state.
    pub fn reset(&mut self) {
        self.regs.reset();
        self.mem.clear();
        self.state = CpuState::Running;
        self.cycles = 0;
        self.last_instr = None;
    }
    
    /// Load a program into memory.
    pub fn load_program(&mut self, program: &[Tryte9]) -> Result<(), MemoryError> {
        self.mem.load_program(81, program) // Load at address 0 (index 81)
    }
    
    /// Execute a single instruction.
    /// 
    /// Returns the instruction that was executed, or an error.
    pub fn step(&mut self) -> Result<Instruction, CpuError> {
        if self.state != CpuState::Running {
            return Err(CpuError::NotRunning(self.state));
        }
        
        // Fetch
        let pc = self.regs.c;
        let raw = self.mem.read_ternary(pc)
            .map_err(|e| CpuError::MemoryError(e))?;
        
        // Advance PC before decode (some jumps will override)
        self.regs.advance_pc();
        
        // Decode
        let instr = decode::decode(raw)
            .map_err(|e| CpuError::DecodeError(e))?;
        
        // Execute
        self.execute(instr)?;
        
        // Update state
        self.cycles += 1;
        self.last_instr = Some(instr);
        
        Ok(instr)
    }
    
    /// Run until halt or error.
    /// 
    /// Returns the number of instructions executed.
    pub fn run(&mut self) -> Result<u64, CpuError> {
        let start_cycles = self.cycles;
        
        while self.state == CpuState::Running {
            self.step()?;
        }
        
        Ok(self.cycles - start_cycles)
    }
    
    /// Run for at most `max_cycles` instructions.
    pub fn run_limited(&mut self, max_cycles: u64) -> Result<u64, CpuError> {
        let start_cycles = self.cycles;
        let limit = self.cycles + max_cycles;
        
        while self.state == CpuState::Running && self.cycles < limit {
            self.step()?;
        }
        
        Ok(self.cycles - start_cycles)
    }
    
    /// Execute a decoded instruction.
    fn execute(&mut self, instr: Instruction) -> Result<(), CpuError> {
        match instr {
            // ==================== Arithmetic ====================
            
            Instruction::Add { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let operand = self.load_word(eff_addr)?;
                let (result, _carry) = arith::add(&self.regs.s, &operand);
                self.regs.s = result;
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            Instruction::Sub { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let operand = self.load_word(eff_addr)?;
                let (result, _carry) = arith::subtract(&self.regs.s, &operand);
                self.regs.s = result;
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            Instruction::Mul { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let operand = self.load_word(eff_addr)?;
                let (low, high) = arith::multiply(&self.regs.s, &operand);
                self.regs.s = high; // High part in S
                self.regs.r = low;  // Low part in R
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            Instruction::Div { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let divisor = self.load_word(eff_addr)?;
                
                if divisor.is_zero() {
                    return Err(CpuError::DivisionByZero);
                }
                
                // Simple integer division
                let dividend = self.regs.s.to_i64();
                let divisor_val = divisor.to_i64();
                let quotient = dividend / divisor_val;
                let remainder = dividend % divisor_val;
                
                self.regs.s = Word18::from_i64(quotient);
                self.regs.r = Word18::from_i64(remainder);
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            Instruction::AddAbs { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let operand = self.load_word(eff_addr)?;
                let abs_operand = if operand.sign() == Trit::N {
                    operand.neg()
                } else {
                    operand
                };
                let (result, _carry) = arith::add(&self.regs.s, &abs_operand);
                self.regs.s = result;
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            Instruction::SubAbs { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let operand = self.load_word(eff_addr)?;
                let abs_operand = if operand.sign() == Trit::N {
                    operand.neg()
                } else {
                    operand
                };
                let (result, _carry) = arith::subtract(&self.regs.s, &abs_operand);
                self.regs.s = result;
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            // ==================== Data Transfer ====================
            
            Instruction::Lda { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let value = self.mem.read_ternary(eff_addr)?;
                // Zero-extend 9 trits to 18 trits (preserves value in balanced ternary)
                self.regs.s = value.to_word18();
                let s_sign = self.regs.s.sign();
                self.regs.set_omega(s_sign);
            }
            
            Instruction::LdaUnsigned { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let value = self.mem.read_ternary(eff_addr)?;
                // Zero-extend (same as to_word18)
                self.regs.s = value.to_word18();
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            Instruction::Sta { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let value = self.regs.s.low();
                self.mem.write_ternary(eff_addr, value)?;
            }
            
            Instruction::Ldf { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let value = self.mem.read_ternary(eff_addr)?;
                // Take low 5 trits
                let trits = value.trits();
                let f_trits = [trits[0], trits[1], trits[2], trits[3], trits[4]];
                self.regs.f = Tryte5::from_i32(
                    f_trits[0].to_i8() as i32 * 1 +
                    f_trits[1].to_i8() as i32 * 3 +
                    f_trits[2].to_i8() as i32 * 9 +
                    f_trits[3].to_i8() as i32 * 27 +
                    f_trits[4].to_i8() as i32 * 81
                );
            }
            
            Instruction::Stf { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let value = self.regs.f.to_tryte9();
                self.mem.write_ternary(eff_addr, value)?;
            }
            
            Instruction::Ldr { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let value = self.mem.read_ternary(eff_addr)?;
                // Zero-extend like LDA
                self.regs.r = value.to_word18();
            }
            
            Instruction::Str { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let value = self.regs.r.low();
                self.mem.write_ternary(eff_addr, value)?;
            }
            
            Instruction::Xchg { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                let mem_value = self.mem.read_ternary(eff_addr)?;
                let s_low = self.regs.s.low();
                self.mem.write_ternary(eff_addr, s_low)?;
                self.regs.s = mem_value.to_word18();
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            // ==================== Control Flow ====================
            
            Instruction::Jmp { addr, mode } => {
                let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                self.regs.jump(eff_addr);
            }
            
            Instruction::Jz { addr, mode } => {
                if self.regs.s.is_zero() {
                    let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                    self.regs.jump(eff_addr);
                }
            }
            
            Instruction::Jp { addr, mode } => {
                if self.regs.s.sign() == Trit::P {
                    let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                    self.regs.jump(eff_addr);
                }
            }
            
            Instruction::Jn { addr, mode } => {
                if self.regs.s.sign() == Trit::N {
                    let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                    self.regs.jump(eff_addr);
                }
            }
            
            Instruction::Jop { addr, mode } => {
                if self.regs.omega == Trit::P {
                    let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                    self.regs.jump(eff_addr);
                }
            }
            
            Instruction::Jon { addr, mode } => {
                if self.regs.omega == Trit::N {
                    let eff_addr = self.regs.effective_address(addr, mode.to_trit());
                    self.regs.jump(eff_addr);
                }
            }
            
            Instruction::Hlt => {
                self.state = CpuState::Halted;
            }
            
            // ==================== Shift Operations ====================
            
            Instruction::Shl { count } => {
                let shifted = arith::shift_left(&self.regs.s, count as usize);
                self.regs.s = shifted;
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            Instruction::Shr { count } => {
                let shifted = arith::shift_right(&self.regs.s, count as usize);
                self.regs.s = shifted;
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
            
            // ==================== Special ====================
            
            Instruction::Nop => {
                // Do nothing
            }
            
            Instruction::Tst => {
                let sign = self.regs.s.sign();
                self.regs.set_omega(sign);
            }
        }
        
        Ok(())
    }
    
    /// Load a memory word as an 18-trit value (zero-extended).
    /// In balanced ternary, zero-extension preserves the original value.
    fn load_word(&self, addr: Tryte9) -> Result<Word18, CpuError> {
        let value = self.mem.read_ternary(addr)?;
        Ok(value.to_word18())
    }
    
    /// Get the last executed instruction.
    pub fn last_instruction(&self) -> Option<Instruction> {
        self.last_instr
    }
    
    /// Check if the CPU is halted.
    pub fn is_halted(&self) -> bool {
        self.state == CpuState::Halted
    }
    
    /// Check if the CPU is running.
    pub fn is_running(&self) -> bool {
        self.state == CpuState::Running
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("state", &self.state)
            .field("cycles", &self.cycles)
            .field("regs", &self.regs)
            .finish()
    }
}

/// Errors that can occur during CPU execution.
#[derive(Debug, Clone, Error)]
pub enum CpuError {
    #[error("CPU not running: {0:?}")]
    NotRunning(CpuState),
    
    #[error("memory error: {0}")]
    MemoryError(#[from] MemoryError),
    
    #[error("decode error: {0}")]
    DecodeError(#[from] DecodeError),
    
    #[error("division by zero")]
    DivisionByZero,
    
    #[error("arithmetic overflow")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::decode::encode;
    
    fn make_program(instructions: &[Instruction]) -> Vec<Tryte9> {
        instructions.iter().map(|i| encode(i)).collect()
    }
    
    #[test]
    fn test_cpu_halt() {
        let mut cpu = Cpu::new();
        let program = make_program(&[Instruction::Hlt]);
        cpu.load_program(&program).unwrap();
        
        let executed = cpu.run().unwrap();
        
        assert_eq!(executed, 1);
        assert!(cpu.is_halted());
    }
    
    #[test]
    fn test_cpu_nop_then_halt() {
        let mut cpu = Cpu::new();
        let program = make_program(&[
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Hlt,
        ]);
        cpu.load_program(&program).unwrap();
        
        let executed = cpu.run().unwrap();
        
        assert_eq!(executed, 4);
        assert!(cpu.is_halted());
    }
    
    #[test]
    fn test_cpu_load_store() {
        let mut cpu = Cpu::new();
        
        // Store a value, then load it
        // Address 10 relative to 0 (index 81 + 10 = 91)
        cpu.mem.write(91, Tryte9::from_i32(42));
        
        let program = make_program(&[
            Instruction::Lda { 
                addr: Tryte9::from_i32(10), 
                mode: AddrMode::Direct 
            },
            Instruction::Hlt,
        ]);
        cpu.load_program(&program).unwrap();
        
        cpu.run().unwrap();
        
        assert_eq!(cpu.regs.s.to_i64(), 42);
    }
    
    #[test]
    fn test_cpu_arithmetic() {
        let mut cpu = Cpu::new();
        
        // Set up: load 10, add 5
        cpu.mem.write(91, Tryte9::from_i32(10));
        cpu.mem.write(92, Tryte9::from_i32(5));
        
        let program = make_program(&[
            Instruction::Lda { 
                addr: Tryte9::from_i32(10), 
                mode: AddrMode::Direct 
            },
            Instruction::Add { 
                addr: Tryte9::from_i32(11), 
                mode: AddrMode::Direct 
            },
            Instruction::Hlt,
        ]);
        cpu.load_program(&program).unwrap();
        
        cpu.run().unwrap();
        
        assert_eq!(cpu.regs.s.to_i64(), 15);
    }
    
    #[test]
    fn test_cpu_conditional_jump() {
        let mut cpu = Cpu::new();
        
        // Load positive value, JP should jump
        cpu.mem.write(91, Tryte9::from_i32(1));
        
        let program = make_program(&[
            Instruction::Lda { 
                addr: Tryte9::from_i32(10), 
                mode: AddrMode::Direct 
            },
            Instruction::Jp { 
                addr: Tryte9::from_i32(3), // Jump to HLT
                mode: AddrMode::Direct 
            },
            Instruction::Nop, // Should be skipped
            Instruction::Hlt,
        ]);
        cpu.load_program(&program).unwrap();
        
        let executed = cpu.run().unwrap();
        
        // Should be: LDA, JP, HLT = 3 instructions (NOP skipped)
        assert_eq!(executed, 3);
    }
    
    #[test]
    fn test_cpu_shift() {
        let mut cpu = Cpu::new();
        
        // Load 1, shift left by 2 (multiply by 9)
        cpu.mem.write(91, Tryte9::from_i32(1));
        
        let program = make_program(&[
            Instruction::Lda { 
                addr: Tryte9::from_i32(10), 
                mode: AddrMode::Direct 
            },
            Instruction::Shl { count: 2 },
            Instruction::Hlt,
        ]);
        cpu.load_program(&program).unwrap();
        
        cpu.run().unwrap();
        
        assert_eq!(cpu.regs.s.to_i64(), 9);
    }
}
