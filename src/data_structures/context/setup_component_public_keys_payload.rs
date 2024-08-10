use super::{
    super::{
        common_types::{EncryptionParametersDef, ProofUnderline, Signature},
        deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
        VerifierDataDecode,
    },
    control_component_public_keys_payload::ControlComponentPublicKeys,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use anyhow::anyhow;
use rug::Integer;
use rust_ev_crypto_primitives::{
    ByteArray, EncryptionParameters, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeysPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub setup_component_public_keys: SetupComponentPublicKeys,
    pub signature: Signature,
}

implement_trait_verifier_data_json_decode!(SetupComponentPublicKeysPayload);

impl VerifyDomainTrait for SetupComponentPublicKeysPayload {}

impl<'a> From<&'a SetupComponentPublicKeysPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentPublicKeysPayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.setup_component_public_keys),
        ])
    }
}

impl<'a> VerifiySignatureTrait<'a> for SetupComponentPublicKeysPayload {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("public keys"),
            HashableMessage::from("setup"),
            HashableMessage::from(&self.election_event_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmConfig)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeys {
    pub combined_control_component_public_keys: Vec<ControlComponentPublicKeys>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub electoral_board_public_key: Vec<Integer>,
    pub electoral_board_schnorr_proofs: Vec<ProofUnderline>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub election_public_key: Vec<Integer>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub choice_return_codes_encryption_public_key: Vec<Integer>,
}

impl<'a> From<&'a SetupComponentPublicKeys> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentPublicKeys) -> Self {
        let mut elts = vec![];
        let cc: Vec<HashableMessage> = value
            .combined_control_component_public_keys
            .iter()
            .map(Self::from)
            .collect();
        elts.push(Self::from(cc));
        elts.push(Self::from(&value.electoral_board_public_key));
        let el_sp: Vec<HashableMessage> = value
            .electoral_board_schnorr_proofs
            .iter()
            .map(Self::from)
            .collect();
        elts.push(Self::from(el_sp));
        elts.push(Self::from(&value.election_public_key));
        elts.push(Self::from(&value.choice_return_codes_encryption_public_key));
        Self::from(elts)
    }
}

#[cfg(test)]
mod test {
    use rust_ev_crypto_primitives::Encode;

    use super::{
        super::super::test::{
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::{config::test::{signing_keystore, test_datasets_context_path, CONFIG_TEST}, direct_trust::Keystore};
    use std::fs;

    test_data_structure!(
        SetupComponentPublicKeysPayload,
        "setupComponentPublicKeysPayload.json",
        test_datasets_context_path
    );

    #[test]
    fn test_sign() {
        let mut payload = get_data_res().unwrap();
        let signature = payload
            .sign(&Keystore(signing_keystore(payload.get_certificate_authority().unwrap()).unwrap()))
            .unwrap();
        payload.signature.signature_contents = ByteArray::base64_encode(&signature);
        let verif_res = payload.verifiy_signature(&CONFIG_TEST.keystore().unwrap());
        assert!(verif_res.is_ok());
        assert!(verif_res.unwrap());
    }
}
