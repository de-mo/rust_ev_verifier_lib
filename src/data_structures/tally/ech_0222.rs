use super::super::VerifierDataDecode;
use roxmltree::Document;
use rust_ev_crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};

#[derive(Debug, Clone)]
pub struct ECH0222 {}

impl VerifierDataDecode for ECH0222 {
    fn from_roxmltree<'a>(_: &'a Document<'a>) -> anyhow::Result<Self> {
        Ok(ECH0222 {})
    }
}

impl<'a> VerifiySignatureTrait<'a> for ECH0222 {
    type Error=anyhow::Error;

    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, Self::Error> {
        //let hashable = XMLFileHashable::new(&self.path, &SchemaKind::config);
        //let hash = hashable.try_hash()?;
        //Ok(HashableMessage::Hashed(hash))
        todo!()
    }

    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("eCH 0222")]
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
    use crate::config::test::test_dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = test_dataset_tally_path()
            .join("tally")
            .join("eCH-0222_Post_E2E_DEV.xml");
        let xml = fs::read_to_string(path).unwrap();
        let config = ECH0222::from_roxmltree(&Document::parse(&xml).unwrap());
        assert!(config.is_ok())
    }
}
