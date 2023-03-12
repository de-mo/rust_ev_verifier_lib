use super::{file::File, GetFileName};
use crate::data_structures::VerifierDataType;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice::Iter;

#[derive(Clone)]
pub struct FileGroup {
    location: PathBuf,
    data_type: VerifierDataType,
    numbers: Vec<usize>,
}

pub struct FileGroupIter<'a> {
    file_group: &'a FileGroup,
    iter: Iter<'a, usize>,
}

impl Iterator for FileGroupIter<'_> {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(i) => Some(File::new(
                &self.file_group.location,
                self.file_group.data_type.clone(),
                Some(*i),
            )),
            None => None,
        }
    }
}

impl FileGroup {
    pub fn new(location: &Path, data_type: VerifierDataType) -> Self {
        let mut res = Self {
            location: location.to_path_buf(),
            data_type,
            numbers: vec![],
        };
        res.set_numbers();
        res
    }

    fn set_numbers(&mut self) {
        if self.exists() {
            for e in fs::read_dir(&self.location).unwrap() {
                let name = e.unwrap().file_name().to_str().unwrap().to_string();
                let matching = self.data_type.get_raw_file_name();
                let matching_splitted: Vec<&str> = matching.split("{}").collect();
                let tmp = name
                    .replace(matching_splitted[0], "")
                    .replace(matching_splitted[1], "")
                    .parse::<usize>();
                match tmp {
                    Ok(i) => self.numbers.push(i),
                    Err(_) => (),
                }
            }
            self.numbers.sort()
        }
    }

    pub fn get_location(&self) -> PathBuf {
        self.location.to_path_buf()
    }

    pub fn exists(&self) -> bool {
        self.location.is_dir()
    }

    pub fn get_paths(&self) -> Vec<PathBuf> {
        self.iter().map(|f| f.get_path()).collect()
    }

    pub fn get_numbers(&self) -> Vec<usize> {
        self.numbers.clone()
    }

    pub fn iter(&self) -> FileGroupIter {
        FileGroupIter {
            file_group: self,
            iter: self.numbers.iter(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data_structures::setup::VerifierSetupDataType;
    use std::path::{Path, PathBuf};

    fn get_location() -> PathBuf {
        Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
    }

    #[test]
    fn test_file_group() {
        let location = get_location();
        let fg = FileGroup::new(
            &location,
            VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload),
        );
        assert!(fg.exists());
        assert_eq!(fg.get_location(), location);
        assert_eq!(fg.get_numbers(), [1, 2, 3, 4]);
        for (i, f) in fg.iter().enumerate() {
            let name = format!("controlComponentPublicKeysPayload.{}.json", i + 1);
            assert_eq!(f.get_path(), location.join(name));
        }
    }

    #[test]
    fn test_file_group_not_exist() {
        let location = get_location().join("toto");
        let fg = FileGroup::new(
            &location,
            VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload),
        );
        assert!(!fg.exists());
        assert_eq!(fg.get_location(), location);
        assert!(fg.get_numbers().is_empty());
        assert!(fg.iter().next().is_none());
    }
}
