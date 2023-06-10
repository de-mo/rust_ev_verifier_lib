//! Hash algorithms

use super::super::{byte_array::ByteArray, GROUP_PARAMETER_P_LENGTH};
use openssl::{
    hash::{hash_xof, MessageDigest},
    md::Md,
    md_ctx::MdCtx,
};

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

/// Wrapper for SHAKE128
pub fn shake128(byte_array: &ByteArray) -> ByteArray {
    let mut digest = [0; GROUP_PARAMETER_P_LENGTH / 8];
    hash_xof(
        MessageDigest::shake_128(),
        &byte_array.to_bytes(),
        digest.as_mut_slice(),
    )
    .unwrap();
    ByteArray::from_bytes(&digest)
}

#[cfg(test)]
mod test {
    use super::super::super::byte_array::Decode;
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

    #[test]
    fn test_shake128() {
        assert_eq!(
            shake128(&ByteArray::from_bytes(b"Some Crypto Text")),
            ByteArray::base16_decode(
                "76237D07F362A5A115FFA4830C75DD2AAEC818FB7236A35FF1643300FBA19D3EDF90490E8B05B26926E8A5FF2F2A4E16526CFA48B11B4A0FDBD8AA1AA124F578671ACEC8B51D154F1DFDE424D51EC26A8F8DFD2253550F3421E18E967509D1C26FCAB13410093C7EAF57B4C2CFD9505E99714BCA3D558B1E9DF23DEE0623854378E1B1077904D77192482DA475210EB023432FF9154D6F655020DE494AD8B3732BA89606C1E54E8260F49C4E6DB7B4408568192F72E08883C016681D9869F622393D824EE579BD70BF1C8B50ED90A1B6091A5E6A0E9C6DCBC57C6005BCED5F9E01FCEA3B4D96FE24475FF73A85A451DE673875D262D76CCFE583503F509EF957B25B2CD5FACA49D1C813AF6854FDC7B1A27A6645964975B37B6DF4D2441F8832361F2DADD48FE8C47D57D4F33AF1C85977ABE2A58D8CEB4366BD46801DC14837005A6496D4BDE2A154EA99BE6F4453501CF827E5108D067C8BFBDF8FB7BDC1D221DB11B34BD47AFC88B4D1DBEA7E967111631E22E40AF45735E1E789DBB7178F"
            )
            .unwrap()
        );
    }
}
