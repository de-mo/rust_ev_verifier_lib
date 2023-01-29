use crate::crypto_primitives::num_bigint::ByteLength;
use num::bigint::{BigUint, ToBigUint};
use std::fmt::Debug;

pub struct ByteArray {
    inner: Vec<u8>,
}

pub trait Encode {
    fn base16_encode(&self) -> String;
    fn base32_encode(&self) -> String;
    fn base64_encode(&self) -> String;
}

pub trait Decode {
    fn base16_decode(s: &String) -> Self;
    fn base32_decode(s: &String) -> Self;
    fn base64_decode(s: &String) -> Self;
}

//TODO Implement trait Encode
//TODO Implement trait Decode

impl Default for ByteArray {
    fn default() -> Self {
        ByteArray::new()
    }
}

impl Debug for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl From<&BigUint> for ByteArray {
    fn from(value: &BigUint) -> Self {
        let byte_length = std::cmp::max(value.byte_length(), 1);
        let mut x = value.clone();
        let mut d: Vec<u8> = Vec::new();
        for _i in 0..byte_length {
            d.insert(0, (x.clone() % 256.to_biguint().unwrap()).to_bytes_le()[0]);
            x = x / 256.to_biguint().unwrap();
        }
        ByteArray::from(&d)
    }
}

impl From<&Vec<u8>> for ByteArray {
    fn from(bytes: &Vec<u8>) -> Self {
        if bytes.is_empty() {
            ByteArray::new()
        } else {
            ByteArray {
                inner: (*bytes.clone()).to_vec(),
            }
        }
    }
}

impl ByteArray {
    pub fn new() -> Self {
        ByteArray { inner: vec![0] }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        ByteArray::from(&Vec::from(bytes))
    }

    pub fn into_biguint(&self) -> BigUint {
        let mut x: BigUint = 0.to_biguint().unwrap();
        for b in self.inner.clone() {
            x = b + x * 256.to_biguint().unwrap()
        }
        x
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        assert_eq!(ByteArray::new().to_bytes(), [0]);
    }

    #[test]
    fn from_vec_bytes() {
        assert_eq!(ByteArray::from(&vec![]).to_bytes(), b"\x00");
        assert_eq!(
            ByteArray::from(&vec![10u8, 5u8, 4u8]).to_bytes(),
            [10, 5, 4]
        );
    }

    #[test]
    fn from_bytes() {
        assert_eq!(ByteArray::from_bytes(&[]).to_bytes(), b"\x00");
        assert_eq!(
            ByteArray::from_bytes(&[10u8, 5u8, 4u8]).to_bytes(),
            [10, 5, 4]
        );
    }

    #[test]
    fn from_biguint() {
        assert_eq!(
            ByteArray::from(&0.to_biguint().unwrap()).to_bytes(),
            b"\x00"
        );
        assert_eq!(
            ByteArray::from(&3.to_biguint().unwrap()).to_bytes(),
            b"\x03"
        );
        assert_eq!(
            ByteArray::from(&23591.to_biguint().unwrap()).to_bytes(),
            b"\x5c\x27"
        );
        assert_eq!(
            ByteArray::from(&23592.to_biguint().unwrap()).to_bytes(),
            b"\x5c\x28"
        );
        assert_eq!(
            ByteArray::from(&4294967295u64.to_biguint().unwrap()).to_bytes(),
            b"\xff\xff\xff\xff"
        );
        assert_eq!(
            ByteArray::from(&4294967296u64.to_biguint().unwrap()).to_bytes(),
            b"\x01\x00\x00\x00\x00"
        );
    }

    #[test]
    fn to_biguint() {
        assert_eq!(
            ByteArray::from_bytes(b"\x00").into_biguint(),
            0.to_biguint().unwrap()
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x03").into_biguint(),
            3.to_biguint().unwrap()
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x5c\x27").into_biguint(),
            23591.to_biguint().unwrap()
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x5c\x28").into_biguint(),
            23592.to_biguint().unwrap()
        );
        assert_eq!(
            ByteArray::from_bytes(b"\xff\xff\xff\xff").into_biguint(),
            4294967295u64.to_biguint().unwrap()
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x01\x00\x00\x00\x00").into_biguint(),
            4294967296u64.to_biguint().unwrap()
        );
    }
}
