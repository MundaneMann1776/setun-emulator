//! TROM file format for Setun programs.
//!
//! TROM (Ternary ROM) is a simple text-based format:
//! - One instruction per line
//! - Trits represented as N/O/P characters
//! - Lines starting with `;` are comments
//! - Blank lines are ignored

use crate::ternary::Tryte9;
use std::path::Path;
use std::io::{BufRead, BufReader, Write};
use thiserror::Error;

/// A loaded TROM file.
#[derive(Debug, Clone)]
pub struct TromFile {
    /// The program instructions.
    pub instructions: Vec<Tryte9>,
    /// Original source lines (for debugging).
    pub source_lines: Vec<String>,
}

impl TromFile {
    /// Create a new empty TROM file.
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            source_lines: Vec::new(),
        }
    }
    
    /// Add an instruction.
    pub fn push(&mut self, instr: Tryte9, source: &str) {
        self.instructions.push(instr);
        self.source_lines.push(source.to_string());
    }
    
    /// Get the number of instructions.
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
    
    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

impl Default for TromFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Load a TROM file from disk.
pub fn load_trom<P: AsRef<Path>>(path: P) -> Result<TromFile, TromError> {
    let file = std::fs::File::open(path.as_ref())
        .map_err(|e| TromError::IoError(e.to_string()))?;
    let reader = BufReader::new(file);
    
    let mut trom = TromFile::new();
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| TromError::IoError(e.to_string()))?;
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        
        // Parse the trit string (first 9 characters, ignoring anything after)
        let trit_str: String = trimmed.chars()
            .filter(|c| matches!(c, 'N' | 'O' | 'P' | 'n' | 'o' | 'p'))
            .take(9)
            .collect();
        
        if trit_str.len() != 9 {
            return Err(TromError::ParseError {
                line: line_num + 1,
                message: format!("expected 9 trits, found {}", trit_str.len()),
            });
        }
        
        let instr = Tryte9::parse(&trit_str)
            .map_err(|e| TromError::ParseError {
                line: line_num + 1,
                message: format!("{}", e),
            })?;
        
        trom.push(instr, trimmed);
    }
    
    Ok(trom)
}

/// Save a TROM file to disk.
pub fn save_trom<P: AsRef<Path>>(path: P, trom: &TromFile) -> Result<(), TromError> {
    let mut file = std::fs::File::create(path.as_ref())
        .map_err(|e| TromError::IoError(e.to_string()))?;
    
    writeln!(file, "; Setun TROM file")
        .map_err(|e| TromError::IoError(e.to_string()))?;
    writeln!(file, "; {} instructions", trom.len())
        .map_err(|e| TromError::IoError(e.to_string()))?;
    writeln!(file).map_err(|e| TromError::IoError(e.to_string()))?;
    
    for (i, instr) in trom.instructions.iter().enumerate() {
        // Format: NNNNNNNNN ; addr comment
        writeln!(file, "{} ; {:03}", instr, i)
            .map_err(|e| TromError::IoError(e.to_string()))?;
    }
    
    Ok(())
}

/// Save instructions directly to TROM.
pub fn save_instructions<P: AsRef<Path>>(path: P, instructions: &[Tryte9]) -> Result<(), TromError> {
    let trom = TromFile {
        instructions: instructions.to_vec(),
        source_lines: instructions.iter().map(|i| format!("{}", i)).collect(),
    };
    save_trom(path, &trom)
}

/// Errors that can occur during TROM operations.
#[derive(Debug, Clone, Error)]
pub enum TromError {
    #[error("I/O error: {0}")]
    IoError(String),
    
    #[error("parse error on line {line}: {message}")]
    ParseError { line: usize, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trom_roundtrip() {
        let mut trom = TromFile::new();
        trom.push(Tryte9::from_i32(0), "HLT");
        trom.push(Tryte9::from_i32(42), "DATA");
        
        // Would need a temp file to test full roundtrip
        assert_eq!(trom.len(), 2);
    }
}
