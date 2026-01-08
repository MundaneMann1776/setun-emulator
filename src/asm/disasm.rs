//! Disassembler for Setun programs.
//!
//! Converts binary TROM instructions back to readable assembly.

use crate::ternary::Tryte9;
use crate::cpu::decode::{decode, Instruction, AddrMode};

/// Disassemble a single instruction to text.
pub fn disassemble_instruction(instr: Tryte9) -> String {
    match decode(instr) {
        Ok(decoded) => format_instruction(&decoded),
        Err(_) => format!("??? ; {}", instr),
    }
}

/// Disassemble a slice of instructions.
pub fn disassemble(instructions: &[Tryte9]) -> String {
    let mut output = String::new();
    output.push_str("; Setun Disassembly\n");
    output.push_str("; -----------------\n\n");
    
    for (addr, instr) in instructions.iter().enumerate() {
        let line = disassemble_instruction(*instr);
        output.push_str(&format!("{:03}: {}  ; {}\n", addr, line, instr));
    }
    
    output
}

/// Format a decoded instruction as assembly text.
fn format_instruction(instr: &Instruction) -> String {
    match instr {
        // Arithmetic
        Instruction::Add { addr, mode } => format!("ADD {}", format_operand(addr, mode)),
        Instruction::Sub { addr, mode } => format!("SUB {}", format_operand(addr, mode)),
        Instruction::Mul { addr, mode } => format!("MUL {}", format_operand(addr, mode)),
        Instruction::Div { addr, mode } => format!("DIV {}", format_operand(addr, mode)),
        Instruction::AddAbs { addr, mode } => format!("ADDABS {}", format_operand(addr, mode)),
        Instruction::SubAbs { addr, mode } => format!("SUBABS {}", format_operand(addr, mode)),
        
        // Transfer
        Instruction::Lda { addr, mode } => format!("LDA {}", format_operand(addr, mode)),
        Instruction::LdaUnsigned { addr, mode } => format!("LDAU {}", format_operand(addr, mode)),
        Instruction::Sta { addr, mode } => format!("STA {}", format_operand(addr, mode)),
        Instruction::Ldf { addr, mode } => format!("LDF {}", format_operand(addr, mode)),
        Instruction::Stf { addr, mode } => format!("STF {}", format_operand(addr, mode)),
        Instruction::Ldr { addr, mode } => format!("LDR {}", format_operand(addr, mode)),
        Instruction::Str { addr, mode } => format!("STR {}", format_operand(addr, mode)),
        Instruction::Xchg { addr, mode } => format!("XCHG {}", format_operand(addr, mode)),
        
        // Control
        Instruction::Jmp { addr, mode } => format!("JMP {}", format_operand(addr, mode)),
        Instruction::Jz { addr, mode } => format!("JZ {}", format_operand(addr, mode)),
        Instruction::Jp { addr, mode } => format!("JP {}", format_operand(addr, mode)),
        Instruction::Jn { addr, mode } => format!("JN {}", format_operand(addr, mode)),
        Instruction::Jop { addr, mode } => format!("JOP {}", format_operand(addr, mode)),
        Instruction::Jon { addr, mode } => format!("JON {}", format_operand(addr, mode)),
        Instruction::Hlt => "HLT".to_string(),
        
        // Shift
        Instruction::Shl { count } => format!("SHL {}", count),
        Instruction::Shr { count } => format!("SHR {}", count),
        
        // Special
        Instruction::Nop => "NOP".to_string(),
        Instruction::Tst => "TST".to_string(),
    }
}

/// Format an address operand with mode suffix.
fn format_operand(addr: &Tryte9, mode: &AddrMode) -> String {
    let addr_val = addr.to_i32();
    match mode {
        AddrMode::Direct => format!("{}", addr_val),
        AddrMode::IndexAdd => format!("{},F+", addr_val),
        AddrMode::IndexSub => format!("{},F-", addr_val),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::decode::encode;
    
    #[test]
    fn test_disassemble_hlt() {
        let hlt = encode(&Instruction::Hlt);
        let result = disassemble_instruction(hlt);
        assert!(result.contains("HLT"));
    }
    
    #[test]
    fn test_disassemble_add() {
        let add = encode(&Instruction::Add { 
            addr: Tryte9::from_i32(10), 
            mode: AddrMode::Direct 
        });
        let result = disassemble_instruction(add);
        assert!(result.contains("ADD"));
    }
    
    #[test]
    fn test_disassemble_with_mode() {
        let jmp = encode(&Instruction::Jmp { 
            addr: Tryte9::from_i32(5), 
            mode: AddrMode::IndexAdd 
        });
        let result = disassemble_instruction(jmp);
        assert!(result.contains("JMP"));
        assert!(result.contains("F+"));
    }
}
