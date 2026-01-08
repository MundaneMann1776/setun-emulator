//! Setun CPU registers.
//!
//! The Setun had 5 registers:
//! - S: 18-trit accumulator (main computation register)
//! - R: 18-trit multiplier register (used in multiply/divide)
//! - F: 5-trit index register (for address modification)
//! - C: 9-trit program counter
//! - ω (omega): 1-trit sign register

use crate::ternary::{Trit, Tryte9, Word18};
use serde::{Serialize, Deserialize};

/// A 5-trit value for the index register.
/// Range: -121 to +121
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Tryte5 {
    trits: [Trit; 5],
}

impl Tryte5 {
    /// Maximum value: 121 (PPPPP)
    pub const MAX: i32 = 121;
    /// Minimum value: -121 (NNNNN)
    pub const MIN: i32 = -121;
    
    /// Create a zero value.
    pub const fn zero() -> Self {
        Self { trits: [Trit::O; 5] }
    }
    
    /// Create from an integer.
    pub fn from_i32(mut value: i32) -> Self {
        assert!(
            value >= Self::MIN && value <= Self::MAX,
            "Value {} out of range for Tryte5 [{}, {}]",
            value, Self::MIN, Self::MAX
        );
        
        let mut trits = [Trit::O; 5];
        let negative = value < 0;
        if negative {
            value = -value;
        }
        
        for i in 0..5 {
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
    
    /// Convert to integer.
    pub fn to_i32(&self) -> i32 {
        let mut result: i32 = 0;
        let mut power: i32 = 1;
        
        for i in 0..5 {
            result += self.trits[i].to_i8() as i32 * power;
            power *= 3;
        }
        
        result
    }
    
    /// Negate.
    pub fn neg(&self) -> Self {
        let mut trits = [Trit::O; 5];
        for i in 0..5 {
            trits[i] = self.trits[i].neg();
        }
        Self { trits }
    }
    
    /// Extend to 9-trit Tryte9 (zero-extended).
    pub fn to_tryte9(&self) -> Tryte9 {
        let mut trits = [Trit::O; 9];
        for i in 0..5 {
            trits[i] = self.trits[i];
        }
        Tryte9::from_trits(trits)
    }
}

impl std::fmt::Debug for Tryte5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "F=")?;
        for i in (0..5).rev() {
            write!(f, "{:?}", self.trits[i])?;
        }
        write!(f, " ({})", self.to_i32())
    }
}

/// The Setun register file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registers {
    /// S: 18-trit accumulator (main computation register)
    pub s: Word18,
    
    /// R: 18-trit multiplier/quotient register
    pub r: Word18,
    
    /// F: 5-trit index register for address modification
    pub f: Tryte5,
    
    /// C: 9-trit program counter
    pub c: Tryte9,
    
    /// ω (omega): 1-trit sign register
    /// Set based on the result of certain operations:
    /// P (+1) if result > 0
    /// O (0) if result = 0
    /// N (-1) if result < 0
    pub omega: Trit,
}

impl Registers {
    /// Create a new register file with all values zeroed.
    pub fn new() -> Self {
        Self {
            s: Word18::zero(),
            r: Word18::zero(),
            f: Tryte5::zero(),
            c: Tryte9::zero(),
            omega: Trit::O,
        }
    }
    
    /// Reset all registers to zero.
    pub fn reset(&mut self) {
        self.s = Word18::zero();
        self.r = Word18::zero();
        self.f = Tryte5::zero();
        self.c = Tryte9::zero();
        self.omega = Trit::O;
    }
    
    /// Set the omega register based on a value's sign.
    pub fn set_omega_from_word(&mut self, value: &Word18) {
        self.omega = value.sign();
    }
    
    /// Set the omega register based on a 9-trit value's sign.
    pub fn set_omega_from_tryte(&mut self, value: &Tryte9) {
        self.omega = value.sign();
    }
    
    /// Set the omega register directly.
    pub fn set_omega(&mut self, sign: Trit) {
        self.omega = sign;
    }
    
    /// Increment the program counter by 1.
    /// Returns the old value.
    pub fn advance_pc(&mut self) -> Tryte9 {
        let old = self.c;
        let new_val = self.c.to_i32() + 1;
        self.c = Tryte9::from_i32(new_val);
        old
    }
    
    /// Set the program counter to an absolute address.
    pub fn jump(&mut self, addr: Tryte9) {
        self.c = addr;
    }
    
    /// Compute an effective address using F register modification.
    /// 
    /// - mode = P (+1): address + F
    /// - mode = O (0): address unchanged  
    /// - mode = N (-1): address - F
    pub fn effective_address(&self, base_addr: Tryte9, mode: Trit) -> Tryte9 {
        let base = base_addr.to_i32();
        let f_val = self.f.to_i32();
        
        let effective = match mode {
            Trit::P => base + f_val,
            Trit::O => base,
            Trit::N => base - f_val,
        };
        
        Tryte9::from_i32(effective)
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tryte5_conversion() {
        assert_eq!(Tryte5::from_i32(0).to_i32(), 0);
        assert_eq!(Tryte5::from_i32(42).to_i32(), 42);
        assert_eq!(Tryte5::from_i32(-42).to_i32(), -42);
        assert_eq!(Tryte5::from_i32(121).to_i32(), 121);
        assert_eq!(Tryte5::from_i32(-121).to_i32(), -121);
    }
    
    #[test]
    fn test_effective_address() {
        let mut regs = Registers::new();
        regs.f = Tryte5::from_i32(10);
        
        let base = Tryte9::from_i32(50);
        
        // Mode O: unchanged
        assert_eq!(regs.effective_address(base, Trit::O).to_i32(), 50);
        
        // Mode P: add F
        assert_eq!(regs.effective_address(base, Trit::P).to_i32(), 60);
        
        // Mode N: subtract F
        assert_eq!(regs.effective_address(base, Trit::N).to_i32(), 40);
    }
    
    #[test]
    fn test_omega_from_value() {
        let mut regs = Registers::new();
        
        regs.set_omega_from_word(&Word18::from_i64(100));
        assert_eq!(regs.omega, Trit::P);
        
        regs.set_omega_from_word(&Word18::from_i64(-100));
        assert_eq!(regs.omega, Trit::N);
        
        regs.set_omega_from_word(&Word18::zero());
        assert_eq!(regs.omega, Trit::O);
    }
    
    #[test]
    fn test_advance_pc() {
        let mut regs = Registers::new();
        regs.c = Tryte9::from_i32(10);
        
        let old = regs.advance_pc();
        assert_eq!(old.to_i32(), 10);
        assert_eq!(regs.c.to_i32(), 11);
    }
}
