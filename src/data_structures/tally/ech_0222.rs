use super::{
    super::{
        xml::{hashable::XMLFileHashable, SchemaKind},
        DataStructureError, VerifierDataDecode,
    },
    VerifierTallyDataType,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    ByteArray, HashableMessage, RecursiveHashTrait,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ECH0222 {
    pub path: PathBuf,
}

impl VerifierDataToTypeTrait for ECH0222 {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::ECH0222)
    }
}

impl VerifierDataDecode for ECH0222 {
    fn stream_xml(p: &Path) -> Result<Self, DataStructureError> {
        Ok(ECH0222 {
            path: p.to_path_buf(),
        })
    }
}

impl<'a> VerifiySignatureTrait<'a> for ECH0222 {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Ech0222, "eCH-0222:extension");
        let hash = hashable
            .recursive_hash()
            .map_err(|e| VerifySignatureError::XMLError {
                msg: String::default(),
                source: e,
            })?;
        Ok(HashableMessage::Hashed(hash))
    }

    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("eCH 0222")]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmTally)
    }

    fn get_signature(&self) -> ByteArray {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_datasets_tally_path;

    #[test]
    fn read_data_set() {
        let path = test_datasets_tally_path()
            .join("tally")
            .join("eCH-0222_v1-0_NE_20231124_TT05.xml");
        let ech_0222 = ECH0222::stream_xml(&path);
        assert!(ech_0222.is_ok())
    }
}
