//! Trait implementing group of files
use super::{file::File, GetFileNameTrait};
use crate::data_structures::VerifierDataType;
use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

/// Trait for the possibility to mock the iteration over filegroup
pub trait FileGroupIterTrait {
    type IterType;
    fn get_elt_at_index(&self, index: &usize) -> Option<Self::IterType>;
    fn is_index_valid(&self, index: &usize) -> bool;
}

/// File Group
#[derive(Clone)]
pub struct FileGroup {
    /// location of the file group
    location: PathBuf,
    /// data_type. With the data_type it is possible to find the files in the location
    data_type: VerifierDataType,
    /// The numbers for which the files are defined
    numbers: Vec<usize>,
}

pub struct FileGroupIter<T> {
    pub file_group: FileGroup,
    index: usize,
    not_used: PhantomData<T>,
}

/// Implement iterator for all the [FileGroupIter] as generic type
impl<T> Iterator for FileGroupIter<T>
where
    Self: FileGroupIterTrait<IterType = T>,
{
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.get_elt_at_index(&self.index) {
            Some(elt) => {
                let res = (self.index.clone(), elt);
                self.index += 1;
                Some(res)
            }
            None => None,
        }
    }
}

impl FileGroupIterTrait for FileGroupIter<File> {
    type IterType = File;
    fn get_elt_at_index(&self, index: &usize) -> Option<Self::IterType> {
        match self.is_index_valid(index) {
            true => Some(File::new(
                &self.file_group.location,
                &self.file_group.data_type,
                Some(*index),
            )),
            false => None,
        }
    }

    fn is_index_valid(&self, index: &usize) -> bool {
        self.file_group.numbers.contains(index)
    }
}

impl<T> FileGroupIter<T> {
    pub fn new(file_group: &FileGroup) -> Self {
        FileGroupIter {
            file_group: file_group.clone(),
            index: 0,
            not_used: PhantomData,
        }
    }
}

/// Macro implementing an iterator over the data covered by the
/// FileGroup
///
/// parameters:
/// - $p: The struct as result
/// - $fct: The function to get the data (defined in the trait associated to the Dir object)
/// - $pread: The result of reading data $p from the file
/// - $preaditer: The iterator name over $pread
///
/// Example:
/// ```rust
/// impl_iterator_over_data_payload!(
///     ControlComponentPublicKeysPayload,
///     control_component_public_keys_payload,
///     ControlComponentPublicKeysPayloadRead,
///     ControlComponentPublicKeysPayloadReadIter
/// );
/// ```
macro_rules! impl_iterator_over_data_payload {
    ($p: ty, $fct: ident, $pread: ident, $preaditer: ident) => {
        type $pread = Result<Box<$p>, FileStructureError>;
        type $preaditer = FileGroupIter<$pread>;
        impl FileGroupIterTrait for $preaditer {
            type IterType = $pread;
            fn get_elt_at_index(&self, index: &usize) -> Option<Self::IterType> {
                match self.is_index_valid(index) {
                    true => Some(
                        File::new(
                            &self.file_group.get_location(),
                            self.file_group.get_data_type(),
                            Some(*index),
                        )
                        .get_data()
                        .map(|d| Box::new(d.$fct().unwrap().clone())),
                    ),
                    false => None,
                }
            }

            fn is_index_valid(&self, index: &usize) -> bool {
                self.file_group.get_numbers().contains(index)
            }
        }
    };
}
pub(crate) use impl_iterator_over_data_payload;

impl FileGroup {
    /// New [FileGroup]
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

    /// Get the location
    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }

    /// Get the data type
    pub fn get_data_type(&self) -> &VerifierDataType {
        &self.data_type
    }

    /// Test if the location exist
    pub fn location_exists(&self) -> bool {
        self.location.is_dir()
    }

    /// Test if the file group has elements, i.e. it exists files
    pub fn has_elements(&self) -> bool {
        !self.numbers.is_empty()
    }

    /// Get the paths of the files
    pub fn get_paths(&self) -> Vec<PathBuf> {
        self.iter().map(|(_, f)| f.get_path()).collect()
    }

    /// Get all the valid numbers of the files
    pub fn get_numbers(&self) -> &Vec<usize> {
        &self.numbers
    }

    /// Get the file with the given number
    pub fn get_file_with_number(&self, number: usize) -> File {
        File::new(&self.location, &self.data_type, Some(number))
    }

    /// Iterate over the files
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
        assert_eq!(fg.get_numbers(), &[1, 2, 3, 4]);
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
