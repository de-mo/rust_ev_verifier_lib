use super::{FileReadMode, FileStructureError, FileType, GetFileNameTrait};
use crate::data_structures::{
    ControlComponentBallotBoxPayload, ControlComponentCodeSharesPayload,
    ControlComponentPublicKeysPayload, ControlComponentShufflePayload, DatasetType, EVotingDecrypt,
    ElectionEventConfiguration, ElectionEventContextPayload, SetupComponentPublicKeysPayload,
    SetupComponentTallyDataPayload, SetupComponentVerificationDataPayload,
    TallyComponentShufflePayload, TallyComponentVotesPayload, VerifierContextData,
    VerifierContextDataType, VerifierData, VerifierDataDecode, VerifierDataType, VerifierSetupData,
    VerifierSetupDataType, VerifierTallyData, VerifierTallyDataType, ECH0110, ECH0222,
};
use glob::glob;
use roxmltree::Document;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct File {
    path: PathBuf,
    data_type: VerifierDataType,
}

macro_rules! create_file {
    ($l: expr, $p: ident, $s: expr) => {
        File::new(&$l, &VerifierDataType::$p($s), None)
    };
    ($l: expr, $p: ident, $s: expr, $n: expr) => {
        File::new(&$l, &VerifierDataType::$p($s), Some($n))
    };
}
pub(crate) use create_file;

macro_rules! read_data_call {
    ($s: expr, $t: expr, $p: ident, $e_p: ident, $e_d: ident) => {
        $s.read_data::<$p>(&FileType::from($t), &FileReadMode::from($t))
            .map($e_p::$p)
            .map(VerifierData::$e_d)
    };
}

impl File {
    pub fn new(location: &Path, data_type: &VerifierDataType, file_nb: Option<usize>) -> Self {
        let name = data_type.get_file_name(file_nb);
        let mut path = location.join(&name);
        if name.contains('*') {
            let r_entries = glob(path.as_os_str().to_str().unwrap());
            if let Ok(entries) = r_entries {
                let p = entries.last();
                if let Some(p_f) = p {
                    path = location.join(p_f.unwrap().file_name().unwrap());
                };
            }
        }
        File {
            path,
            data_type: data_type.clone(),
        }
    }

    
    pub fn location(&self) -> PathBuf {
        self.path.parent().unwrap().to_path_buf()
    }

    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_file()
    }

    pub fn path(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    pub fn path_to_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

    pub fn get_verifier_data(&self) -> Result<VerifierData, FileStructureError> {
        if !self.exists() {
            return Err(FileStructureError::PathNotFile(self.path()));
        }
        match self.data_type {
            DatasetType::Context(t) => match t {
                VerifierContextDataType::ElectionEventContextPayload => read_data_call!(
                    self,
                    t,
                    ElectionEventContextPayload,
                    VerifierContextData,
                    Context
                ),
                VerifierContextDataType::SetupComponentPublicKeysPayload => read_data_call!(
                    self,
                    t,
                    SetupComponentPublicKeysPayload,
                    VerifierContextData,
                    Context
                ),
                VerifierContextDataType::ControlComponentPublicKeysPayload => read_data_call!(
                    self,
                    t,
                    ControlComponentPublicKeysPayload,
                    VerifierContextData,
                    Context
                ),
                VerifierContextDataType::SetupComponentTallyDataPayload => read_data_call!(
                    self,
                    t,
                    SetupComponentTallyDataPayload,
                    VerifierContextData,
                    Context
                ),
                VerifierContextDataType::ElectionEventConfiguration => read_data_call!(
                    self,
                    t,
                    ElectionEventConfiguration,
                    VerifierContextData,
                    Context
                ),
            },
            DatasetType::Setup(t) => match t {
                VerifierSetupDataType::SetupComponentVerificationDataPayload => read_data_call!(
                    self,
                    t,
                    SetupComponentVerificationDataPayload,
                    VerifierSetupData,
                    Setup
                ),
                VerifierSetupDataType::ControlComponentCodeSharesPayload => read_data_call!(
                    self,
                    t,
                    ControlComponentCodeSharesPayload,
                    VerifierSetupData,
                    Setup
                ),
            },
            DatasetType::Tally(t) => match t {
                VerifierTallyDataType::EVotingDecrypt => {
                    read_data_call!(self, t, EVotingDecrypt, VerifierTallyData, Tally)
                }
                VerifierTallyDataType::ECH0110 => {
                    read_data_call!(self, t, ECH0110, VerifierTallyData, Tally)
                }
                VerifierTallyDataType::ECH0222 => {
                    read_data_call!(self, t, ECH0222, VerifierTallyData, Tally)
                }
                VerifierTallyDataType::TallyComponentVotesPayload => {
                    read_data_call!(
                        self,
                        t,
                        TallyComponentVotesPayload,
                        VerifierTallyData,
                        Tally
                    )
                }
                VerifierTallyDataType::TallyComponentShufflePayload => {
                    read_data_call!(
                        self,
                        t,
                        TallyComponentShufflePayload,
                        VerifierTallyData,
                        Tally
                    )
                }
                VerifierTallyDataType::ControlComponentBallotBoxPayload => {
                    read_data_call!(
                        self,
                        t,
                        ControlComponentBallotBoxPayload,
                        VerifierTallyData,
                        Tally
                    )
                }
                VerifierTallyDataType::ControlComponentShufflePayload => {
                    read_data_call!(
                        self,
                        t,
                        ControlComponentShufflePayload,
                        VerifierTallyData,
                        Tally
                    )
                }
            },
        }
    }

    fn read_data<D: VerifierDataDecode>(
        &self,
        file_type: &FileType,
        mode: &FileReadMode,
    ) -> Result<D, FileStructureError> {
        match mode {
            FileReadMode::Memory => {
                let s =
                    std::fs::read_to_string(self.path()).map_err(|e| FileStructureError::IO {
                        path: self.path(),
                        source: e,
                    })?;
                match file_type {
                    FileType::Json => D::decode_json(s.as_str()).map_err(|e| {
                        FileStructureError::ReadDataStructure {
                            path: self.path(),
                            source: e,
                        }
                    }),
                    FileType::Xml => {
                        let doc =
                            Document::parse(&s).map_err(|e| FileStructureError::ParseRoXML {
                                msg: format!(
                                    "Cannot parse content of xml file {}",
                                    self.path_to_str()
                                ),
                                source: e,
                            })?;
                        D::decode_xml(&doc).map_err(|e| FileStructureError::ReadDataStructure {
                            path: self.path(),
                            source: e,
                        })
                    }
                }
            }
            FileReadMode::Streaming => match file_type {
                FileType::Json => D::stream_json(self.path().as_path()).map_err(|e| {
                    FileStructureError::ReadDataStructure {
                        path: self.path(),
                        source: e,
                    }
                }),
                FileType::Xml => D::stream_xml(self.path().as_path()).map_err(|e| {
                    FileStructureError::ReadDataStructure {
                        path: self.path(),
                        source: e,
                    }
                }),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{test_datasets_context_path, test_datasets_tally_path};
    use crate::data_structures::{
        context::VerifierContextDataType, tally::VerifierTallyDataType, VerifierContextDataTrait,
        VerifierDataType,
    };

    #[test]
    fn test_file() {
        let location = test_datasets_context_path();
        let f = File::new(
            &location,
            &VerifierDataType::Context(VerifierContextDataType::ElectionEventContextPayload),
            None,
        );
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(f.path(), location.join("electionEventContextPayload.json"));
        let data = f.get_verifier_data().unwrap();
        assert!(data.is_context());
        let enc_data = data.election_event_context_payload();
        assert!(enc_data.is_some())
    }

    #[test]
    fn test_file_macro() {
        let location = test_datasets_context_path();
        let f = create_file!(
            &location,
            Context,
            VerifierContextDataType::ElectionEventContextPayload
        );
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(f.path(), location.join("electionEventContextPayload.json"));
        let data = f.get_verifier_data().unwrap();
        assert!(data.is_context());
        let enc_data = data.election_event_context_payload();
        assert!(enc_data.is_some())
    }

    #[test]
    fn test_file_not_exist() {
        let location = test_datasets_context_path().join("toto");
        let f = File::new(
            &location,
            &VerifierDataType::Context(VerifierContextDataType::ElectionEventContextPayload),
            None,
        );
        assert!(!f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(f.path(), location.join("electionEventContextPayload.json"));
        let data = f.get_verifier_data();
        assert!(data.is_err())
    }

    #[test]
    fn test_file_with_nb() {
        let location = test_datasets_context_path();
        let f = File::new(
            &location,
            &VerifierDataType::Context(VerifierContextDataType::ControlComponentPublicKeysPayload),
            Some(2),
        );
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(
            f.path(),
            location.join("controlComponentPublicKeysPayload.2.json")
        );
        let data = f.get_verifier_data().unwrap();
        assert!(data.is_context());
    }

    #[test]
    fn test_file_with_astrerix() {
        let location = test_datasets_tally_path();
        let f = File::new(
            &location,
            &VerifierDataType::Tally(VerifierTallyDataType::ECH0110),
            None,
        );
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(
            f.path(),
            location.join("eCH-0110_v4-0_NE_20231124_TT05.xml")
        );
    }

    #[test]
    fn test_file_with_nb_not_exist() {
        let location = test_datasets_context_path();
        let f = File::new(
            &location,
            &VerifierDataType::Context(VerifierContextDataType::ControlComponentPublicKeysPayload),
            Some(6),
        );
        assert!(!f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(
            f.path(),
            location.join("controlComponentPublicKeysPayload.6.json")
        );
        let data = f.get_verifier_data();
        assert!(data.is_err());
    }

    #[test]
    fn test_file_with_nb_macro() {
        let location = test_datasets_context_path();
        let f = create_file!(
            &location,
            Context,
            VerifierContextDataType::ControlComponentPublicKeysPayload,
            2
        );
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(
            f.path(),
            location.join("controlComponentPublicKeysPayload.2.json")
        );
        let data = f.get_verifier_data().unwrap();
        assert!(data.is_context());
    }
}
