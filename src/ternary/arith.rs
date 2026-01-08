//! Multi-trit arithmetic operations.
//!
//! Provides addition, subtraction, multiplication, and negation
//! for balanced ternary words using ripple-carry algorithms.

use crate::ternary::{Trit, Tryte9, Word18};

/// Negate a 9-trit word.
#[inline]
pub fn negate_tryte9(a: &Tryte9) -> Tryte9 {
    a.neg()
}

/// Negate an 18-trit word.
#[inline]
pub fn negate(a: &Word18) -> Word18 {
    a.neg()
}

/// Add two 9-trit words, returning (result, carry_out).
pub fn add_tryte9(a: &Tryte9, b: &Tryte9) -> (Tryte9, Trit) {
    let mut result = Tryte9::zero();
    let mut carry = Trit::O;
    
    for i in 0..9 {
        let (sum, new_carry) = a.get(i).full_add(b.get(i), carry);
        result.set(i, sum);
        carry = new_carry;
    }
    
    (result, carry)
}

/// Add two 18-trit words, returning (result, carry_out).
pub fn add(a: &Word18, b: &Word18) -> (Word18, Trit) {
    let mut result = Word18::zero();
    let mut carry = Trit::O;
    
    for i in 0..18 {
        let (sum, new_carry) = a.get(i).full_add(b.get(i), carry);
        result.set(i, sum);
        carry = new_carry;
    }
    
    (result, carry)
}

/// Subtract two 9-trit words (a - b), returning (result, borrow_out).
#[inline]
pub fn subtract_tryte9(a: &Tryte9, b: &Tryte9) -> (Tryte9, Trit) {
    add_tryte9(a, &b.neg())
}

/// Subtract two 18-trit words (a - b), returning (result, borrow_out).
#[inline]
pub fn subtract(a: &Word18, b: &Word18) -> (Word18, Trit) {
    add(a, &b.neg())
}

/// Multiply two 18-trit words, returning a 36-trit result as (low, high).
///
/// Uses the schoolbook multiplication algorithm adapted for balanced ternary.
/// Note: Single-trit multiplication never carries, which simplifies partial products.
pub fn multiply(a: &Word18, b: &Word18) -> (Word18, Word18) {
    // We need 36 trits to hold the full product
    let mut product = [Trit::O; 36];
    
    // Schoolbook multiplication: for each trit in a, multiply by b and add shifted
    for i in 0..18 {
        if a.get(i).is_zero() {
            continue; // Multiplying by zero contributes nothing
        }
        
        let mut carry = Trit::O;
        for j in 0..18 {
            // Single-trit multiply (never carries)
            let partial = a.get(i).mul(b.get(j));
            
            // Add to accumulator with carry
            let (sum1, c1) = product[i + j].full_add(partial, Trit::O);
            let (sum2, c2) = sum1.full_add(carry, Trit::O);
            product[i + j] = sum2;
            carry = c1.any(c2);
        }
        
        // Propagate any remaining carry
        let mut k = i + 18;
        while !carry.is_zero() && k < 36 {
            let (sum, new_carry) = product[k].full_add(carry, Trit::O);
            product[k] = sum;
            carry = new_carry;
            k += 1;
        }
    }
    
    // Split into low and high 18-trit words
    let mut low_trits = [Trit::O; 18];
    let mut high_trits = [Trit::O; 18];
    
    for i in 0..18 {
        low_trits[i] = product[i];
        high_trits[i] = product[i + 18];
    }
    
    (Word18::from_trits(low_trits), Word18::from_trits(high_trits))
}

/// Shift a word left by n trit positions (multiply by 3^n).
/// Fills vacated positions with zeros. Trits shifted out are lost.
pub fn shift_left(a: &Word18, n: usize) -> Word18 {
    if n >= 18 {
        return Word18::zero();
    }
    
    let mut result = Word18::zero();
    for i in 0..(18 - n) {
        result.set(i + n, a.get(i));
    }
    result
}

/// Shift a word right by n trit positions (divide by 3^n, truncated).
/// In balanced ternary, truncation equals rounding!
pub fn shift_right(a: &Word18, n: usize) -> Word18 {
    if n >= 18 {
        return Word18::zero();
    }
    
    let mut result = Word18::zero();
    for i in n..18 {
        result.set(i - n, a.get(i));
    }
    result
}

/// Compare two words, returning their relationship.
pub fn compare(a: &Word18, b: &Word18) -> std::cmp::Ordering {
    let a_val = a.to_i64();
    let b_val = b.to_i64();
    a_val.cmp(&b_val)
}

/// Check if addition would overflow (result outside representable range).
/// In balanced ternary, overflow is indicated by a non-zero final carry.
#[inline]
pub fn would_overflow(a: &Word18, b: &Word18) -> bool {
    let (_, carry) = add(a, b);
    !carry.is_zero()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_basic() {
        let a = Word18::from_i64(100);
        let b = Word18::from_i64(50);
        let (result, carry) = add(&a, &b);
        
        assert_eq!(result.to_i64(), 150);
        assert!(carry.is_zero());
    }
    
    #[test]
    fn test_add_negative() {
        let a = Word18::from_i64(100);
        let b = Word18::from_i64(-150);
        let (result, _) = add(&a, &b);
        
        assert_eq!(result.to_i64(), -50);
    }
    
    #[test]
    fn test_subtract() {
        let a = Word18::from_i64(100);
        let b = Word18::from_i64(30);
        let (result, _) = subtract(&a, &b);
        
        assert_eq!(result.to_i64(), 70);
    }
    
    #[test]
    fn test_multiply_simple() {
        let a = Word18::from_i64(7);
        let b = Word18::from_i64(6);
        let (low, high) = multiply(&a, &b);
        
        assert_eq!(low.to_i64(), 42);
        assert!(high.is_zero()); // No overflow for small numbers
    }
    
    #[test]
    fn test_multiply_negative() {
        let a = Word18::from_i64(-7);
        let b = Word18::from_i64(6);
        let (low, high) = multiply(&a, &b);
        
        assert_eq!(low.to_i64(), -42);
        // High word should be sign extension
        assert!(high.to_i64() <= 0);
    }
    
    #[test]
    fn test_multiply_larger() {
        let a = Word18::from_i64(1000);
        let b = Word18::from_i64(1000);
        let (low, high) = multiply(&a, &b);
        
        // 1000 * 1000 = 1,000,000 which fits in 18 trits
        assert_eq!(low.to_i64(), 1_000_000);
        assert!(high.is_zero());
    }
    
    #[test]
    fn test_shift_left() {
        let a = Word18::from_i64(1);
        
        // Shift left by 1 = multiply by 3
        let shifted = shift_left(&a, 1);
        assert_eq!(shifted.to_i64(), 3);
        
        // Shift left by 2 = multiply by 9
        let shifted2 = shift_left(&a, 2);
        assert_eq!(shifted2.to_i64(), 9);
    }
    
    #[test]
    fn test_shift_right() {
        let a = Word18::from_i64(27);
        
        // Shift right by 1 = divide by 3
        let shifted = shift_right(&a, 1);
        assert_eq!(shifted.to_i64(), 9);
        
        // Shift right by 3 = divide by 27
        let shifted3 = shift_right(&a, 3);
        assert_eq!(shifted3.to_i64(), 1);
    }
    
    #[test]
    fn test_additive_inverse() {
        // a + (-a) should equal 0
        for val in [-9841i64, -100, -1, 0, 1, 100, 9841] {
            let a = Word18::from_i64(val);
            let neg_a = negate(&a);
            let (result, _) = add(&a, &neg_a);
            assert!(result.is_zero(), "Expected {} + (-{}) = 0", val, val);
        }
    }
    
    #[test]
    fn test_add_commutativity() {
        let a = Word18::from_i64(12345);
        let b = Word18::from_i64(-6789);
        
        let (r1, _) = add(&a, &b);
        let (r2, _) = add(&b, &a);
        
        assert_eq!(r1.to_i64(), r2.to_i64());
    }
    
    #[test]
    fn test_tryte9_add() {
        let a = Tryte9::from_i32(100);
        let b = Tryte9::from_i32(50);
        let (result, carry) = add_tryte9(&a, &b);
        
        assert_eq!(result.to_i32(), 150);
        assert!(carry.is_zero());
    }
}
