//! Single balanced ternary digit (trit).
//!
//! A trit can hold one of three values: -1, 0, or +1.
//! We use a 2-bit Binary-Coded Ternary (BCT) encoding:
//! - `0b00` = 0 (Zero)
//! - `0b01` = +1 (Positive)
//! - `0b10` = -1 (Negative)
//! - `0b11` = Invalid (handled in debug mode)

use std::fmt;
use serde::{Serialize, Deserialize};

/// A single balanced ternary digit.
///
/// Represented internally using 2-bit BCT encoding for efficient
/// bitwise operations while maintaining the balanced ternary semantics.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Trit {
    /// Negative (-1)
    N = 0b10,
    /// Zero (0)  
    O = 0b00,
    /// Positive (+1)
    P = 0b01,
}

impl Trit {
    /// All possible trit values in order: N, O, P
    pub const ALL: [Trit; 3] = [Trit::N, Trit::O, Trit::P];
    
    /// Create a trit from a raw BCT byte.
    /// 
    /// In debug mode, panics on invalid encoding (0b11).
    /// In release mode, normalizes invalid values to O (zero).
    #[inline]
    pub fn from_bct(byte: u8) -> Self {
        match byte & 0b11 {
            0b00 => Trit::O,
            0b01 => Trit::P,
            0b10 => Trit::N,
            _ => {
                #[cfg(debug_assertions)]
                panic!("Invalid BCT trit encoding: 0b{:02b}", byte);
                #[cfg(not(debug_assertions))]
                Trit::O
            }
        }
    }
    
    /// Get the raw BCT byte representation.
    #[inline]
    pub const fn to_bct(self) -> u8 {
        self as u8
    }
    
    /// Create a trit from an integer value.
    /// 
    /// # Panics
    /// Panics if value is not in {-1, 0, 1}.
    #[inline]
    pub fn from_i8(value: i8) -> Self {
        match value {
            -1 => Trit::N,
            0 => Trit::O,
            1 => Trit::P,
            _ => panic!("Invalid trit value: {} (must be -1, 0, or 1)", value),
        }
    }
    
    /// Convert to integer value.
    #[inline]
    pub const fn to_i8(self) -> i8 {
        match self {
            Trit::N => -1,
            Trit::O => 0,
            Trit::P => 1,
        }
    }
    
    /// Negate the trit (flip N ↔ P, O stays O).
    #[inline]
    pub const fn neg(self) -> Self {
        match self {
            Trit::N => Trit::P,
            Trit::O => Trit::O,
            Trit::P => Trit::N,
        }
    }
    
    /// Minimum (ternary AND) - returns the lesser value.
    #[inline]
    pub const fn min(self, other: Self) -> Self {
        match (self.to_i8(), other.to_i8()) {
            (a, b) if a <= b => self,
            _ => other,
        }
    }
    
    /// Maximum (ternary OR) - returns the greater value.
    #[inline]
    pub const fn max(self, other: Self) -> Self {
        match (self.to_i8(), other.to_i8()) {
            (a, b) if a >= b => self,
            _ => other,
        }
    }
    
    /// Consensus - returns the value if both inputs match, else O.
    #[inline]
    pub const fn consensus(self, other: Self) -> Self {
        match (self, other) {
            (Trit::P, Trit::P) => Trit::P,
            (Trit::N, Trit::N) => Trit::N,
            _ => Trit::O,
        }
    }
    
    /// Any (gullibility) - accepts any non-zero input, prefers first.
    /// Used in carry chain combination.
    #[inline]
    pub const fn any(self, other: Self) -> Self {
        match self {
            Trit::O => other,
            _ => self,
        }
    }
    
    /// Half-adder sum: (a + b) mod 3, normalized to {-1, 0, 1}.
    #[inline]
    pub const fn sum(self, other: Self) -> Self {
        let total = self.to_i8() + other.to_i8();
        match total {
            -2 => Trit::P,   // -2 → +1 (wrap)
            -1 => Trit::N,
            0 => Trit::O,
            1 => Trit::P,
            2 => Trit::N,    // +2 → -1 (wrap)
            _ => unreachable!(),
        }
    }
    
    /// Half-adder carry: carry output when adding two trits.
    #[inline]
    pub const fn carry(self, other: Self) -> Self {
        let total = self.to_i8() + other.to_i8();
        match total {
            -2 => Trit::N,   // Borrow
            2 => Trit::P,    // Carry
            _ => Trit::O,    // No carry
        }
    }
    
    /// Full adder: adds three trits (a, b, c_in), returns (sum, carry_out).
    #[inline]
    pub const fn full_add(self, other: Self, carry_in: Self) -> (Self, Self) {
        // First half-adder: a + b
        let s1 = self.sum(other);
        let c1 = self.carry(other);
        
        // Second half-adder: s1 + carry_in
        let sum = s1.sum(carry_in);
        let c2 = s1.carry(carry_in);
        
        // Combine carries (they can't both be non-zero)
        let carry_out = c1.any(c2);
        
        (sum, carry_out)
    }
    
    /// Single-trit multiplication (never carries).
    #[inline]
    pub const fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Trit::O, _) | (_, Trit::O) => Trit::O,
            (Trit::P, Trit::P) | (Trit::N, Trit::N) => Trit::P,
            (Trit::P, Trit::N) | (Trit::N, Trit::P) => Trit::N,
        }
    }
    
    /// Returns true if this trit is zero.
    #[inline]
    pub const fn is_zero(self) -> bool {
        matches!(self, Trit::O)
    }
    
    /// Returns true if this trit is positive.
    #[inline]
    pub const fn is_positive(self) -> bool {
        matches!(self, Trit::P)
    }
    
    /// Returns true if this trit is negative.
    #[inline]
    pub const fn is_negative(self) -> bool {
        matches!(self, Trit::N)
    }
}

impl Default for Trit {
    fn default() -> Self {
        Trit::O
    }
}

impl fmt::Debug for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trit::N => write!(f, "N"),
            Trit::O => write!(f, "O"),
            Trit::P => write!(f, "P"),
        }
    }
}

impl fmt::Display for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trit::N => write!(f, "-"),
            Trit::O => write!(f, "0"),
            Trit::P => write!(f, "+"),
        }
    }
}

impl std::ops::Neg for Trit {
    type Output = Self;
    
    fn neg(self) -> Self::Output {
        Trit::neg(self)
    }
}

impl From<i8> for Trit {
    fn from(value: i8) -> Self {
        Trit::from_i8(value)
    }
}

impl From<Trit> for i8 {
    fn from(trit: Trit) -> Self {
        trit.to_i8()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_negation_involution() {
        for t in Trit::ALL {
            assert_eq!(t.neg().neg(), t, "negate(negate({:?})) should equal {:?}", t, t);
        }
    }
    
    #[test]
    fn test_sum_commutativity() {
        for a in Trit::ALL {
            for b in Trit::ALL {
                assert_eq!(a.sum(b), b.sum(a), "sum({:?}, {:?}) should be commutative", a, b);
            }
        }
    }
    
    #[test]
    fn test_multiplication_table() {
        // N * N = P, N * O = O, N * P = N
        assert_eq!(Trit::N.mul(Trit::N), Trit::P);
        assert_eq!(Trit::N.mul(Trit::O), Trit::O);
        assert_eq!(Trit::N.mul(Trit::P), Trit::N);
        
        // O * anything = O
        assert_eq!(Trit::O.mul(Trit::N), Trit::O);
        assert_eq!(Trit::O.mul(Trit::O), Trit::O);
        assert_eq!(Trit::O.mul(Trit::P), Trit::O);
        
        // P * N = N, P * O = O, P * P = P
        assert_eq!(Trit::P.mul(Trit::N), Trit::N);
        assert_eq!(Trit::P.mul(Trit::O), Trit::O);
        assert_eq!(Trit::P.mul(Trit::P), Trit::P);
    }
    
    #[test]
    fn test_full_adder() {
        // 0 + 0 + 0 = 0, carry 0
        assert_eq!(Trit::O.full_add(Trit::O, Trit::O), (Trit::O, Trit::O));
        
        // 1 + 1 + 0 = -1, carry 1 (2 = -1 + 3)
        assert_eq!(Trit::P.full_add(Trit::P, Trit::O), (Trit::N, Trit::P));
        
        // 1 + 1 + 1 = 0, carry 1 (3 = 0 + 3)
        assert_eq!(Trit::P.full_add(Trit::P, Trit::P), (Trit::O, Trit::P));
        
        // -1 + -1 + -1 = 0, carry -1 (-3 = 0 - 3)
        assert_eq!(Trit::N.full_add(Trit::N, Trit::N), (Trit::O, Trit::N));
    }
    
    #[test]
    fn test_consensus() {
        assert_eq!(Trit::P.consensus(Trit::P), Trit::P);
        assert_eq!(Trit::N.consensus(Trit::N), Trit::N);
        assert_eq!(Trit::O.consensus(Trit::O), Trit::O);
        assert_eq!(Trit::P.consensus(Trit::N), Trit::O);
        assert_eq!(Trit::P.consensus(Trit::O), Trit::O);
    }
    
    #[test]
    fn test_min_max() {
        assert_eq!(Trit::P.min(Trit::N), Trit::N);
        assert_eq!(Trit::P.max(Trit::N), Trit::P);
        assert_eq!(Trit::O.min(Trit::P), Trit::O);
        assert_eq!(Trit::O.max(Trit::N), Trit::O);
    }
    
    #[test]
    fn test_bct_roundtrip() {
        for t in Trit::ALL {
            assert_eq!(Trit::from_bct(t.to_bct()), t);
        }
    }
    
    #[test]
    fn test_i8_roundtrip() {
        for t in Trit::ALL {
            assert_eq!(Trit::from_i8(t.to_i8()), t);
        }
    }
}
