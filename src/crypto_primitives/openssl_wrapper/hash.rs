//! Hash algorithms

use super::super::byte_array::ByteArray;
use openssl::md::Md;
use openssl::md_ctx::MdCtx;

/// Wrapper for SHA3-256
pub fn sha3_256(byte_array: &ByteArray) -> ByteArray {
    let mut ctx = MdCtx::new().unwrap();
    ctx.digest_init(Md::sha3_256()).unwrap();
    ctx.digest_update(&byte_array.to_bytes()).unwrap();
    let mut digest = [0; 32];
    ctx.digest_final(&mut digest).unwrap();
    ByteArray::from_bytes(&digest)
}

/// Wrapper for SHA256
pub fn sha256(byte_array: &ByteArray) -> ByteArray {
    let mut ctx = MdCtx::new().unwrap();
    ctx.digest_init(Md::sha256()).unwrap();
    ctx.digest_update(&byte_array.to_bytes()).unwrap();
    let mut digest = [0; 32];
    ctx.digest_final(&mut digest).unwrap();
    ByteArray::from_bytes(&digest)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sha3_256() {
        let e: Vec<u8> = vec![
            28u8, 158u8, 189u8, 108u8, 175u8, 2u8, 132u8, 10u8, 91u8, 43u8, 127u8, 15u8, 200u8,
            112u8, 236u8, 29u8, 177u8, 84u8, 136u8, 106u8, 233u8, 254u8, 98u8, 27u8, 130u8, 43u8,
            20u8, 253u8, 11u8, 245u8, 19u8, 214u8,
        ];
        assert_eq!(sha3_256(&ByteArray::from_bytes(b"\x41")).to_bytes(), e);
    }

    #[test]
    fn test_sha256() {
        assert_eq!(
            sha256(&ByteArray::from_bytes(b"Some Crypto Text")).to_bytes(),
            b"\x60\x78\x56\x38\x8a\xca\x5c\x51\x83\xc4\xd1\x4d\xc8\xf9\xcc\xf2\
            \xa5\x21\xb3\x10\x93\x72\xfa\xd6\x7c\x55\xf5\xc9\xe3\xd1\x83\x19"
        );
    }
}
