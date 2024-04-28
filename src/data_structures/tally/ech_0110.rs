use super::super::{
    xml::{hashable::XMLFileHashable, SchemaKind},
    VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use rust_ev_crypto_primitives::{ByteArray, HashableMessage, RecursiveHashTrait};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ECH0110 {
    pub path: PathBuf,
}

impl VerifierDataDecode for ECH0110 {
    fn from_xml_file(p: &Path) -> anyhow::Result<Self> {
        Ok(ECH0110 {
            path: p.to_path_buf(),
        })
    }
}

impl<'a> VerifiySignatureTrait<'a> for ECH0110 {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Ech0110, "eCH-0110:extension");
        let hash = hashable.try_hash()?;
        Ok(HashableMessage::Hashed(hash))
    }

    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("eCH 0110")]
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
            .join("eCH-0110_Post_E2E_DEV.xml");
        let ech_0110 = ECH0110::from_xml_file(&path);
        assert!(ech_0110.is_ok())
    }
}
