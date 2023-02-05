use crate::error::{create_result_error, VerifierError};
use num::bigint::{BigUint, ParseBigIntError};
use num::Num;
use std::fmt::Debug;
use std::{error::Error, fmt::Display};
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
    type HexaErrorType: Error;
    type ErrorKindType: Display + Debug;
    fn from_hexa(
        s: &String,
    ) -> Result<Self, VerifierError<Self::HexaErrorType, Self::ErrorKindType>>;
    fn to_hexa(&self) -> String;
}

impl Hexa for BigUint {
    type HexaErrorType = ParseBigIntError;
    type ErrorKindType = BigUIntErrorType;

    fn from_hexa(s: &String) -> Result<Self, VerifierError<ParseBigIntError, BigUIntErrorType>> {
        match <BigUint as Num>::from_str_radix(s, 16) {
            Ok(n) => Result::Ok(n),
            Err(e) => create_result_error!(
                BigUIntErrorType::FromHexaError,
                format!("Cannot convert biguint from hexa {}", s),
                e,
                ParseBigIntError
            ),
        }
    }

    fn to_hexa(&self) -> String {
        self.to_str_radix(16)
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
    fn from_exa() {
        assert_eq!(
            <BigUint as Num>::from_str_radix("a", 16).unwrap(),
            10.to_biguint().unwrap()
        )
    }

    //TODO Test from_exa and to_hexa
}
