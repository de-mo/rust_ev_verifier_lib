use super::super::{
    xml::{hashable::XMLFileHashable, SchemaKind},
    VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use rust_ev_crypto_primitives::{ByteArray, HashableMessage, RecursiveHashTrait};
use std::path::{Path, PathBuf};


#[derive(Debug, Clone)]
pub struct ECH0222 {pub path: PathBuf,}

impl VerifierDataDecode for ECH0222 {
    fn from_xml_file(p: &Path) -> anyhow::Result<Self> {
        Ok(ECH0222 {
            path: p.to_path_buf(),
        })
    }
}

impl<'a> VerifiySignatureTrait<'a> for ECH0222 {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Ech0222);
        let hash = hashable.try_hash()?;
        Ok(HashableMessage::Hashed(hash))
    }

    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("eCH 0222")]
    }

    fn get_certificate_authority(&self) -> anyhow::Result<String> {
        Ok(String::from(CertificateAuthority::Canton))
    }

    fn get_signature(&self) -> ByteArray {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_dataset_tally_path;

    #[test]
    fn read_data_set() {
        let path = test_dataset_tally_path()
            .join("tally")
            .join("eCH-0222_Post_E2E_DEV.xml");
        let ech_0222 = ECH0222::from_xml_file(&path);
        assert!(ech_0222.is_ok())
    }
}
