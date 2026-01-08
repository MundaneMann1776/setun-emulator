//! Fixed-width balanced ternary words.
//!
//! This module provides the two word sizes used in the Setun:
//! - `Tryte9`: 9-trit "nitrit" for instructions and memory cells
//! - `Word18`: 18-trit full word for accumulator and computation

use std::fmt;
use serde::{Serialize, Deserialize};
use crate::ternary::Trit;

/// A 9-trit word (nitrit).
///
/// Used for:
/// - Memory cells (the Setun had 162 of these)
/// - Individual instructions (two fit in an 18-trit word)
/// - The index register F (only uses 5 trits, but stored as 9)
///
/// Value range: -9,841 to +9,841
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Tryte9 {
    /// Trits stored from least significant (index 0) to most significant (index 8)
    trits: [Trit; 9],
}

/// An 18-trit word.
///
/// Used for:
/// - The accumulator register S
/// - The multiplier register R
/// - Full-precision arithmetic
///
/// Value range: -193,710,244 to +193,710,244
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Word18 {
    /// Trits stored from least significant (index 0) to most significant (index 17)
    trits: [Trit; 18],
}

// ============================================================================
// Tryte9 Implementation
// ============================================================================

impl Tryte9 {
    /// Number of trits in a Tryte9.
    pub const WIDTH: usize = 9;
    
    /// Maximum positive value: +9,841 (all P's: PPP PPP PPP)
    pub const MAX: i32 = 9_841;
    
    /// Minimum negative value: -9,841 (all N's: NNN NNN NNN)
    pub const MIN: i32 = -9_841;
    
    /// Create a new Tryte9 with all zeros.
    #[inline]
    pub const fn zero() -> Self {
        Self { trits: [Trit::O; 9] }
    }
    
    /// Create a Tryte9 from an array of trits (LSB first).
    #[inline]
    pub const fn from_trits(trits: [Trit; 9]) -> Self {
        Self { trits }
    }
    
    /// Get the underlying trit array.
    #[inline]
    pub const fn trits(&self) -> &[Trit; 9] {
        &self.trits
    }
    
    /// Get a mutable reference to the trit array.
    #[inline]
    pub fn trits_mut(&mut self) -> &mut [Trit; 9] {
        &mut self.trits
    }
    
    /// Get a single trit by index (0 = LSB).
    #[inline]
    pub const fn get(&self, index: usize) -> Trit {
        self.trits[index]
    }
    
    /// Set a single trit by index (0 = LSB).
    #[inline]
    pub fn set(&mut self, index: usize, trit: Trit) {
        self.trits[index] = trit;
    }
    
    /// Create from a decimal integer.
    ///
    /// # Panics
    /// Panics if value is outside the range [-9841, +9841].
    pub fn from_i32(mut value: i32) -> Self {
        assert!(
            value >= Self::MIN && value <= Self::MAX,
            "Value {} out of range for Tryte9 [{}, {}]",
            value, Self::MIN, Self::MAX
        );
        
        let mut trits = [Trit::O; 9];
        let negative = value < 0;
        if negative {
            value = -value;
        }
        
        for i in 0..9 {
            let remainder = ((value % 3) + 1) as i8; // 0, 1, 2 -> 1, 2, 3
            let (trit, carry) = match remainder {
                1 => (Trit::O, 0),
                2 => (Trit::P, 0),
                3 => (Trit::N, 1), // 3 mod 3 = 0, but we need -1 + carry
                _ => unreachable!(),
            };
            trits[i] = trit;
            value = value / 3 + carry;
        }
        
        let mut result = Self { trits };
        if negative {
            result = result.neg();
        }
        result
    }
    
    /// Convert to a decimal integer.
    pub fn to_i32(&self) -> i32 {
        let mut result: i32 = 0;
        let mut power: i32 = 1;
        
        for i in 0..9 {
            result += self.trits[i].to_i8() as i32 * power;
            power *= 3;
        }
        
        result
    }
    
    /// Negate all trits.
    #[inline]
    pub fn neg(&self) -> Self {
        let mut trits = [Trit::O; 9];
        for i in 0..9 {
            trits[i] = self.trits[i].neg();
        }
        Self { trits }
    }
    
    /// Check if this word is zero.
    pub fn is_zero(&self) -> bool {
        self.trits.iter().all(|t| t.is_zero())
    }
    
    /// Get the sign of this word (the leading non-zero trit).
    pub fn sign(&self) -> Trit {
        for i in (0..9).rev() {
            if !self.trits[i].is_zero() {
                return self.trits[i];
            }
        }
        Trit::O
    }
    
    /// Extend to an 18-trit word (zero-extended).
    /// 
    /// Note: In balanced ternary, zero-extension preserves the value.
    /// Sign extension would change the value (unlike in two's complement).
    pub fn to_word18(&self) -> Word18 {
        let mut trits = [Trit::O; 18];
        for i in 0..9 {
            trits[i] = self.trits[i];
        }
        Word18 { trits }
    }
    
    /// Parse from a string like "0tPON" or "PONOOOOOO".
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let s = s.strip_prefix("0t").unwrap_or(s);
        
        if s.len() != 9 {
            return Err(ParseError::WrongLength { expected: 9, got: s.len() });
        }
        
        let mut trits = [Trit::O; 9];
        for (i, c) in s.chars().rev().enumerate() {
            trits[i] = match c {
                'N' | 'n' | '-' => Trit::N,
                'O' | 'o' | '0' => Trit::O,
                'P' | 'p' | '+' => Trit::P,
                _ => return Err(ParseError::InvalidChar(c)),
            };
        }
        
        Ok(Self { trits })
    }
}

impl fmt::Debug for Tryte9 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tryte9(0t")?;
        for i in (0..9).rev() {
            write!(f, "{:?}", self.trits[i])?;
        }
        write!(f, " = {})", self.to_i32())
    }
}

impl fmt::Display for Tryte9 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0t")?;
        for i in (0..9).rev() {
            write!(f, "{:?}", self.trits[i])?;
        }
        Ok(())
    }
}

impl std::ops::Neg for Tryte9 {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        Tryte9::neg(&self)
    }
}

// ============================================================================
// Word18 Implementation
// ============================================================================

impl Word18 {
    /// Number of trits in a Word18.
    pub const WIDTH: usize = 18;
    
    /// Maximum positive value: +193,710,244
    pub const MAX: i64 = 193_710_244;
    
    /// Minimum negative value: -193,710,244
    pub const MIN: i64 = -193_710_244;
    
    /// Create a new Word18 with all zeros.
    #[inline]
    pub const fn zero() -> Self {
        Self { trits: [Trit::O; 18] }
    }
    
    /// Create a Word18 from an array of trits (LSB first).
    #[inline]
    pub const fn from_trits(trits: [Trit; 18]) -> Self {
        Self { trits }
    }
    
    /// Get the underlying trit array.
    #[inline]
    pub const fn trits(&self) -> &[Trit; 18] {
        &self.trits
    }
    
    /// Get a mutable reference to the trit array.
    #[inline]
    pub fn trits_mut(&mut self) -> &mut [Trit; 18] {
        &mut self.trits
    }
    
    /// Get a single trit by index (0 = LSB).
    #[inline]
    pub const fn get(&self, index: usize) -> Trit {
        self.trits[index]
    }
    
    /// Set a single trit by index (0 = LSB).
    #[inline]
    pub fn set(&mut self, index: usize, trit: Trit) {
        self.trits[index] = trit;
    }
    
    /// Create from a decimal integer.
    ///
    /// # Panics
    /// Panics if value is outside the valid range.
    pub fn from_i64(mut value: i64) -> Self {
        assert!(
            value >= Self::MIN && value <= Self::MAX,
            "Value {} out of range for Word18 [{}, {}]",
            value, Self::MIN, Self::MAX
        );
        
        let mut trits = [Trit::O; 18];
        let negative = value < 0;
        if negative {
            value = -value;
        }
        
        for i in 0..18 {
            let remainder = ((value % 3) + 1) as i8;
            let (trit, carry) = match remainder {
                1 => (Trit::O, 0),
                2 => (Trit::P, 0),
                3 => (Trit::N, 1),
                _ => unreachable!(),
            };
            trits[i] = trit;
            value = value / 3 + carry;
        }
        
        let mut result = Self { trits };
        if negative {
            result = result.neg();
        }
        result
    }
    
    /// Convert to a decimal integer.
    pub fn to_i64(&self) -> i64 {
        let mut result: i64 = 0;
        let mut power: i64 = 1;
        
        for i in 0..18 {
            result += self.trits[i].to_i8() as i64 * power;
            power *= 3;
        }
        
        result
    }
    
    /// Negate all trits.
    #[inline]
    pub fn neg(&self) -> Self {
        let mut trits = [Trit::O; 18];
        for i in 0..18 {
            trits[i] = self.trits[i].neg();
        }
        Self { trits }
    }
    
    /// Check if this word is zero.
    pub fn is_zero(&self) -> bool {
        self.trits.iter().all(|t| t.is_zero())
    }
    
    /// Get the sign of this word (the leading non-zero trit).
    pub fn sign(&self) -> Trit {
        for i in (0..18).rev() {
            if !self.trits[i].is_zero() {
                return self.trits[i];
            }
        }
        Trit::O
    }
    
    /// Extract the low 9-trit half.
    pub fn low(&self) -> Tryte9 {
        let mut trits = [Trit::O; 9];
        for i in 0..9 {
            trits[i] = self.trits[i];
        }
        Tryte9 { trits }
    }
    
    /// Extract the high 9-trit half.
    pub fn high(&self) -> Tryte9 {
        let mut trits = [Trit::O; 9];
        for i in 0..9 {
            trits[i] = self.trits[i + 9];
        }
        Tryte9 { trits }
    }
    
    /// Create from two 9-trit halves.
    pub fn from_halves(low: Tryte9, high: Tryte9) -> Self {
        let mut trits = [Trit::O; 18];
        for i in 0..9 {
            trits[i] = low.trits[i];
            trits[i + 9] = high.trits[i];
        }
        Self { trits }
    }
    
    /// Parse from a string like "0tPONOOOOOOOOOOOOOOO".
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let s = s.strip_prefix("0t").unwrap_or(s);
        
        if s.len() != 18 {
            return Err(ParseError::WrongLength { expected: 18, got: s.len() });
        }
        
        let mut trits = [Trit::O; 18];
        for (i, c) in s.chars().rev().enumerate() {
            trits[i] = match c {
                'N' | 'n' | '-' => Trit::N,
                'O' | 'o' | '0' => Trit::O,
                'P' | 'p' | '+' => Trit::P,
                _ => return Err(ParseError::InvalidChar(c)),
            };
        }
        
        Ok(Self { trits })
    }
}

impl fmt::Debug for Word18 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Word18(0t")?;
        for i in (0..18).rev() {
            write!(f, "{:?}", self.trits[i])?;
            if i == 9 {
                write!(f, " ")?; // Visual separator between halves
            }
        }
        write!(f, " = {})", self.to_i64())
    }
}

impl fmt::Display for Word18 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0t")?;
        for i in (0..18).rev() {
            write!(f, "{:?}", self.trits[i])?;
        }
        Ok(())
    }
}

impl std::ops::Neg for Word18 {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        Word18::neg(&self)
    }
}

impl From<Tryte9> for Word18 {
    fn from(tryte: Tryte9) -> Self {
        tryte.to_word18()
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur when parsing ternary strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input string was the wrong length.
    WrongLength { expected: usize, got: usize },
    /// An invalid character was encountered.
    InvalidChar(char),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::WrongLength { expected, got } => {
                write!(f, "expected {} trits, got {}", expected, got)
            }
            ParseError::InvalidChar(c) => {
                write!(f, "invalid trit character: '{}' (expected N/O/P)", c)
            }
        }
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tryte9_zero() {
        let zero = Tryte9::zero();
        assert_eq!(zero.to_i32(), 0);
        assert!(zero.is_zero());
    }
    
    #[test]
    fn test_tryte9_from_i32() {
        // Test some known values
        assert_eq!(Tryte9::from_i32(0).to_i32(), 0);
        assert_eq!(Tryte9::from_i32(1).to_i32(), 1);
        assert_eq!(Tryte9::from_i32(-1).to_i32(), -1);
        assert_eq!(Tryte9::from_i32(42).to_i32(), 42);
        assert_eq!(Tryte9::from_i32(-42).to_i32(), -42);
        assert_eq!(Tryte9::from_i32(9841).to_i32(), 9841);
        assert_eq!(Tryte9::from_i32(-9841).to_i32(), -9841);
    }
    
    #[test]
    fn test_tryte9_negation() {
        let value = Tryte9::from_i32(42);
        let negated = value.neg();
        assert_eq!(negated.to_i32(), -42);
        assert_eq!(negated.neg().to_i32(), 42);
    }
    
    #[test]
    fn test_tryte9_sign() {
        assert_eq!(Tryte9::from_i32(42).sign(), Trit::P);
        assert_eq!(Tryte9::from_i32(-42).sign(), Trit::N);
        assert_eq!(Tryte9::from_i32(0).sign(), Trit::O);
    }
    
    #[test]
    fn test_tryte9_parse() {
        // String format: MSB first (left-to-right), so "OOOOOOPON" means:
        // positions 8..0 = O,O,O,O,O,O,P,O,N (reversed during parsing)
        // trits[0]=N, trits[1]=O, trits[2]=P, trits[3..8]=O
        // = (-1)*1 + 0*3 + 1*9 = -1 + 9 = 8
        let parsed = Tryte9::parse("OOOOOOPON").unwrap();
        assert_eq!(parsed.to_i32(), 8);
        
        // Test 42: 42 in balanced ternary is P N N O (from MSB)
        // 42 = 1*27 + (-1)*9 + (-1)*3 + 0*1 = 27 - 9 - 3 = 15... that's not right
        // Let me compute: 42 / 3 = 14 r 0 -> trit 0 = O
        // 14 / 3 = 4 r 2 -> needs adjustment: 5 carry, trit 1 = N
        // 5 / 3 = 1 r 2 -> 2 carry, trit 2 = N  
        // 2 / 3 = 0 r 2 -> 1 carry, trit 3 = N
        // 1 / 3 = 0 r 1 -> trit 4 = P
        // So 42 = 0tOOOOPNNNO (MSB first) = P*81 - N*27 - N*9 - N*3 = 81 - 27 - 9 - 3 = 42 ✓
        let forty_two = Tryte9::from_i32(42);
        let parsed_42 = Tryte9::parse(&format!("{}", forty_two)).unwrap();
        assert_eq!(parsed_42.to_i32(), 42);
        
        // All positive
        let all_p = Tryte9::parse("PPPPPPPPP").unwrap();
        assert_eq!(all_p.to_i32(), 9841);
    }
    
    #[test]
    fn test_word18_basics() {
        let zero = Word18::zero();
        assert_eq!(zero.to_i64(), 0);
        assert!(zero.is_zero());
        
        let value = Word18::from_i64(123456);
        assert_eq!(value.to_i64(), 123456);
        
        let negated = value.neg();
        assert_eq!(negated.to_i64(), -123456);
    }
    
    #[test]
    fn test_word18_halves() {
        let low = Tryte9::from_i32(42);
        let high = Tryte9::from_i32(100);
        let combined = Word18::from_halves(low, high);
        
        assert_eq!(combined.low().to_i32(), 42);
        assert_eq!(combined.high().to_i32(), 100);
    }
    
    #[test]
    fn test_tryte9_to_word18() {
        let positive = Tryte9::from_i32(42);
        let extended = positive.to_word18();
        assert_eq!(extended.to_i64(), 42);
        
        let negative = Tryte9::from_i32(-42);
        let extended_neg = negative.to_word18();
        assert_eq!(extended_neg.to_i64(), -42);
    }
}
