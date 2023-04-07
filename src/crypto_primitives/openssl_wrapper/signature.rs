//! Implement signature functions

use super::super::byte_array::ByteArray;
use openssl::{
    hash::MessageDigest,
    pkey::{PKeyRef, Public},
    sign::{RsaPssSaltlen, Verifier},
};

/// Verify the signature
/// TODO: Not working
pub fn verify(pkey: &PKeyRef<Public>, bytes: &ByteArray, signature: &ByteArray) -> bool {
    let mut verifier = Verifier::new(MessageDigest::sha384(), pkey).unwrap();
    //verifier.set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH);
    //println!(
    //    "{:?}",
    //    verifier.set_rsa_pss_saltlen(RsaPssSaltlen::custom(32))
    //);
    println!("{:?}", verifier.rsa_padding());
    println!(
        "{:?}",
        verifier
            .set_rsa_mgf1_md(MessageDigest::sha256())
            .unwrap_err()
    );
    //verifier.update(&bytes.to_bytes()).unwrap();
    verifier
        .verify_oneshot(&signature.to_bytes(), &bytes.to_bytes())
        .unwrap()
    //verifier.verify(&signature.to_bytes()).unwrap()
}
