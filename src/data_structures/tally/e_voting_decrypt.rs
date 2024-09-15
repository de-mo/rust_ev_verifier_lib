use super::super::{
    xml::{hashable::XMLFileHashable, SchemaKind},
    DataStructureError, VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError};
use rust_ev_crypto_primitives::{ByteArray, HashableMessage, RecursiveHashTrait};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EVotingDecrypt {
    pub path: PathBuf,
}

impl VerifierDataDecode for EVotingDecrypt {
    fn stream_xml(p: &Path) -> Result<Self, DataStructureError> {
        Ok(EVotingDecrypt {
            path: p.to_path_buf(),
        })
    }
}

impl<'a> VerifiySignatureTrait<'a> for EVotingDecrypt {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Box<VerifySignatureError>> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Decrypt, "signature");
        let hash = hashable
            .recursive_hash()
            .map_err(|e| VerifySignatureError::XMLError {
                msg: String::default(),
                source: e,
            })?;
        Ok(HashableMessage::Hashed(hash))
    }

    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("evoting decrypt")]
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
            .join("evoting-decrypt_NE_20231124_TT05.xml");
        let decrypt = EVotingDecrypt::stream_xml(&path);
        assert!(decrypt.is_ok())
    }
}
