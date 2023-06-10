use super::super::{
    common_types::{EncryptionGroup, ExponentiatedEncryptedElement, Proof, SignatureJson},
    deserialize_seq_string_hex_to_seq_bigunit, implement_trait_verifier_data_json_decode,
    VerifierDataDecode,
};
use anyhow::anyhow;
use crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use num_bigint::BigUint;
use serde::Deserialize;

pub type ControlComponentCodeSharesPayload = Vec<ControlComponentCodeSharesPayloadInner>;

implement_trait_verifier_data_json_decode!(ControlComponentCodeSharesPayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeSharesPayloadInner {
    pub election_event_id: String,
    pub verification_card_set_id: String,
    pub chunk_id: usize,
    pub control_component_code_shares: Vec<ControlComponentCodeShare>,
    pub encryption_group: EncryptionGroup,
    pub node_id: usize,
    pub signature: SignatureJson,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ControlComponentCodeShare {
    pub verification_card_id: String,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub voter_choice_return_code_generation_public_key: Vec<BigUint>,
    #[serde(deserialize_with = "deserialize_seq_string_hex_to_seq_bigunit")]
    pub voter_vote_cast_return_code_generation_public_key: Vec<BigUint>,
    pub exponentiated_encrypted_partial_choice_return_codes: ExponentiatedEncryptedElement,
    pub encrypted_partial_choice_return_code_exponentiation_proof: Proof,
    pub exponentiated_encrypted_confirmation_key: ExponentiatedEncryptedElement,
    pub encrypted_confirmation_key_exponentiation_proof: Proof,
}

impl<'a> From<&'a ControlComponentCodeSharesPayloadInner> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentCodeSharesPayloadInner) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.election_event_id));
        elts.push(Self::from(&value.verification_card_set_id));
        elts.push(Self::from(&value.chunk_id));
        elts.push(Self::from(&value.encryption_group));
        let l: Vec<HashableMessage> = value
            .control_component_code_shares
            .iter()
            .map(|e| Self::from(e))
            .collect();
        elts.push(Self::from(l));
        elts.push(Self::from(&value.node_id));
        Self::from(elts)
    }
}

impl<'a> From<&'a ControlComponentCodeShare> for HashableMessage<'a> {
    fn from(value: &'a ControlComponentCodeShare) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.verification_card_id));
        elts.push(Self::from(
            &value.voter_choice_return_code_generation_public_key,
        ));
        elts.push(Self::from(
            &value.voter_vote_cast_return_code_generation_public_key,
        ));
        elts.push(Self::from(
            &value.exponentiated_encrypted_partial_choice_return_codes,
        ));
        elts.push(Self::from(
            &value.encrypted_partial_choice_return_code_exponentiation_proof,
        ));
        elts.push(Self::from(&value.exponentiated_encrypted_confirmation_key));
        elts.push(Self::from(
            &value.encrypted_confirmation_key_exponentiation_proof,
        ));
        Self::from(elts)
    }
}

impl<'a> VerifiySignatureTrait<'a> for ControlComponentCodeSharesPayloadInner {
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("encrypted code shares"),
            HashableMessage::from(&self.node_id),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.verification_card_set_id),
        ]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::get_ca_cc(&self.node_id).unwrap()
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::test::dataset_tally_path;
    use std::fs;

    #[test]
    fn read_data_set() {
        let path = dataset_tally_path()
            .join("setup")
            .join("verification_card_sets")
            .join("681B3488DE4CD4AD7FCED14B7A654169")
            .join("controlComponentCodeSharesPayload.0.json");
        let json = fs::read_to_string(&path).unwrap();
        let r_eec = ControlComponentCodeSharesPayload::from_json(&json);
        //println!("{:?}", r_eec);
        assert!(r_eec.is_ok())
    }
}
