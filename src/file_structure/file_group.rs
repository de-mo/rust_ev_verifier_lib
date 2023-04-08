use super::{file::File, GetFileName};
use crate::data_structures::VerifierDataType;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::slice::Iter;

#[derive(Clone)]
pub struct FileGroup {
    location: PathBuf,
    data_type: VerifierDataType,
    numbers: Vec<usize>,
}

pub struct FileGroupIter<'a, T> {
    pub file_group: &'a FileGroup,
    pub iter: Iter<'a, usize>,
    not_used: PhantomData<T>,
}

impl Iterator for FileGroupIter<'_, File> {
    type Item = (usize, File);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(i) => Some((
                *i,
                File::new(
                    &self.file_group.location,
                    self.file_group.data_type.clone(),
                    Some(*i),
                ),
            )),
            None => None,
        }
    }
}

impl<'a, T> FileGroupIter<'a, T> {
    pub fn new(file_group: &'a FileGroup) -> Self {
        FileGroupIter {
            file_group,
            iter: file_group.numbers.iter(),
            not_used: PhantomData,
        }
    }
}

macro_rules! impl_iterator_payload {
    ($p: ty, $f: ident, $pread: ident, $preaditer: ident) => {
        type $pread = Result<Box<$p>, FileStructureError>;
        type $preaditer<'a> = FileGroupIter<'a, $pread>;
        impl Iterator for $preaditer<'_> {
            type Item = (usize, $pread);

            fn next(&mut self) -> Option<Self::Item> {
                match self.iter.next() {
                    Some(i) => Some((
                        *i,
                        File::new(
                            &self.file_group.get_location(),
                            self.file_group.get_data_type(),
                            Some(*i),
                        )
                        .get_data()
                        .map(|d| Box::new(d.$f().unwrap().clone())),
                    )),
                    None => None,
                }
            }
        }
    };
}
pub(crate) use impl_iterator_payload;

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
        if self.location_exists() {
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

    pub fn get_data_type(&self) -> VerifierDataType {
        self.data_type.clone()
    }

    pub fn location_exists(&self) -> bool {
        self.location.is_dir()
    }

    pub fn has_elements(&self) -> bool {
        !self.numbers.is_empty()
    }

    pub fn get_paths(&self) -> Vec<PathBuf> {
        self.iter().map(|(_, f)| f.get_path()).collect()
    }

    pub fn get_numbers(&self) -> Vec<usize> {
        self.numbers.clone()
    }

    pub fn get_file_with_number(&self, number: usize) -> File {
        File::new(&self.location, self.data_type.clone(), Some(number))
    }

    pub fn iter(&self) -> FileGroupIter<File> {
        FileGroupIter::new(self)
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
        assert!(fg.location_exists());
        assert!(fg.has_elements());
        assert_eq!(fg.get_location(), location);
        assert_eq!(fg.get_numbers(), [1, 2, 3, 4]);
        for (i, f) in fg.iter() {
            let name = format!("controlComponentPublicKeysPayload.{}.json", i);
            assert_eq!(f.get_path(), location.join(name));
        }
    }

    #[test]
    fn test_get_file_with_number() {
        let location = get_location();
        let fg = FileGroup::new(
            &location,
            VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload),
        );
        let f = fg.get_file_with_number(1);
        assert_eq!(
            f.get_path(),
            location.join("controlComponentPublicKeysPayload.1.json")
        );
    }

    #[test]
    fn test_file_group_not_exist() {
        let location = get_location().join("toto");
        let fg = FileGroup::new(
            &location,
            VerifierDataType::Setup(VerifierSetupDataType::ControlComponentPublicKeysPayload),
        );
        assert!(!fg.location_exists());
        assert!(!fg.has_elements());
        assert_eq!(fg.get_location(), location);
        assert!(fg.get_numbers().is_empty());
        assert!(fg.iter().next().is_none());
    }
}
