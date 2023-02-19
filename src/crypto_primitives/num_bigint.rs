use crate::error::{create_result_with_error, create_verifier_error, VerifierError};
use num::bigint::BigUint;
use num::Num;
use std::fmt::Debug;
use std::fmt::Display;
//use num::Num;

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

pub trait ByteLength {
    /// Byte legnth of a BigUInt
    fn byte_length(&self) -> u64;
}

impl ByteLength for BigUint {
    fn byte_length(&self) -> u64 {
        let bits = self.bits();
        let bytes = bits / 8;
        if bits % 8 == 0 {
            bytes
        } else {
            bytes + 1
        }
    }
}

pub trait Hexa: Sized {
    fn from_hexa(s: &String) -> Result<Self, BigUIntError>;
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
