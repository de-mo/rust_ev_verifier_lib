use super::super::{error::DeserializeError, DataStructureTrait};
use crate::{
    crypto_primitives::{
        direct_trust::CertificateAuthority, hashing::RecursiveHashable,
        signature::VerifiySignatureTrait,
    },
    data_structures::common_types::{Signature, SignatureTrait},
};
use roxmltree::Document;

#[derive(Debug, Clone)]
pub struct EVotingDecrypt {}

impl DataStructureTrait for EVotingDecrypt {
    fn from_roxmltree<'a>(_: &'a Document<'a>) -> Result<Self, DeserializeError> {
        Ok(EVotingDecrypt {})
    }
}

impl From<&EVotingDecrypt> for RecursiveHashable {
    fn from(_: &EVotingDecrypt) -> Self {
        todo!()
    }
}

impl VerifiySignatureTrait<'_> for EVotingDecrypt {
    fn get_context_data(&self) -> RecursiveHashable {
        RecursiveHashable::from(&"evoting decrypt".to_string())
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::Canton
    }
}

impl SignatureTrait for EVotingDecrypt {
    fn get_signature_struct(&self) -> &Signature {
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
