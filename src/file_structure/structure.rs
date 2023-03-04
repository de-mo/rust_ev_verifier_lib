use crate::data_structures::{VerifierData, VerifierDataTrait};
use std::borrow::{Borrow, BorrowMut};
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
    files: Option<Box<HashMap<usize, Box<File>>>>,
}

pub struct Directory {
    path: PathBuf,
    files: Vec<Box<File>>,
    file_groups: Vec<Box<FileGroup>>,
    directories: Vec<Box<Directory>>,
}

impl File {
    pub fn new(location: &Path, data: &VerifierData, file_nb: Option<usize>) -> Self {
        File {
            path: location.join(data.get_file_name(file_nb)),
            data: data.new_empty(),
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

impl FileGroup {
    pub fn new(location: &Path, data_type: VerifierData) -> Self {
        Self {
            location: location.to_path_buf(),
            data_type,
            files: None,
        }
    }

    pub fn exists(&self) -> bool {
        self.location.is_dir()
    }

    pub fn get_paths(&self) -> Vec<PathBuf> {
        todo!()
    }

    fn find_all_files(&self) -> Result<Vec<(usize, String)>, FileStructureError> {
        if !self.exists() {
            return create_result_with_error!(
                FileStructureErrorType::IsNotDir,
                format!(
                    "Location \"{}\" must be an existing directory",
                    self.location.to_str().unwrap()
                )
            );
        }
        let mut res = vec![];
        for e in fs::read_dir(&self.location).unwrap() {
            let name = e.unwrap().file_name().to_str().unwrap().to_string();
            let matching = self.data_type.get_raw_file_name();
            let matching_splitted: Vec<&str> = matching.split("{}").collect();
            let tmp = name
                .replace(matching_splitted[0], "")
                .replace(matching_splitted[1], "")
                .parse::<usize>();
            match tmp {
                Ok(i) => res.push((i, name)),
                Err(_) => (),
            }
        }
        Ok(res)
    }

    fn set_files(&mut self) {
        let list = self.find_all_files();
        if list.is_ok() {
            let mut hm: HashMap<usize, Box<File>> = HashMap::new();
            for (i, _) in list.unwrap() {
                hm.insert(
                    i,
                    Box::new(File::new(&self.location, &self.data_type, Some(i))),
                );
            }
            self.files = Some(Box::new(hm));
        }
    }

    pub fn get_files(&mut self) -> Result<&Box<HashMap<usize, Box<File>>>, FileStructureError> {
        if !self.exists() {
            return create_result_with_error!(
                FileStructureErrorType::IsNotDir,
                format!(
                    "Location \"{}\" must be an existing directory",
                    self.location.to_str().unwrap()
                )
            );
        }
        let files = &self.files;
        match files {
            Some(_) => (),
            None => {
                self.set_files();
            }
        };
        Ok(self.files.as_ref().unwrap())
    }
}

impl Directory {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            files: vec![],
            file_groups: vec![],
            directories: vec![],
        }
    }

    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn push_file(&mut self, data: &VerifierData, file_nb: Option<usize>) -> &Self {
        self.files
            .push(Box::new(File::new(&self.path, data, file_nb)));
        self
    }

    pub fn push_file_groups(&mut self, data_type: VerifierData) -> &Self {
        self.file_groups
            .push(Box::new(FileGroup::new(&self.path, data_type)));
        self
    }

    pub fn push_directory(&mut self, name: &String) -> &Self {
        self.directories
            .push(Box::new(Directory::new(&self.path.join(name))));
        self
    }

    pub fn push_directory_object(&mut self, dir: Directory) -> &Self {
        self.directories.push(Box::new(dir));
        self
    }

    pub fn exists(&self) -> bool {
        self.path.is_dir()
    }

    pub fn get_files(&self) -> &Vec<Box<File>> {
        &self.files
    }

    pub fn get_file_groups(&self) -> &Vec<Box<FileGroup>> {
        &self.file_groups
    }

    pub fn get_directory(&self) -> &Vec<Box<Directory>> {
        &self.directories
    }
}
