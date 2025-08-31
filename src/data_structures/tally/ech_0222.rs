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

use std::{rc::Rc, sync::Arc};

use super::{
    super::{DataStructureError, VerifierDataDecode},
    VerifierTallyDataType,
};
use crate::{
    data_structures::{VerifierDataToTypeTrait, VerifierDataType},
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifiyXMLSignatureTrait},
};

#[derive(Debug, Clone)]
pub struct ECH0222 {
    pub data: Arc<String>,
}

impl VerifierDataToTypeTrait for ECH0222 {
    fn data_type() -> VerifierDataType {
        VerifierDataType::Tally(VerifierTallyDataType::ECH0222)
    }
}

impl VerifierDataDecode for ECH0222 {
    fn decode_xml<'a>(s: String) -> Result<Self, DataStructureError> {
        Ok(ECH0222 { data: Arc::new(s) })
    }
}

impl<'a> VerifiyXMLSignatureTrait<'a> for ECH0222 {
    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::SdmTally)
    }

    fn get_data_str(&self) -> Option<Arc<String>> {
        Some(self.data.clone())
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
    use crate::config::test::test_datasets_tally_path;

    #[test]
    fn read_data_set() {
        let path = test_datasets_tally_path()
            .join("tally")
            .join("eCH-0222_v1-0_NE_20231124_TT05.xml");
        let ech_0222 = ECH0222::stream_xml(&path);
        assert!(ech_0222.is_ok())
    }
}
