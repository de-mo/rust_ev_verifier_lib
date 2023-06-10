//! Implement signature functions

use super::{
    super::{byte_array::ByteArray, hashing::HashableMessage},
    OpensslError,
};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, PKeyRef, Public},
    rsa::Padding,
    sign::{RsaPssSaltlen, Verifier},
};

/// Verify the signature usinfg RSASSA-PSS as a signature algorithm
///
/// SHA-256 is used as the underlying hash function and hash for the mask generation function.
/// The mask generation function used for PSS is MGF1, defined in appendix B.2 of RFC8017.
/// The length of the salt is set to the length of the underlying hash function (i.e. 32 bytes).
/// The trailer field number is 1, which represents the trailer field with value 0xbc, in accordance with the same RFC.
pub fn verify(
    pkey: &PKeyRef<Public>,
    message: &HashableMessage,
    additional_context: &HashableMessage,
    signature: &ByteArray,
) -> Result<bool, OpensslError> {
    // With the next two lines, it is sure that the certificate is recognized as SRA certificate from openssl
    let pkey_temp = PKey::from_rsa(pkey.rsa().map_err(OpensslError::PublicKeyError)?)
        .map_err(OpensslError::PublicKeyError)?;
    let rsa_pkey = pkey_temp.as_ref();
    let mut verifier = Verifier::new(MessageDigest::sha256(), rsa_pkey).map_err(|e| {
        OpensslError::SignatureVerify {
            msg: "Error creating Sign Verifier".to_string(),
            source: e,
        }
    })?;
    // Necessary for the next functions
    verifier
        .set_rsa_padding(Padding::PKCS1_PSS)
        .map_err(|e| OpensslError::SignatureVerify {
            msg: "Error set_rsa_padding".to_string(),
            source: e,
        })?;
    verifier
        .set_rsa_mgf1_md(MessageDigest::sha256())
        .map_err(|e| OpensslError::SignatureVerify {
            msg: "Error set_rsa_mgf1_md".to_string(),
            source: e,
        })?;
    verifier
        .set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)
        .map_err(|e| OpensslError::SignatureVerify {
            msg: "Error set_rsa_pss_saltlen".to_string(),
            source: e,
        })?;
    let hash_vec: Vec<HashableMessage> = vec![message.to_owned(), additional_context.to_owned()];
    let h = HashableMessage::from(&hash_vec).recursive_hash();
    verifier
        .verify_oneshot(&signature.to_bytes(), &h.to_bytes())
        .map_err(|e| OpensslError::SignatureVerify {
            msg: "Error verify_oneshot".to_string(),
            source: e,
        })
}
