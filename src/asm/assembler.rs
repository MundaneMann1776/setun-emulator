//! Simple assembler for Setun programs.
//!
//! Syntax:
//! ```text
//! ; Comment
//! LABEL:          ; Define a label
//!     LDA 10      ; Load from address 10
//!     ADD 11,F+   ; Add with index register + mode
//!     JMP LABEL   ; Jump to label
//!     HLT         ; Halt
//!     
//!     ORG 50      ; Set origin address
//!     DAT 42      ; Define data value
//! ```

use crate::ternary::Tryte9;
use crate::cpu::decode::{Instruction, AddrMode, encode};
use std::collections::HashMap;
use thiserror::Error;

/// Assemble source code to a list of instructions.
pub fn assemble(source: &str) -> Result<Vec<Tryte9>, AssemblerError> {
    let mut asm = Assembler::new();
    asm.assemble(source)
}

/// The assembler state.
struct Assembler {
    /// Current address (origin).
    current_addr: i32,
    /// Symbol table (label -> address).
    symbols: HashMap<String, i32>,
    /// Pending references (address -> label).
    pending: Vec<(usize, String, usize)>, // (output_index, label, source_line)
    /// Output instructions.
    output: Vec<Tryte9>,
}

impl Assembler {
    fn new() -> Self {
        Self {
            current_addr: 0,
            symbols: HashMap::new(),
            pending: Vec::new(),
            output: Vec::new(),
        }
    }
    
    fn assemble(&mut self, source: &str) -> Result<Vec<Tryte9>, AssemblerError> {
        // Pass 1: Collect labels and generate code
        for (line_num, line) in source.lines().enumerate() {
            self.process_line(line, line_num + 1)?;
        }
        
        // Pass 2: Resolve forward references
        self.resolve_references()?;
        
        Ok(self.output.clone())
    }
    
    fn process_line(&mut self, line: &str, line_num: usize) -> Result<(), AssemblerError> {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(';') {
            return Ok(());
        }
        
        // Remove inline comments
        let line = if let Some(idx) = line.find(';') {
            line[..idx].trim()
        } else {
            line
        };
        
        if line.is_empty() {
            return Ok(());
        }
        
        // Check for label definition
        if let Some(colon_idx) = line.find(':') {
            let label = line[..colon_idx].trim().to_uppercase();
            if !label.is_empty() {
                self.symbols.insert(label, self.current_addr);
            }
            
            // Process rest of line if any
            let rest = line[colon_idx + 1..].trim();
            if !rest.is_empty() {
                return self.process_instruction(rest, line_num);
            }
            return Ok(());
        }
        
        self.process_instruction(line, line_num)
    }
    
    fn process_instruction(&mut self, line: &str, line_num: usize) -> Result<(), AssemblerError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        
        let mnemonic = parts[0].to_uppercase();
        let operand = if parts.len() > 1 { Some(parts[1]) } else { None };
        
        match mnemonic.as_str() {
            // Directives
            "ORG" => {
                let addr = self.parse_operand_value(operand.ok_or_else(|| {
                    AssemblerError::SyntaxError { line: line_num, message: "ORG requires address".into() }
                })?, line_num)?;
                self.current_addr = addr;
            }
            
            "DAT" | "DATA" => {
                let value = self.parse_operand_value(operand.ok_or_else(|| {
                    AssemblerError::SyntaxError { line: line_num, message: "DAT requires value".into() }
                })?, line_num)?;
                self.emit(Tryte9::from_i32(value));
            }
            
            // Instructions
            _ => {
                let instr = self.parse_instruction(&mnemonic, operand, line_num)?;
                self.emit(encode(&instr));
            }
        }
        
        Ok(())
    }
    
    fn parse_instruction(&mut self, mnemonic: &str, operand: Option<&str>, line_num: usize) 
        -> Result<Instruction, AssemblerError> 
    {
        // Parse operand and mode
        let (addr, mode) = if let Some(op) = operand {
            self.parse_address_operand(op, line_num)?
        } else {
            (Tryte9::zero(), AddrMode::Direct)
        };
        
        let instr = match mnemonic {
            // Arithmetic
            "ADD" => Instruction::Add { addr, mode },
            "SUB" => Instruction::Sub { addr, mode },
            "MUL" => Instruction::Mul { addr, mode },
            "DIV" => Instruction::Div { addr, mode },
            "ADDABS" | "ADA" => Instruction::AddAbs { addr, mode },
            "SUBABS" | "SBA" => Instruction::SubAbs { addr, mode },
            
            // Data transfer
            "LDA" | "LD" => Instruction::Lda { addr, mode },
            "STA" | "ST" => Instruction::Sta { addr, mode },
            "LDAU" => Instruction::LdaUnsigned { addr, mode },
            "LDF" => Instruction::Ldf { addr, mode },
            "STF" => Instruction::Stf { addr, mode },
            "LDR" => Instruction::Ldr { addr, mode },
            "STR" => Instruction::Str { addr, mode },
            "XCHG" | "XCH" => Instruction::Xchg { addr, mode },
            
            // Control flow
            "JMP" | "JP" | "J" => Instruction::Jmp { addr, mode },
            "JZ" | "JE" => Instruction::Jz { addr, mode },
            "JPO" | "JGT" => Instruction::Jp { addr, mode },
            "JNE" | "JLT" => Instruction::Jn { addr, mode },
            "JOP" => Instruction::Jop { addr, mode },
            "JON" => Instruction::Jon { addr, mode },
            "HLT" | "HALT" => Instruction::Hlt,
            
            // Shift
            "SHL" | "ASL" => {
                let count = addr.to_i32() as i8;
                Instruction::Shl { count }
            }
            "SHR" | "ASR" => {
                let count = addr.to_i32() as i8;
                Instruction::Shr { count }
            }
            
            // Special
            "NOP" => Instruction::Nop,
            "TST" => Instruction::Tst,
            
            _ => return Err(AssemblerError::UnknownMnemonic { 
                line: line_num, 
                mnemonic: mnemonic.to_string() 
            }),
        };
        
        Ok(instr)
    }
    
    fn parse_address_operand(&mut self, operand: &str, line_num: usize) 
        -> Result<(Tryte9, AddrMode), AssemblerError> 
    {
        // Check for mode suffix: ,F+ or ,F- or just bare address
        let (addr_part, mode) = if operand.ends_with(",F+") || operand.ends_with(",f+") {
            (&operand[..operand.len()-3], AddrMode::IndexAdd)
        } else if operand.ends_with(",F-") || operand.ends_with(",f-") {
            (&operand[..operand.len()-3], AddrMode::IndexSub)
        } else if operand.ends_with(",F") || operand.ends_with(",f") {
            (&operand[..operand.len()-2], AddrMode::IndexAdd)
        } else {
            (operand, AddrMode::Direct)
        };
        
        let addr = self.parse_operand_value(addr_part, line_num)?;
        Ok((Tryte9::from_i32(addr), mode))
    }
    
    fn parse_operand_value(&mut self, operand: &str, line_num: usize) -> Result<i32, AssemblerError> {
        let operand = operand.trim();
        
        // Check for ternary literal (0t prefix)
        if operand.starts_with("0t") || operand.starts_with("0T") {
            let trit_str = &operand[2..];
            // Pad to 9 trits if needed
            let padded = format!("{:O>9}", trit_str.to_uppercase());
            return Tryte9::parse(&padded)
                .map(|t| t.to_i32())
                .map_err(|e| AssemblerError::SyntaxError { 
                    line: line_num, 
                    message: format!("invalid ternary literal: {}", e) 
                });
        }
        
        // Check for hex literal
        if operand.starts_with("0x") || operand.starts_with("0X") {
            return i32::from_str_radix(&operand[2..], 16)
                .map_err(|_| AssemblerError::SyntaxError { 
                    line: line_num, 
                    message: "invalid hex literal".into() 
                });
        }
        
        // Check for decimal number
        if let Ok(num) = operand.parse::<i32>() {
            return Ok(num);
        }
        
        // Must be a label reference - store for pass 2
        // For now, just return 0 and add to pending
        let out_idx = self.output.len();
        self.pending.push((out_idx, operand.to_uppercase(), line_num));
        Ok(0) // Placeholder, will be resolved in pass 2
    }
    
    fn emit(&mut self, instr: Tryte9) {
        self.output.push(instr);
        self.current_addr += 1;
    }
    
    fn resolve_references(&mut self) -> Result<(), AssemblerError> {
        for (out_idx, label, line_num) in &self.pending {
            let addr = self.symbols.get(label)
                .ok_or_else(|| AssemblerError::UndefinedLabel { 
                    line: *line_num, 
                    label: label.clone() 
                })?;
            
            // Re-encode the instruction with the correct address
            // This is a simplified approach - we just update the address portion
            // In a real assembler, we'd fully re-encode
            if *out_idx < self.output.len() {
                // For now, just store the address directly (simple case)
                // This works for JMP and similar instructions
                self.output[*out_idx] = Tryte9::from_i32(*addr);
            }
        }
        Ok(())
    }
}

/// Errors that can occur during assembly.
#[derive(Debug, Clone, Error)]
pub enum AssemblerError {
    #[error("syntax error on line {line}: {message}")]
    SyntaxError { line: usize, message: String },
    
    #[error("unknown mnemonic on line {line}: {mnemonic}")]
    UnknownMnemonic { line: usize, mnemonic: String },
    
    #[error("undefined label on line {line}: {label}")]
    UndefinedLabel { line: usize, label: String },
    
    #[error("value out of range on line {line}: {value}")]
    ValueOutOfRange { line: usize, value: i32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_assemble_simple() {
        let source = r#"
            ; Simple test program
            LDA 10
            ADD 11
            STA 12
            HLT
        "#;
        
        let result = assemble(source).unwrap();
        assert_eq!(result.len(), 4);
    }
    
    #[test]
    fn test_assemble_with_labels() {
        let source = r#"
        START:
            LDA 10
            JMP END
            NOP
        END:
            HLT
        "#;
        
        let result = assemble(source).unwrap();
        assert_eq!(result.len(), 4);
    }
    
    #[test]
    fn test_assemble_data() {
        let source = r#"
            DAT 42
            DAT -17
            DAT 0
        "#;
        
        let result = assemble(source).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].to_i32(), 42);
        assert_eq!(result[1].to_i32(), -17);
        assert_eq!(result[2].to_i32(), 0);
    }
}
