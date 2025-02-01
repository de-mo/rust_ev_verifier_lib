use super::{FileReadMode, FileStructureError, FileType, GetFileNameTrait};
use crate::data_structures::{VerifierDataDecode, VerifierDataToTypeTrait, VerifierDataType};
use glob::glob;
use roxmltree::Document;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct File<D: VerifierDataDecode + VerifierDataToTypeTrait> {
    path: PathBuf,
    phantom: PhantomData<D>,
}

macro_rules! create_file {
    ($l: expr, $p: ident, $s: expr) => {
        File::new(&$l, None)
    };
    ($l: expr, $p: ident, $s: expr, $n: expr) => {
        File::new(&$l, Some($n))
    };
}
pub(crate) use create_file;

impl<D: VerifierDataDecode + VerifierDataToTypeTrait> File<D> {
    /// New file of given type in the location.
    ///
    /// `file_nb` defines if the file is part of a group, with the given number
    pub fn new(location: &Path, file_nb: Option<usize>) -> Self {
        let name = Self::data_type().get_file_name(file_nb);
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
            phantom: PhantomData,
        }
    }

    pub fn data_type() -> VerifierDataType {
        D::data_type()
    }

    /// Location of the file
    pub fn location(&self) -> PathBuf {
        self.path.parent().unwrap().to_path_buf()
    }

    /// Does the file exist
    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_file()
    }

    /// Path of the file
    pub fn path(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    /// Path of the file as string
    pub fn path_to_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

    /// Decode the verifier data containing the the file
    ///
    /// The generic `D` defines the type of the file
    pub fn decode_verifier_data(&self) -> Result<D, FileStructureError> {
        if !self.exists() {
            return Err(FileStructureError::PathNotFile(self.path()));
        }
        self.read_data()
    }

    fn read_data(&self) -> Result<D, FileStructureError> {
        let file_type = FileType::from(&Self::data_type());
        let mode = FileReadMode::from(&Self::data_type());
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
    use crate::data_structures::tally::ech_0110::ECH0110;
    use crate::data_structures::{ControlComponentPublicKeysPayload, ElectionEventContextPayload};

    #[test]
    fn test_file() {
        let location = test_datasets_context_path();
        let f = File::<ElectionEventContextPayload>::new(&location, None);
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(f.path(), location.join("electionEventContextPayload.json"));
        let data = f.decode_verifier_data();
        assert!(data.is_ok())
    }

    /*#[test]
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
        let data = f.decode_verifier_data::<ElectionEventContextPayload>();
        assert!(data.is_ok())
    }*/

    #[test]
    fn test_file_not_exist() {
        let location = test_datasets_context_path().join("toto");
        let f = File::<ElectionEventContextPayload>::new(&location, None);
        assert!(!f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(f.path(), location.join("electionEventContextPayload.json"));
        let data = f.decode_verifier_data();
        assert!(data.is_err())
    }

    #[test]
    fn test_file_with_nb() {
        let location = test_datasets_context_path();
        let f = File::<ControlComponentPublicKeysPayload>::new(&location, Some(2));
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(
            f.path(),
            location.join("controlComponentPublicKeysPayload.2.json")
        );
        let data = f.decode_verifier_data();
        assert!(data.is_ok());
    }

    #[test]
    fn test_file_with_astrerix() {
        let location = test_datasets_tally_path();
        let f = File::<ECH0110>::new(&location, None);
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
        let f = File::<ControlComponentPublicKeysPayload>::new(&location, Some(6));
        assert!(!f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(
            f.path(),
            location.join("controlComponentPublicKeysPayload.6.json")
        );
        let data = f.decode_verifier_data();
        assert!(data.is_err());
    }

    /*#[test]
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
        let data = f.decode_verifier_data::<ControlComponentPublicKeysPayload>();
        assert!(data.is_ok());
    }*/
}
