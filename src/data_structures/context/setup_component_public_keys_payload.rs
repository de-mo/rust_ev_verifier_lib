// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use super::{
    super::{
        common_types::{EncryptionParametersDef, SchnorrProof, Signature},
        deserialize_seq_string_base64_to_seq_integer, implement_trait_verifier_data_json_decode,
        DataStructureError, DataStructureErrorImpl, VerifierDataDecode,
    },
    control_component_public_keys_payload::ControlComponentPublicKeys,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{
        CertificateAuthority, VerifiyJSONSignatureTrait, VerifiySignatureTrait,
    },
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::EncryptionParameters, ByteArray, HashableMessage, VerifyDomainTrait,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeysPayload {
    #[serde(with = "EncryptionParametersDef")]
    pub encryption_group: EncryptionParameters,
    pub election_event_id: String,
    pub setup_component_public_keys: SetupComponentPublicKeys,
    pub signature: Signature,
}

impl VerifierDataToTypeTrait for SetupComponentPublicKeysPayload {
    fn data_type() -> crate::data_structures::VerifierDataType {
        VerifierDataType::Context(super::VerifierContextDataType::SetupComponentPublicKeysPayload)
    }
}

implement_trait_verifier_data_json_decode!(SetupComponentPublicKeysPayload);

impl VerifyDomainTrait<String> for SetupComponentPublicKeysPayload {}

impl<'a> From<&'a SetupComponentPublicKeysPayload> for HashableMessage<'a> {
    fn from(value: &'a SetupComponentPublicKeysPayload) -> Self {
        Self::from(vec![
            Self::from(&value.encryption_group),
            Self::from(&value.election_event_id),
            Self::from(&value.setup_component_public_keys),
        ])
    }
}

impl<'a> VerifiyJSONSignatureTrait<'a> for SetupComponentPublicKeysPayload {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, DataStructureError> {
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

impl<'a> VerifiySignatureTrait<'a> for SetupComponentPublicKeysPayload {
    fn verifiy_signature(
        &'a self,
        keystore: &crate::direct_trust::Keystore,
    ) -> Result<bool, crate::direct_trust::VerifySignatureError> {
        self.verifiy_json_signature(keystore)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetupComponentPublicKeys {
    pub combined_control_component_public_keys: Vec<ControlComponentPublicKeys>,
    #[serde(deserialize_with = "deserialize_seq_string_base64_to_seq_integer")]
    pub electoral_board_public_key: Vec<Integer>,
    pub electoral_board_schnorr_proofs: Vec<SchnorrProof>,
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
        elts.push(Self::from(value.electoral_board_public_key.as_slice()));
        let el_sp: Vec<HashableMessage> = value
            .electoral_board_schnorr_proofs
            .iter()
            .map(Self::from)
            .collect();
        elts.push(Self::from(el_sp));
        elts.push(Self::from(value.election_public_key.as_slice()));
        elts.push(Self::from(
            value.choice_return_codes_encryption_public_key.as_slice(),
        ));
        Self::from(elts)
    }
}

#[cfg(test)]
mod test {
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::EncodeTrait;

    use super::{
        super::super::test::{
            test_data_structure, test_data_structure_read_data_set,
            test_data_structure_verify_domain, test_data_structure_verify_signature,
        },
        *,
    };
    use crate::{
        config::test::{signing_keystore, test_datasets_context_path, CONFIG_TEST},
        direct_trust::Keystore,
    };
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
            .sign(&Keystore(
                signing_keystore(payload.get_certificate_authority().unwrap()).unwrap(),
            ))
            .unwrap();
        payload.signature.signature_contents = ByteArray::base64_encode(&signature).unwrap();
        let verif_res = payload.verifiy_signature(&CONFIG_TEST.keystore().unwrap());
        assert!(verif_res.is_ok());
        assert!(verif_res.unwrap());
    }
}
