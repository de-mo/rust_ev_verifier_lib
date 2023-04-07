//! Module to extend functionalities of BigUInt
//!
//! The extended functionalities are implemented using Trait that have to be
//! used in the client modules

use crate::error::{create_result_with_error, create_verifier_error, VerifierError};
use num::bigint::BigUint;
use num::Num;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BigUIntErrorType {
    FromHexaError,
}

impl Display for BigUIntErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FromHexaError => "BigUint from Hexa",
        };
        write!(f, "{s}")
    }
}

type BigUIntError = VerifierError<BigUIntErrorType>;

/// Trait to calculate byte length
pub trait ByteLength {
    /// Byte legnth of a BigUInt
    fn byte_length(&self) -> usize;
}

impl ByteLength for BigUint {
    fn byte_length(&self) -> usize {
        let bits = self.bits() as usize;
        let bytes = bits / 8;
        if bits % 8 == 0 {
            bytes
        } else {
            bytes + 1
        }
    }
}

/// Trait to implement constant numbers
pub trait Constants {
    fn zero() -> Self;
    fn one() -> Self;
    fn two() -> Self;
    fn three() -> Self;
    fn four() -> Self;
    fn five() -> Self;
}

impl Constants for BigUint {
    fn zero() -> Self {
        BigUint::from(0u8)
    }

    fn one() -> Self {
        BigUint::from(1u8)
    }

    fn two() -> Self {
        BigUint::from(2u8)
    }
    fn three() -> Self {
        BigUint::from(3u8)
    }
    fn four() -> Self {
        BigUint::from(4u8)
    }
    fn five() -> Self {
        BigUint::from(5u8)
    }
}

/// Trait to extend operations of BigUInt
pub trait Operations {
    /// Calculate the exponentiate modulo: n^exp % modulus
    fn mod_exponentiate(&self, exp: &Self, modulus: &Self) -> Self;
}

impl Operations for BigUint {
    fn mod_exponentiate(&self, exp: &Self, modulus: &Self) -> Self {
        self.modpow(exp, modulus)
    }
}

/// Transformation from or to String in hexadecimal according to the specifications
pub trait Hexa: Sized {
    /// Create object from hexadecimal String. If not valid return an error
    /// ```rust
    /// BigUint::from_hexa(&"0x12D9E8".to_string())
    /// ```
    fn from_hexa(s: &String) -> Result<Self, BigUIntError>;

    /// Generate the hexadecimal String
    fn to_hexa(&self) -> String;
}

impl Hexa for BigUint {
    fn from_hexa(s: &String) -> Result<Self, BigUIntError> {
        if !s.starts_with("0x") && !s.starts_with("0X") {
            return create_result_with_error!(
                BigUIntErrorType::FromHexaError,
                format!("Malformed hexa string. Must start with \"0x\" {}", s)
            );
        };
        <BigUint as Num>::from_str_radix(&s[2..], 16).or_else(|e| {
            create_result_with_error!(
                BigUIntErrorType::FromHexaError,
                format!("Cannot convert biguint from hexa {}", s),
                e
            )
        })
    }

    fn to_hexa(&self) -> String {
        format!("{}{}", "0x", self.to_str_radix(16))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use num::bigint::ToBigUint;
    use num::Num;

    #[test]
    fn bit_length() {
        assert_eq!(0.to_biguint().unwrap().bits(), 0);
        assert_eq!(1.to_biguint().unwrap().bits(), 1);
        assert_eq!(10.to_biguint().unwrap().bits(), 4);
    }

    #[test]
    fn byte_length() {
        assert_eq!(0.to_biguint().unwrap().byte_length(), 0);
        assert_eq!(3.to_biguint().unwrap().byte_length(), 1);
        assert_eq!(23591.to_biguint().unwrap().byte_length(), 2);
        assert_eq!(23592.to_biguint().unwrap().byte_length(), 2);
        assert_eq!(4294967295u64.to_biguint().unwrap().byte_length(), 4);
        assert_eq!(4294967296u64.to_biguint().unwrap().byte_length(), 5);
    }

    #[test]
    fn from_str_radix() {
        assert_eq!(
            <BigUint as Num>::from_str_radix("a", 16).unwrap(),
            10.to_biguint().unwrap()
        )
    }

    #[test]
    fn from_exa() {
        assert_eq!(
            BigUint::from_hexa(&"0x0".to_string()).unwrap(),
            0.to_biguint().unwrap()
        );
        assert_eq!(
            BigUint::from_hexa(&"0xa".to_string()).unwrap(),
            10.to_biguint().unwrap()
        );
        assert_eq!(
            BigUint::from_hexa(&"0xab".to_string()).unwrap(),
            171.to_biguint().unwrap()
        );
        assert_eq!(
            BigUint::from_hexa(&"0x12D9E8".to_string()).unwrap(),
            1235432.to_biguint().unwrap()
        );
        assert!(BigUint::from_hexa(&"123".to_string()).is_err());
        assert!(BigUint::from_hexa(&"0xtt".to_string()).is_err())
    }

    #[test]
    fn to_exa() {
        assert_eq!(0.to_biguint().unwrap().to_hexa(), "0x0");
        assert_eq!(10.to_biguint().unwrap().to_hexa(), "0xa");
        assert_eq!(171.to_biguint().unwrap().to_hexa(), "0xab");
        assert_eq!(1235432.to_biguint().unwrap().to_hexa(), "0x12d9e8");
    }
}
