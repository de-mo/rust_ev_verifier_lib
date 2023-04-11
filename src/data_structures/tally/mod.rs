pub mod e_voting_decrypt;
pub mod ech_0110;

use super::{error::DeserializeError, DataStructureTrait, VerifierDataTrait};
use crate::file_structure::FileType;
use enum_kinds::EnumKind;
use {e_voting_decrypt::EVotingDecrypt, ech_0110::ECH0110};

#[derive(Clone, EnumKind)]
#[enum_kind(VerifierTallyDataType)]
pub enum VerifierTallyData {
    EVotingDecrypt(EVotingDecrypt),
    ECH0110(ECH0110),
}

impl VerifierTallyDataType {
    pub fn get_file_type(&self) -> FileType {
        match self {
            Self::EVotingDecrypt => FileType::Xml,
            Self::ECH0110 => FileType::Xml,
        }
    }

    /// Read from String as json or xml
    ///
    /// All the types have to oimplement the trait [DataStructureTrait]
    pub fn verifier_data_from_file(
        &self,
        s: &String,
    ) -> Result<VerifierTallyData, DeserializeError> {
        match self {
            VerifierTallyDataType::EVotingDecrypt => {
                EVotingDecrypt::from_string(s, &self.get_file_type())
                    .map(|r| VerifierTallyData::EVotingDecrypt(r))
            }
            VerifierTallyDataType::ECH0110 => ECH0110::from_string(s, &self.get_file_type())
                .map(|r| VerifierTallyData::ECH0110(r)),
        }
    }
}

impl VerifierDataTrait for VerifierTallyData {
    fn e_voting_decrypt(&self) -> Option<&EVotingDecrypt> {
        if let VerifierTallyData::EVotingDecrypt(d) = self {
            return Some(d);
        }
        None
    }

    fn ech_110(&self) -> Option<&ECH0110> {
        if let VerifierTallyData::ECH0110(d) = self {
            return Some(d);
        }
        None
    }
}
