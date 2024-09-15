use super::super::{
    common_types::{EncryptionParametersDef, ExponentiatedEncryptedElement, Signature},
    deserialize_seq_string_base64_to_seq_integer, deserialize_string_base64_to_integer,
    implement_trait_verifier_data_json_decode, DataStructureError, VerifierDataDecode,
};
use crate::{
    data_structures::common_types::DecryptionProof,
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};
use rust_ev_crypto_primitives::Integer;
use rust_ev_crypto_primitives::{
    ByteArray, EncryptionParameters, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TallyComponentShufflePayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub ballot_box_id: String,
    pub verifiable_shuffle: VerifiableShuffle,
    pub verifiable_plaintext_decryption: VerifiablePlaintextDecryption,
    pub signature: Signature,
}
implement_trait_verifier_data_json_decode!(TallyComponentShufflePayload);

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiableShuffle {
    pub shuffled_ciphertexts: Vec<ExponentiatedEncryptedElement>,
    pub shuffle_argument: ShuffleArgument,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiablePlaintextDecryption {
    pub decrypted_votes: Vec<DecryptedVote>,
    pub decryption_proofs: Vec<DecryptionProof>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShuffleArgument {
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    #[serde(rename = "c_A")]
    pub c_a: Vec<Integer>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    #[serde(rename = "c_B")]
    pub c_b: Vec<Integer>,
    #[serde(rename = "productArgument")]
    pub product_argument: ProductArgument,
    #[serde(rename = "multiExponentiationArgument")]
    pub multi_exponentiation_argument: MultiExponentiationArgument,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductArgument {
    pub single_value_product_argument: SingleValueProductArgument,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SingleValueProductArgument {
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub c_d: Integer,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub c_delta: Integer,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    #[serde(rename = "c_Delta")]
    pub c_delta_upper: Integer,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub a_tilde: Vec<Integer>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub b_tilde: Vec<Integer>,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub r_tilde: Integer,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub s_tilde: Integer,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultiExponentiationArgument {
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    #[serde(rename = "c_A_0")]
    pub c_a_0: Integer,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    #[serde(rename = "c_B")]
    pub c_b: Vec<Integer>,
    #[serde(rename = "E")]
    pub e: Vec<ExponentiatedEncryptedElement>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub a: Vec<Integer>,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub r: Integer,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub b: Integer,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub s: Integer,
    #[serde(deserialize_with = "deserialize_string_base64_to_integer")]
    pub tau: Integer,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedVote {
    pub message: Vec<String>,
}

impl VerifyDomainTrait<String> for TallyComponentShufflePayload {}

impl<'a> From<&'a TallyComponentShufflePayload> for HashableMessage<'a> {
    fn from(value: &'a TallyComponentShufflePayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.ballot_box_id),
            Self::from(&value.verifiable_shuffle),
            Self::from(
                value
                    .verifiable_plaintext_decryption
                    .decrypted_votes
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
            Self::from(
                value
                    .verifiable_plaintext_decryption
                    .decryption_proofs
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
        ])
    }
}

impl<'a> From<&'a VerifiableShuffle> for HashableMessage<'a> {
    fn from(value: &'a VerifiableShuffle) -> Self {
        Self::from(vec![
            Self::from(
                value
                    .shuffled_ciphertexts
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
            Self::from(&value.shuffle_argument),
        ])
    }
}

/*
impl<'a> From<&'a VerifiablePlaintextDecryption> for HashableMessage<'a> {
    fn from(value: &'a VerifiablePlaintextDecryption) -> Self {
        Self::from(vec![
            Self::from(
                value
                    .decrypted_votes
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
            Self::from(
                value
                    .decryption_proofs
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
        ])
    }
}
 */

impl<'a> From<&'a ShuffleArgument> for HashableMessage<'a> {
    fn from(value: &'a ShuffleArgument) -> Self {
        Self::from(vec![
            Self::from(&value.c_a),
            Self::from(&value.c_b),
            Self::from(&value.product_argument),
            Self::from(&value.multi_exponentiation_argument),
        ])
    }
}

impl<'a> From<&'a ProductArgument> for HashableMessage<'a> {
    fn from(value: &'a ProductArgument) -> Self {
        Self::from(vec![Self::from(&value.single_value_product_argument)])
    }
}

impl<'a> From<&'a SingleValueProductArgument> for HashableMessage<'a> {
    fn from(value: &'a SingleValueProductArgument) -> Self {
        Self::from(vec![
            Self::from(&value.c_d),
            Self::from(&value.c_delta),
            Self::from(&value.c_delta_upper),
            Self::from(&value.a_tilde),
            Self::from(&value.b_tilde),
            Self::from(&value.r_tilde),
            Self::from(&value.s_tilde),
        ])
    }
}

impl<'a> From<&'a MultiExponentiationArgument> for HashableMessage<'a> {
    fn from(value: &'a MultiExponentiationArgument) -> Self {
        Self::from(vec![
            Self::from(&value.c_a_0),
            Self::from(&value.c_b),
            Self::from(
                value
                    .e
                    .iter()
                    .map(HashableMessage::from)
                    .collect::<Vec<Self>>(),
            ),
            Self::from(&value.a),
            Self::from(&value.r),
            Self::from(&value.b),
            Self::from(&value.s),
            Self::from(&value.tau),
        ])
    }
}

impl<'a> From<&'a DecryptedVote> for HashableMessage<'a> {
    fn from(value: &'a DecryptedVote) -> Self {
        Self::from(vec![Self::from(&value.message)])
    }
}

impl<'a> VerifiySignatureTrait<'a> for TallyComponentShufflePayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, VerifySignatureError> {
        Ok(HashableMessage::from(self))
    }

    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>> {
        vec![
            HashableMessage::from("shuffle"),
            HashableMessage::from("offline"),
            HashableMessage::from(&self.election_event_id),
            HashableMessage::from(&self.ballot_box_id),
        ]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmTally)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::super::test::{
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::config::test::{test_ballot_box_path, CONFIG_TEST};
    use std::fs;

    test_data_structure!(
        TallyComponentShufflePayload,
        "tallyComponentShufflePayload.json",
        test_ballot_box_path,
        "signature not workting"
    );
}
