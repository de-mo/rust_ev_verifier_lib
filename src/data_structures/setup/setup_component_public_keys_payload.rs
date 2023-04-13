use super::{
    super::{
        common_types::{EncryptionGroup, ProofUnderline, SignatureJson},
        deserialize_seq_string_hex_to_seq_bigunit,
        error::{DeserializeError, DeserializeErrorType},
        implement_trait_data_structure, DataStructureTrait,
    },
    control_component_public_keys_payload::ControlComponentPublicKeys,
};
use crate::{
    crypto_primitives::{
        byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
        signature::VerifiySignatureTrait,
    },
    error::{create_verifier_error, VerifierError},
};
use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeysPayload {
    pub encryption_group: EncryptionGroup,
    pub election_event_id: String,
    pub setup_component_public_keys: SetupComponentPublicKeys,
    pub signature: SignatureJson,
}

implement_trait_data_structure!(SetupComponentPublicKeysPayload);

impl<'a> From<&'a SetupComponentPublicKeysPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentPublicKeysPayload) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.encryption_group));
        elts.push(Self::from(&value.setup_component_public_keys));
        Self::from(elts)
    }
}

impl<'a> VerifiySignatureTrait<'a> for SetupComponentPublicKeysPayload {
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("public keys"),
            HashableMessage::from("setup"),
            HashableMessage::from(&self.election_event_id),
        ]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::SdmConfig
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeys {
    pub combined_control_component_public_keys: Vec<ControlComponentPublicKeys>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub electoral_board_public_key: Vec<BigUint>,
    pub electoral_board_schnorr_proofs: Vec<ProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub election_public_key: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub choice_return_codes_encryption_public_key: Vec<BigUint>,
}

impl<'a> From<&'a SetupComponentPublicKeys> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentPublicKeys) -> Self {
        let mut elts = vec![];
        let cc: Vec<HashableMessage> = value
            .combined_control_component_public_keys
            .iter()
            .map(|e| Self::from(e))
            .collect();
        elts.push(Self::from(cc));
        elts.push(Self::from(&value.electoral_board_public_key));
        let el_sp: Vec<HashableMessage> = value
            .electoral_board_schnorr_proofs
            .iter()
            .map(|e| Self::from(e))
            .collect();
        elts.push(Self::from(el_sp));
        elts.push(Self::from(&value.election_public_key));
        elts.push(Self::from(&value.choice_return_codes_encryption_public_key));
        Self::from(elts)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::file_structure::FileType;
    use std::fs;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
            .join("setupComponentPublicKeysPayload.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = SetupComponentPublicKeysPayload::from_string(&json, &FileType::Json);
        assert!(r_eec.is_ok())
    }
}
