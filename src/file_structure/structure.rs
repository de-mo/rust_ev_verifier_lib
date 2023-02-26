use crate::data_structures::{VerifierData, VerifierDataTrait};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{FileStructureError, FileStructureErrorType, GetFileName};
use crate::error::{create_result_with_error, create_verifier_error, VerifierError};

pub struct File {
    path: PathBuf,
    data: VerifierData,
}

pub struct FileGroup {
    location: PathBuf,
    data_type: VerifierData,
    files: Option<Box<HashMap<usize, File>>>,
}

impl File {
    pub fn new(location: &Path, data: VerifierData) -> Self {
        File {
            path: location.join(data.get_file_name()),
            data,
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

    pub fn get_data(&mut self) -> Result<&VerifierData, FileStructureError> {
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
            match self.data.new_from_json(&s) {
                Ok(x) => self.data = x,
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
