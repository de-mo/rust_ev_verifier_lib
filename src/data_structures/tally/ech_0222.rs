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
    super::{DataStructureError, VerifierDataDecode},
    VerifierTallyDataType,
};
use crate::{
    data_structures::{xml::XMLData, VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifiyXMLSignatureTrait},
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ECH0222 {
    inner: XMLData<ECH0222Data, DataStructureError>,
}

#[derive(Debug, Clone)]
pub struct ECH0222Data {}

impl ECH0222 {
    pub fn get_data(&self) -> Result<Arc<ECH0222Data>, DataStructureError> {
        self.inner.get_data()
    }

    pub fn unwrap_data(&self) -> Arc<ECH0222Data> {
        self.get_data().unwrap()
    }
}

impl VerifierDataToTypeTrait for ECH0222 {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::ECH0222)
    }
}

fn decode_xml(_s: &str) -> Result<ECH0222Data, DataStructureError> {
    Ok(ECH0222Data {})
}

impl VerifierDataDecode for ECH0222 {
    fn decode_xml<'a>(s: String) -> Result<Self, DataStructureError> {
        Ok(Self {
            inner: XMLData::new(s.as_str(), decode_xml),
        })
    }
}

impl<'a> VerifiyXMLSignatureTrait<'a> for ECH0222 {
    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmTally)
    }

    fn get_data_str(&self) -> Option<Arc<String>> {
        self.inner.get_raw()
    }
}

impl<'a> VerifiySignatureTrait<'a> for ECH0222 {
    fn verifiy_signature(
        &'a self,
        keystore: &crate::direct_trust::Keystore,
    ) -> Result<bool, crate::direct_trust::VerifySignatureError> {
        self.verifiy_xml_signature(keystore)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{get_keystore, test_datasets_tally_path};
    use std::fs;

    fn get_data_res() -> Result<ECH0222, DataStructureError> {
        ECH0222::decode_xml(
            fs::read_to_string(
                test_datasets_tally_path().join("eCH-0222_v3-0_NE_20231124_TT05.xml"),
            )
            .unwrap(),
        )
    }
    #[test]
    fn read_data_set() {
        let data_res = get_data_res();
        assert!(data_res.is_ok(), "{:?}", data_res.unwrap_err());
    }

    #[test]
    fn verify_signature() {
        let data = get_data_res().unwrap();
        let ks = get_keystore();
        let sign_validate_res = data.verify_signatures(&ks);
        for r in sign_validate_res {
            if r.is_err() {
                println!("error validating signature: {:?}", r.as_ref().unwrap_err())
            }
            assert!(r.is_ok());
            assert!(r.unwrap())
        }
    }
}
