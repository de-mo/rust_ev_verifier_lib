//! Trait implementing group of files
use super::{file::File, GetFileNameTrait};
use crate::data_structures::VerifierDataType;
use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

/// Trait for the possibility to mock the iteration over filegroup
pub trait FileGroupIterTrait<T>: Iterator<Item = (usize, T)> {
    fn current_elt(&self) -> Option<T>;
    fn current_pos(&self) -> &usize;
    fn current_index(&self) -> Option<&usize>;
    fn is_over(&self) -> bool {
        self.current_index().is_none()
    }
}

/// File Group
#[derive(Clone)]
pub struct FileGroup {
    /// location of the file group
    location: PathBuf,
    /// data_type. With the data_type it is possible to find the files in the location
    data_type: VerifierDataType,
    /// The numbers for which the files are defined
    indexes: Vec<usize>,
}

pub struct FileGroupIter<T> {
    pub file_group: FileGroup,
    pos: usize,
    not_used: PhantomData<T>,
}

/// Implement iterator for all the [FileGroupIter] as generic type
impl<T> Iterator for FileGroupIter<T>
where
    Self: FileGroupIterTrait<T>,
{
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.is_over() {
            true => None,
            false => {
                let res = (
                    self.current_index().unwrap().clone(),
                    self.current_elt().unwrap(),
                );
                self.pos += 1;
                Some(res)
            }
        }
    }
}

impl FileGroupIterTrait<File> for FileGroupIter<File> {
    fn current_elt(&self) -> Option<File> {
        self.current_file()
    }

    fn current_pos(&self) -> &usize {
        self.current_pos_impl()
    }

    fn current_index(&self) -> Option<&usize> {
        self.current_index_impl()
    }
}

impl<T> FileGroupIter<T> {
    /// Create a new [FileGroupIter<T>]
    pub fn new(file_group: &FileGroup) -> Self {
        FileGroupIter {
            file_group: file_group.clone(),
            pos: 0,
            not_used: PhantomData,
        }
    }

    /// Get the current position
    pub fn current_pos_impl(&self) -> &usize {
        &self.pos
    }

    /// Get the current index
    pub fn current_index_impl(&self) -> Option<&usize> {
        match &self.pos < &self.file_group.get_numbers().len() {
            true => Some(&self.file_group.get_numbers()[self.pos]),
            false => None,
        }
    }

    /// Get the current file
    pub fn current_file(&self) -> Option<File> {
        match self.current_index_impl() {
            Some(i) => Some(File::new(
                &self.file_group.location,
                &self.file_group.data_type,
                Some(*i),
            )),
            None => None,
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
///     ControlComponentPublicKeysPayloadAsResult,
///     ControlComponentPublicKeysPayloadAsResultIter
/// );
/// ```
macro_rules! impl_iterator_over_data_payload {
    ($p: ty, $fct: ident, $pread: ident, $preaditer: ident) => {
        type $pread = Result<Box<$p>, FileStructureError>;
        type $preaditer = FileGroupIter<$pread>;
        impl FileGroupIterTrait<$pread> for $preaditer {
            fn current_elt(&self) -> Option<$pread> {
                match self.current_file() {
                    Some(f) => Some(f.get_data().map(|d| Box::new(d.$fct().unwrap().clone()))),
                    None => None,
                }
            }
            fn current_pos(&self) -> &usize {
                self.current_pos_impl()
            }

            fn current_index(&self) -> Option<&usize> {
                self.current_index_impl()
            }
        }
    };
}
pub(super) use impl_iterator_over_data_payload;

/// Macro implementing adding a type to a directory to
/// iterate over a filegroup resulting a specific element type
macro_rules! add_type_for_file_group_iter_trait {
    ($t: ident, $res: ident) => {
        type $t: FileGroupIterTrait<$res>;
    };
}
pub(super) use add_type_for_file_group_iter_trait;

impl FileGroup {
    /// New [FileGroup]
    pub fn new(location: &Path, data_type: VerifierDataType) -> Self {
        let mut res = Self {
            location: location.to_path_buf(),
            data_type,
            indexes: vec![],
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
                    Ok(i) => self.indexes.push(i),
                    Err(_) => (),
                }
            }
            self.indexes.sort()
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
        !self.indexes.is_empty()
    }

    /// Get the paths of the files
    pub fn get_paths(&self) -> Vec<PathBuf> {
        self.iter().map(|(_, f)| f.get_path()).collect()
    }

    /// Get all the valid numbers of the files
    pub fn get_numbers(&self) -> &Vec<usize> {
        &self.indexes
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

#[cfg(any(test, doc))]
pub mod mock {
    //! Module defining mocking structure [FileGroupTrait]
    use super::{
        super::mock::{mock_payload, wrap_file_group_getter, wrap_payload_getter},
        *,
    };
    use crate::error::{create_result_with_error, create_verifier_error, VerifierError};
    use std::collections::HashMap;

    pub struct MockFileGroupIter<'a, 'b, T> {
        orig: &'a Box<dyn FileGroupIterTrait<T>>,
        mocked_data: &'b HashMap<usize, T>,
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
    macro_rules! impl_iterator_over_data_payload_mock {
        ($p: ty, $fct: ident, $pread: ident, $preaditer: ident) => {
            type $pread = Result<Box<$p>, FileStructureError>;
            type $preaditer = MockFileGroupIter<$pread>;
            impl FileGroupIterTrait for $preaditer {
                type IterType = $pread;
                fn current_elt(&self) -> Option<Self::IterType> {
                    match self.current_index() {
                        Some(i) => match self.mock_data().get(i) {
                            Some(elt) => Some(elt),
                            None => Some(self.orig().current_elt()),
                        },
                        None => None,
                    }
                }
            }
        };
    }
    pub(super) use impl_iterator_over_data_payload_mock;

    impl<'a, 'b, T> MockFileGroupIter<'a, 'b, T> {
        pub fn new(
            fg_iter: &'a Box<dyn FileGroupIterTrait<T>>,
            mock_data: &'b HashMap<usize, T>,
        ) -> Self {
            MockFileGroupIter {
                orig: fg_iter,
                mocked_data: mock_data,
            }
        }

        pub fn orig(&self) -> &Box<dyn FileGroupIterTrait<T>> {
            &self.orig
        }
        pub fn mocked_data(&self) -> &HashMap<usize, T> {
            &self.mocked_data
        }
        pub fn current_pos(&self) -> &usize {
            self.orig.current_pos()
        }

        pub fn current_index(&self) -> Option<&usize> {
            self.orig.current_index()
        }

        pub fn is_over(&self) -> bool {
            self.orig.is_over()
        }
    }

    // Implement iterator for all the [FileGroupIter] as generic type
    impl<'a, 'b, T> Iterator for MockFileGroupIter<'a, 'b, T>
    where
        Self: FileGroupIterTrait<T>,
    {
        type Item = (usize, T);

        fn next(&mut self) -> Option<Self::Item> {
            match self.current_index() {
                Some(i) => {
                    let res = (*i, self.current_elt().unwrap());
                    self.next();
                    Some(res)
                }
                None => None,
            }
        }
    }
}
