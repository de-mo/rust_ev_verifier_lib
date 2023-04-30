use super::super::{error::DeserializeError, VerifierDataDecode};
use crate::crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use roxmltree::Document;

#[derive(Debug, Clone)]
pub struct ECH0110 {}

impl VerifierDataDecode for ECH0110 {
    fn from_roxmltree<'a>(_: &'a Document<'a>) -> Result<Self, DeserializeError> {
        Ok(ECH0110 {})
    }
}

impl<'a> From<&'a ECH0110> for HashableMessage<'a> {
    fn from(_: &'a ECH0110) -> Self {
        todo!()
    }
}

impl<'a> VerifiySignatureTrait<'a> for ECH0110 {
    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("eCH 0110")]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::Canton
    }
    fn get_signature(&self) -> ByteArray {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset1")
            .join("tally")
            .join("eCH-0110_Post_E2E_DEV.xml");
        let xml = fs::read_to_string(&path).unwrap();
        let config = ECH0110::from_roxmltree(&Document::parse(&xml).unwrap());
        assert!(config.is_ok())
    }
}
