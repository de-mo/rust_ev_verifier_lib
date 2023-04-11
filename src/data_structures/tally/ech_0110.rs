use super::super::{error::DeserializeError, DataStructureTrait};
use crate::crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::RecursiveHashable,
    signature::VerifiySignatureTrait,
};
use roxmltree::Document;

#[derive(Debug, Clone)]
pub struct ECH0110 {}

impl DataStructureTrait for ECH0110 {
    fn from_roxmltree<'a>(_: &'a Document<'a>) -> Result<Self, DeserializeError> {
        Ok(ECH0110 {})
    }
}

impl From<&ECH0110> for RecursiveHashable {
    fn from(_: &ECH0110) -> Self {
        todo!()
    }
}

impl VerifiySignatureTrait<'_> for ECH0110 {
    fn get_context_data(&self) -> RecursiveHashable {
        RecursiveHashable::from(&"evoting decrypt".to_string())
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
    use crate::file_structure::FileType;
    use std::{fs, path::Path};

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset1")
            .join("tally")
            .join("eCH-0110_Post_E2E_DEV.xml");
        let xml = fs::read_to_string(&path).unwrap();
        let config = ECH0110::from_string(&xml, &FileType::Xml);
        assert!(config.is_ok())
    }
}
