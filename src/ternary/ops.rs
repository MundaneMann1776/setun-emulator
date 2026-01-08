//! Tritwise operations trait.
//!
//! Defines common operations that can be applied to all ternary types:
//! trits, trytes, and words.

use crate::ternary::Trit;

/// Trait for types that support tritwise operations.
pub trait TritOps {
    /// The output type for operations that return a word of the same size.
    type Output;
    
    /// Negate all trits (flip N ↔ P).
    fn ternary_neg(&self) -> Self::Output;
    
    /// Tritwise minimum (ternary AND).
    fn ternary_min(&self, other: &Self) -> Self::Output;
    
    /// Tritwise maximum (ternary OR).
    fn ternary_max(&self, other: &Self) -> Self::Output;
    
    /// Tritwise consensus.
    fn ternary_consensus(&self, other: &Self) -> Self::Output;
}

impl TritOps for Trit {
    type Output = Trit;
    
    #[inline]
    fn ternary_neg(&self) -> Trit {
        self.neg()
    }
    
    #[inline]
    fn ternary_min(&self, other: &Self) -> Trit {
        Trit::min(*self, *other)
    }
    
    #[inline]
    fn ternary_max(&self, other: &Self) -> Trit {
        Trit::max(*self, *other)
    }
    
    #[inline]
    fn ternary_consensus(&self, other: &Self) -> Trit {
        self.consensus(*other)
    }
}

// Implement TritOps for Tryte9 and Word18 using a macro
macro_rules! impl_trit_ops {
    ($type:ty, $width:expr) => {
        impl TritOps for $type {
            type Output = Self;
            
            fn ternary_neg(&self) -> Self {
                self.neg()
            }
            
            fn ternary_min(&self, other: &Self) -> Self {
                let mut result = *self;
                for i in 0..$width {
                    result.trits_mut()[i] = self.get(i).min(other.get(i));
                }
                result
            }
            
            fn ternary_max(&self, other: &Self) -> Self {
                let mut result = *self;
                for i in 0..$width {
                    result.trits_mut()[i] = self.get(i).max(other.get(i));
                }
                result
            }
            
            fn ternary_consensus(&self, other: &Self) -> Self {
                let mut result = *self;
                for i in 0..$width {
                    result.trits_mut()[i] = self.get(i).consensus(other.get(i));
                }
                result
            }
        }
    };
}

use crate::ternary::{Tryte9, Word18};
impl_trit_ops!(Tryte9, 9);
impl_trit_ops!(Word18, 18);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ternary::{Tryte9, Word18};
    
    #[test]
    fn test_tryte9_ternary_min() {
        let a = Tryte9::from_i32(42);
        let b = Tryte9::from_i32(-10);
        let result = a.ternary_min(&b);
        
        // Each trit should be the minimum of the two
        for i in 0..9 {
            assert_eq!(result.get(i), a.get(i).min(b.get(i)));
        }
    }
    
    #[test]
    fn test_tryte9_ternary_max() {
        let a = Tryte9::from_i32(42);
        let b = Tryte9::from_i32(-10);
        let result = a.ternary_max(&b);
        
        for i in 0..9 {
            assert_eq!(result.get(i), a.get(i).max(b.get(i)));
        }
    }
    
    #[test]
    fn test_word18_tritwise_ops() {
        let a = Word18::from_i64(12345);
        let b = Word18::from_i64(-6789);
        
        // Just verify the operations don't panic and produce valid results
        let _neg = a.ternary_neg();
        let _min = a.ternary_min(&b);
        let _max = a.ternary_max(&b);
        let _cons = a.ternary_consensus(&b);
    }
}
