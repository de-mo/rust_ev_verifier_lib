use super::super::{
    error::{DeserializeError, DeserializeErrorType},
    DataStructureTrait,
};
use crate::{
    crypto_primitives::{
        byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::RecursiveHashable,
        signature::VerifiySignatureTrait,
    },
    error::{create_result_with_error, create_verifier_error, VerifierError},
};
use roxmltree::Document;

#[derive(Debug, Clone)]
pub struct ElectionEventConfiguration {
    pub voter_total: usize,
}

impl DataStructureTrait for ElectionEventConfiguration {
    fn from_roxmltree<'a>(doc: &'a Document<'a>) -> Result<Self, DeserializeError> {
        let node = match doc.descendants().find(|e| e.has_tag_name("voterTotal")) {
            Some(n) => n,
            None => {
                return create_result_with_error!(
                    DeserializeErrorType::XMLError,
                    "cannot find tag voterTotal"
                )
            }
        };
        let voter_total = match node.text() {
            Some(t) => t.parse::<usize>().map_err(|e| {
                create_verifier_error!(
                    DeserializeErrorType::XMLError,
                    "Test in voterTotal is not a number",
                    e
                )
            })?,
            None => {
                return create_result_with_error!(
                    DeserializeErrorType::XMLError,
                    "cannot read test in tag voterTotal"
                )
            }
        };
        Ok(ElectionEventConfiguration { voter_total })
    }
}

impl From<&ElectionEventConfiguration> for RecursiveHashable {
    fn from(_: &ElectionEventConfiguration) -> Self {
        todo!()
    }
}

impl VerifiySignatureTrait<'_> for ElectionEventConfiguration {
    fn get_context_data(&self) -> RecursiveHashable {
        RecursiveHashable::from(&"configuration".to_string())
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
            .join("dataset-setup1")
            .join("setup")
            .join("configuration-anonymized.xml");
        let xml = fs::read_to_string(&path).unwrap();
        let config = ElectionEventConfiguration::from_string(&xml, &FileType::Xml);
        assert!(config.is_ok())
    }
}
