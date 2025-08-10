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

use super::{FileReadMode, FileStructureError, FileStructureErrorImpl, FileType, GetFileNameTrait};
use crate::data_structures::{VerifierDataDecode, VerifierDataToTypeTrait, VerifierDataType};
use glob::glob;
use roxmltree::Document;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

#[derive(Clone)]
pub struct File<D: VerifierDataDecode + VerifierDataToTypeTrait> {
    path: PathBuf,
    cache: OnceLock<Arc<D>>,
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
            cache: OnceLock::new(),
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
    pub fn decode_verifier_data(&self) -> Result<Arc<D>, FileStructureError> {
        if let Some(cache) = self.cache.get() {
            return Ok(cache.clone());
        }
        if !self.exists() {
            return Err(FileStructureError::from(
                FileStructureErrorImpl::PathNotFile(self.path()),
            ));
        }
        let res = self.read_data().map(Arc::new).map_err(|e| {
            FileStructureErrorImpl::ReadDataDecoding {
                source: Box::new(e),
            }
        })?;
        if FileReadMode::from(&Self::data_type()) == FileReadMode::Cache {
            self.cache.get_or_init(|| res.clone());
        }
        Ok(res)
    }

    fn read_data(&self) -> Result<D, FileStructureError> {
        let file_type = FileType::from(&Self::data_type());
        let mode = FileReadMode::from(&Self::data_type());
        match mode {
            FileReadMode::Memory | FileReadMode::Cache => {
                let s = std::fs::read_to_string(self.path()).map_err(|e| {
                    FileStructureErrorImpl::IO {
                        path: self.path(),
                        source: e,
                    }
                })?;
                match file_type {
                    FileType::Json => D::decode_json(s.as_str())
                        .map_err(|e| FileStructureErrorImpl::ReadDataStructure {
                            msg: "Decoding json",
                            path: self.path(),
                            source: Box::new(e),
                        })
                        .map_err(FileStructureError::from),
                    FileType::Xml => {
                        let doc = Document::parse(&s).map_err(|e| {
                            FileStructureErrorImpl::ParseRoXML {
                                path: self.path(),
                                source: e,
                            }
                        })?;
                        D::decode_xml(&doc)
                            .map_err(|e| FileStructureErrorImpl::ReadDataStructure {
                                msg: "Decoding xml",
                                path: self.path(),
                                source: Box::new(e),
                            })
                            .map_err(FileStructureError::from)
                    }
                }
            }
            FileReadMode::Streaming => match file_type {
                FileType::Json => D::stream_json(self.path().as_path())
                    .map_err(|e| FileStructureErrorImpl::ReadDataStructure {
                        msg: "Streaming json",
                        path: self.path(),
                        source: Box::new(e),
                    })
                    .map_err(FileStructureError::from),
                FileType::Xml => D::stream_xml(self.path().as_path())
                    .map_err(|e| FileStructureErrorImpl::ReadDataStructure {
                        msg: "Streaming xml",
                        path: self.path(),
                        source: Box::new(e),
                    })
                    .map_err(FileStructureError::from),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{test_datasets_context_path, test_datasets_tally_path};
    use crate::data_structures::context::control_component_public_keys_payload::ControlComponentPublicKeysPayload;
    use crate::data_structures::tally::ech_0222::ECH0222;
    use crate::data_structures::ElectionEventContextPayload;

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
        let f = File::<ECH0222>::new(&location, None);
        assert!(f.exists());
        assert_eq!(f.location(), location);
        assert_eq!(
            f.path(),
            location.join("eCH-0222_v4-0_NE_20231124_TT05.xml")
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
