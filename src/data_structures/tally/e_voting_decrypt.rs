use super::super::{error::DeserializeError, DataStructureTrait};
use crate::crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use roxmltree::Document;

#[derive(Debug, Clone)]
pub struct EVotingDecrypt {}

impl DataStructureTrait for EVotingDecrypt {
    fn from_roxmltree<'a>(_: &'a Document<'a>) -> Result<Self, DeserializeError> {
        Ok(EVotingDecrypt {})
    }
}

impl<'a> From<&'a EVotingDecrypt> for HashableMessage<'a> {
    fn from(_: &'a EVotingDecrypt) -> Self {
        todo!()
    }
}

impl<'a> VerifiySignatureTrait<'a> for EVotingDecrypt {
    fn get_context_data(&self) -> &'static str {
        "evoting decrypt"
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
            .join("evoting-decrypt_Post_E2E_DEV.xml");
        let xml = fs::read_to_string(&path).unwrap();
        let config = EVotingDecrypt::from_string(&xml, &FileType::Xml);
        assert!(config.is_ok())
    }
}
