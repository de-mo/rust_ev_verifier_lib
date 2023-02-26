use super::VerifierDataType;
use crate::data_structures::verifier_data::{VerifierData, VerifierDataTrait};
use crate::data_structures::DataStructureTrait;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{FileStructureError, FileStructureErrorType, GetFileName};
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

pub struct File<T: DataStructureTrait> {
    path: PathBuf,
    pub data: VerifierData<T>,
}

pub struct FileGroup<T: DataStructureTrait> {
    location: PathBuf,
    data_type: VerifierDataType,
    files: Option<Box<HashMap<usize, File<T>>>>,
}

impl<T: DataStructureTrait> File<T> {
    pub fn new(location: &Path, data: VerifierData<T>) -> Self {
        File {
            path: location.join(data.get_data_type().get_file_name()),
            data: data,
        }
    }

    pub fn get_location(&self) -> &Path {
        self.path.parent().unwrap()
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn to_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

    fn read_data(&self) -> Result<String, FileStructureError> {
        fs::read_to_string(&self.path).map_err(|e| {
            create_verifier_error!(
                FileStructureErrorType::FileError,
                format!("Cannot read file \"{}\"", self.to_str()),
                e
            )
        })
    }

    pub fn get_data(&mut self) -> Result<&VerifierData<T>, FileStructureError> {
        if !self.exists() {
            return create_result_with_error!(
                FileStructureErrorType::FileError,
                format!("File \"{}\" does not exists", self.to_str())
            );
        }
        if self.data.is_none() {
            let s = match self.read_data() {
                Ok(s) => s,
                Err(e) => {
                    return create_result_with_error!(
                        FileStructureErrorType::FileError,
                        format!("Cannot read the content of the file \"{}\"", self.to_str()),
                        e
                    )
                }
            };
            match self.data.from_json(&s) {
                Ok(_) => (),
                Err(e) => {
                    return create_result_with_error!(
                        FileStructureErrorType::DataError,
                        format!("Content of the file \"{}\" is not valid", self.to_str()),
                        e
                    )
                }
            }
        }
        Ok(&self.data)
    }
}

/*
impl FileGroup {
    pub fn new(location: &Path, data_type: VerifierData) -> Self {
        FileGroup {
            location: location.to_path_buf(),
            data_type,
            files: None,
        }
    }

    fn get_paths(&self) -> Vec<PathBuf> {
        todo!()
    }

    fn get_files(&self) -> Box<HashMap<usize, File>> {
        todo!()
    }
} */
/*
pub struct Directory {
    path: Path,
    files: Vec<File>,
    file_groups: Vec<FileGroup>,
    directories: Vec<Directory>,
}

impl FileGroup {
    pub fn get_files() -> Vec<File> {}
}
 */
