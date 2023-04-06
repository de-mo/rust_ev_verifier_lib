use num::BigUint;

use crate::error::{create_result_with_error, create_verifier_error, VerifierError};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberTheoryErrorType {
    NotInRange,
}

impl Display for NumberTheoryErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::NotInRange => "Not in Range",
        };
        write!(f, "{s}")
    }
}

type NumberTheoryError = VerifierError<NumberTheoryErrorType>;

pub fn is_small_prime(n: usize) -> Result<bool, NumberTheoryError> {
    if n >= usize::pow(2, 31) {
        return create_result_with_error!(NumberTheoryErrorType::NotInRange, "n is to big");
    };
    if n == 0 {
        return create_result_with_error!(NumberTheoryErrorType::NotInRange, "n cannt be 0");
    };
    if n == 1 {
        return Ok(false);
    };
    if n == 2 || n == 3 {
        return Ok(true);
    }
    if n % 2 == 0 || n % 3 == 0 {
        return Ok(false);
    }
    let mut i = 5;
    let limit = f64::sqrt(n as f64) as usize + 1;
    while i <= limit {
        if n % i == 0 || n % (i + 2) == 0 {
            return Ok(false);
        }
        i = i + 6
    }
    return Ok(true);
}

pub fn satisfy_euler_criterion(n: usize, p: &BigUint) -> bool {
    let n_bui = BigUint::from(n);
    let exp = (p - BigUint::from(1u8)) / BigUint::from(2u8);
    return n_bui.modpow(&exp, p) == BigUint::from(1u8);
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_is_small_prime() {
        assert!(is_small_prime(0).is_err());
        assert!(is_small_prime(usize::pow(2, 31)).is_err());
        assert!((is_small_prime(2)).unwrap());
        assert!((is_small_prime(3)).unwrap());
        assert!((is_small_prime(5)).unwrap());
        assert!((is_small_prime(41)).unwrap());
        assert!((!(is_small_prime(4)).unwrap()));
        assert!((!(is_small_prime(12)).unwrap()));
        assert!((!(is_small_prime(49)).unwrap()));
        assert!((!(is_small_prime(99)).unwrap()));
    }

    #[test]
    fn test_satisfy_euler_criterion() {
        assert!(!satisfy_euler_criterion(17, &BigUint::from(3u8)));
        assert!(satisfy_euler_criterion(17, &BigUint::from(13u8)));
    }
}
