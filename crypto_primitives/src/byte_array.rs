//! This module implement the struct ByteArray that is used over the

use super::num_bigint::ByteLength;
use data_encoding::{DecodeError, BASE32, BASE64, HEXUPPER};
use num_bigint::{BigUint, ToBigUint};
use num_traits::Pow;
use std::fmt::{Debug, Display};
use thiserror::Error;

/// ByteArray represent a byte of arrays
#[derive(Clone, PartialEq, Eq)]
pub struct ByteArray {
    inner: Vec<u8>,
}

/// Trait to encode in string in different bases
pub trait Encode {
    /// Code to baase16 according specifications
    fn base16_encode(&self) -> String;

    /// Code to baase32 according specifications
    fn base32_encode(&self) -> String;

    /// Code to baase64 according specifications
    fn base64_encode(&self) -> String;
}

/// Trait to decode from string in different bases
pub trait Decode: Sized {
    /// Code from string in base16 according specifications. The letters are in upper.
    fn base16_decode(s: &str) -> Result<Self, ByteArrayError>;

    /// Code from string in base32 according specifications.
    fn base32_decode(s: &str) -> Result<Self, ByteArrayError>;

    /// Code from string in base32 according specifications.
    fn base64_decode(s: &str) -> Result<Self, ByteArrayError>;
}

impl ByteArray {
    /// Create an ampty Bytearray (only woth 0)
    pub fn new() -> Self {
        ByteArray { inner: vec![0] }
    }

    /// ByteArray from a slice of bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        ByteArray::from(&Vec::from(bytes))
    }

    /// ByteArray into BigUint
    pub fn into_biguint(&self) -> BigUint {
        let mut x: BigUint = 0.to_biguint().unwrap();
        for b in self.inner.clone() {
            x = b + x * 256.to_biguint().unwrap()
        }
        x
    }

    /// Len of the ByteArray in bytes
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// ByteArray to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.clone()
    }

    /// Append other to self
    pub fn append(&mut self, other: &ByteArray) -> &ByteArray {
        self.inner.extend(other.inner.clone().into_iter());
        self
    }

    /// Create a bew ByteArray prepending a byte
    pub fn prepend_byte(&self, byte: u8) -> ByteArray {
        let mut res = ByteArray::from(&vec![byte]);
        res.append(self);
        res
    }

    /// Cut the byte array to given bit length according to the specifications
    pub fn cut_bit_length(&self, n: usize) -> Result<ByteArray, ByteArrayError> {
        if n < 1 || n > 8 * self.len() {
            return Err(ByteArrayError::CutToBitLengthIndexError {
                index: n,
                ba: self.clone(),
            });
        }
        let bs = self.to_bytes();
        println!("bs: {:?}", bs);
        let length = (n + 8 - 1) / 8;
        println!("length: {:?}", length);
        let offset = self.len() - length;
        println!("offset: {:?}", length);
        let mut arr: Vec<u8> = vec![];
        if n % 8 != 0 {
            println!("n % 8: {:?}", (n % 8));
            println!("2^(n % 8): {:?}", Pow::pow(2u8, n % 8));
            println!("2^(n % 8)-1: {:?}", Pow::pow(2u8, n % 8) - 1);
            println!("mask: {:?}", (Pow::pow(2u8, n % 8) - 1));
            arr.push(bs[offset] & (Pow::pow(2u8, n % 8) - 1));
        } else {
            arr.push(bs[offset])
        }
        for i in 1..length {
            println!("i: {:?}", i);
            println!("bs[offset+i]: {:?}", bs[offset + i]);
            arr.push(bs[offset + i])
        }
        Ok(ByteArray::from(&arr))
    }
}

#[derive(Error, Debug)]
pub enum ByteArrayError {
    #[error("Error decoding {orig} in base {base} caused by {source}")]
    DecodeError {
        orig: String,
        base: u8,
        source: DecodeError,
    },
    #[error("Error in cut_bit_length for {ba}: the index {index} must be between 1 and 8*{}", ba.len())]
    CutToBitLengthIndexError { index: usize, ba: ByteArray },
}

impl Encode for ByteArray {
    fn base16_encode(&self) -> String {
        HEXUPPER.encode(&self.inner)
    }

    fn base32_encode(&self) -> String {
        BASE32.encode(&self.inner)
    }

    fn base64_encode(&self) -> String {
        BASE64.encode(&self.inner)
    }
}

impl Decode for ByteArray {
    fn base16_decode(s: &str) -> Result<Self, ByteArrayError> {
        HEXUPPER
            .decode(s.as_bytes())
            .map_err(|e| ByteArrayError::DecodeError {
                orig: s.to_string(),
                base: 16,
                source: e,
            })
            .map(|r| Self::from(&r))
    }

    fn base32_decode(s: &str) -> Result<Self, ByteArrayError> {
        BASE32
            .decode(s.as_bytes())
            .map_err(|e| ByteArrayError::DecodeError {
                orig: s.to_string(),
                base: 32,
                source: e,
            })
            .map(|r| Self::from(&r))
    }

    fn base64_decode(s: &str) -> Result<Self, ByteArrayError> {
        BASE64
            .decode(s.as_bytes())
            .map_err(|e| ByteArrayError::DecodeError {
                orig: s.to_string(),
                base: 64,
                source: e,
            })
            .map(|r| Self::from(&r))
    }
}

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

impl Display for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base16_encode())
    }
}

impl From<&BigUint> for ByteArray {
    fn from(value: &BigUint) -> Self {
        let byte_length = std::cmp::max(value.byte_length(), 1);
        let mut x = value.clone();
        let mut d: Vec<u8> = Vec::new();
        for _i in 0..byte_length {
            d.insert(0, (x.clone() % 256.to_biguint().unwrap()).to_bytes_le()[0]);
            x /= 256.to_biguint().unwrap();
        }
        ByteArray::from(&d)
    }
}

impl From<&usize> for ByteArray {
    fn from(value: &usize) -> Self {
        ByteArray::from(&BigUint::from(*value))
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

impl From<&String> for ByteArray {
    fn from(s: &String) -> Self {
        ByteArray::from_bytes(s.as_bytes())
    }
}

impl From<&str> for ByteArray {
    fn from(s: &str) -> Self {
        ByteArray::from(&s.to_string())
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
        assert_eq!(
            ByteArray::from_bytes(b"\x41\x42\x43").to_bytes(),
            [65, 66, 67]
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
    fn from_string() {
        assert_eq!(
            ByteArray::from(&"ABC".to_string()).to_bytes(),
            b"\x41\x42\x43"
        );
        assert_eq!(ByteArray::from(&"Ä".to_string()).to_bytes(), b"\xc3\x84");
        assert_eq!(
            ByteArray::from(&"1001".to_string()).to_bytes(),
            b"\x31\x30\x30\x31"
        );
        assert_eq!(ByteArray::from(&"1A".to_string()).to_bytes(), b"\x31\x41");
    }

    #[test]
    fn append() {
        let mut b = ByteArray::from_bytes(b"\x04\x03");
        b.append(&ByteArray::from_bytes(b"\x10\x11\x12"));
        assert_eq!(b, ByteArray::from_bytes(b"\x04\x03\x10\x11\x12"))
    }

    #[test]
    fn prepend_byte() {
        assert_eq!(
            ByteArray::from_bytes(b"\x03").prepend_byte(4u8),
            ByteArray::from_bytes(b"\x04\x03")
        )
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

    #[test]
    fn cut_bit_length() {
        assert_eq!(
            ByteArray::base64_decode("/w==")
                .unwrap()
                .cut_bit_length(1)
                .unwrap(),
            ByteArray::base64_decode("AQ==").unwrap()
        );
        assert_eq!(
            ByteArray::base64_decode("Dw==")
                .unwrap()
                .cut_bit_length(2)
                .unwrap(),
            ByteArray::base64_decode("Aw==").unwrap()
        );
        assert_eq!(
            ByteArray::base64_decode("/w==")
                .unwrap()
                .cut_bit_length(8)
                .unwrap(),
            ByteArray::base64_decode("/w==").unwrap()
        );
        assert_eq!(
            ByteArray::base64_decode("vu8=")
                .unwrap()
                .cut_bit_length(7)
                .unwrap(),
            ByteArray::base64_decode("bw==").unwrap()
        );
        assert_eq!(
            ByteArray::base64_decode("wP/u")
                .unwrap()
                .cut_bit_length(13)
                .unwrap(),
            ByteArray::base64_decode("H+4=").unwrap()
        );
        assert_eq!(
            ByteArray::base64_decode("q80=")
                .unwrap()
                .cut_bit_length(9)
                .unwrap(),
            ByteArray::base64_decode("Ac0=").unwrap()
        );
        assert!(ByteArray::from_bytes(b"10011").cut_bit_length(0).is_err());
        assert!(ByteArray::from_bytes(b"\x11").cut_bit_length(9).is_err());
    }

    #[test]
    fn base16_encode() {
        assert_eq!(ByteArray::from_bytes(b"").base16_encode(), "00");
        assert_eq!(ByteArray::from_bytes(b"\x41").base16_encode(), "41");
        assert_eq!(ByteArray::from_bytes(b"\x60").base16_encode(), "60");
        assert_eq!(ByteArray::from_bytes(b"\x00").base16_encode(), "00");
        assert_eq!(ByteArray::from_bytes(b"\x7f").base16_encode(), "7F");
        assert_eq!(ByteArray::from_bytes(b"\x80").base16_encode(), "80");
        assert_eq!(ByteArray::from_bytes(b"\xff").base16_encode(), "FF");
        assert_eq!(ByteArray::from_bytes(b"\x41\x00").base16_encode(), "4100");
        assert_eq!(
            ByteArray::from_bytes(b"\x01\x01\x01").base16_encode(),
            "010101"
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x7F\x00\xFE\x03").base16_encode(),
            "7F00FE03"
        );
    }

    #[test]
    fn base16_decode() {
        assert_eq!(ByteArray::base16_decode("00").unwrap().to_bytes(), b"\x00");
        assert_eq!(ByteArray::base16_decode("41").unwrap().to_bytes(), b"\x41");
        assert_eq!(ByteArray::base16_decode("60").unwrap().to_bytes(), b"\x60");
        assert_eq!(ByteArray::base16_decode("7F").unwrap().to_bytes(), b"\x7F");
        assert_eq!(ByteArray::base16_decode("80").unwrap().to_bytes(), b"\x80");
        assert_eq!(ByteArray::base16_decode("FF").unwrap().to_bytes(), b"\xff");
        assert_eq!(
            ByteArray::base16_decode("4100").unwrap().to_bytes(),
            b"\x41\x00"
        );
        assert_eq!(
            ByteArray::base16_decode("010101").unwrap().to_bytes(),
            b"\x01\x01\x01"
        );
        assert_eq!(
            ByteArray::base16_decode("7F00FE03").unwrap().to_bytes(),
            b"\x7F\x00\xFE\x03"
        );
        assert!(ByteArray::base16_decode("234G").is_err())
    }

    #[test]
    fn base32_encode() {
        assert_eq!(ByteArray::from_bytes(b"").base32_encode(), "AA======");
        assert_eq!(ByteArray::from_bytes(b"\x41").base32_encode(), "IE======");
        assert_eq!(ByteArray::from_bytes(b"\x60").base32_encode(), "MA======");
        assert_eq!(ByteArray::from_bytes(b"\x00").base32_encode(), "AA======");
        assert_eq!(ByteArray::from_bytes(b"\x7f").base32_encode(), "P4======");
        assert_eq!(ByteArray::from_bytes(b"\x80").base32_encode(), "QA======");
        assert_eq!(ByteArray::from_bytes(b"\xff").base32_encode(), "74======");
        assert_eq!(
            ByteArray::from_bytes(b"\x41\x00").base32_encode(),
            "IEAA===="
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x01\x01\x01").base32_encode(),
            "AEAQC==="
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x7F\x00\xFE\x03").base32_encode(),
            "P4AP4AY="
        );
    }

    #[test]
    fn base32_decode() {
        assert_eq!(
            ByteArray::base32_decode("AA======").unwrap().to_bytes(),
            b"\x00"
        );
        assert_eq!(
            ByteArray::base32_decode("IE======").unwrap().to_bytes(),
            b"\x41"
        );
        assert_eq!(
            ByteArray::base32_decode("MA======").unwrap().to_bytes(),
            b"\x60"
        );
        assert_eq!(
            ByteArray::base32_decode("P4======").unwrap().to_bytes(),
            b"\x7F"
        );
        assert_eq!(
            ByteArray::base32_decode("QA======").unwrap().to_bytes(),
            b"\x80"
        );
        assert_eq!(
            ByteArray::base32_decode("74======").unwrap().to_bytes(),
            b"\xff"
        );
        assert_eq!(
            ByteArray::base32_decode("IEAA====").unwrap().to_bytes(),
            b"\x41\x00"
        );
        assert_eq!(
            ByteArray::base32_decode("AEAQC===").unwrap().to_bytes(),
            b"\x01\x01\x01"
        );
        assert_eq!(
            ByteArray::base32_decode("P4AP4AY=").unwrap().to_bytes(),
            b"\x7F\x00\xFE\x03"
        );
        assert!(ByteArray::base32_decode("P4AP4AY").is_err())
    }

    #[test]
    fn base64_encode() {
        assert_eq!(ByteArray::from_bytes(b"").base64_encode(), "AA==");
        assert_eq!(ByteArray::from_bytes(b"\x41").base64_encode(), "QQ==");
        assert_eq!(ByteArray::from_bytes(b"\x60").base64_encode(), "YA==");
        assert_eq!(ByteArray::from_bytes(b"\x00").base64_encode(), "AA==");
        assert_eq!(ByteArray::from_bytes(b"\x7f").base64_encode(), "fw==");
        assert_eq!(ByteArray::from_bytes(b"\x80").base64_encode(), "gA==");
        assert_eq!(ByteArray::from_bytes(b"\xff").base64_encode(), "/w==");
        assert_eq!(ByteArray::from_bytes(b"\x41\x00").base64_encode(), "QQA=");
        assert_eq!(
            ByteArray::from_bytes(b"\x01\x01\x01").base64_encode(),
            "AQEB"
        );
        assert_eq!(
            ByteArray::from_bytes(b"\x7F\x00\xFE\x03").base64_encode(),
            "fwD+Aw=="
        );
    }

    #[test]
    fn base64_decode() {
        assert_eq!(
            ByteArray::base64_decode("AA==").unwrap().to_bytes(),
            b"\x00"
        );
        assert_eq!(
            ByteArray::base64_decode("QQ==").unwrap().to_bytes(),
            b"\x41"
        );
        assert_eq!(
            ByteArray::base64_decode("YA==").unwrap().to_bytes(),
            b"\x60"
        );
        assert_eq!(
            ByteArray::base64_decode("fw==").unwrap().to_bytes(),
            b"\x7F"
        );
        assert_eq!(
            ByteArray::base64_decode("gA==").unwrap().to_bytes(),
            b"\x80"
        );
        assert_eq!(
            ByteArray::base64_decode("/w==").unwrap().to_bytes(),
            b"\xff"
        );
        assert_eq!(
            ByteArray::base64_decode("QQA=").unwrap().to_bytes(),
            b"\x41\x00"
        );
        assert_eq!(
            ByteArray::base64_decode("AQEB").unwrap().to_bytes(),
            b"\x01\x01\x01"
        );
        assert_eq!(
            ByteArray::base64_decode("fwD+Aw==").unwrap().to_bytes(),
            b"\x7F\x00\xFE\x03"
        );
        assert!(ByteArray::base64_decode("fwD+Aw=").is_err())
    }
}
