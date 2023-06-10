//! Trait implementing group of files
use super::{file::File, GetFileNameTrait};
use crate::data_structures::VerifierDataType;
use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

/// Trait for the possibility to mock the iteration over filegroup
pub(crate) trait FileGroupIterTrait<T>: Iterator<Item = (usize, T)> {
    fn current_elt(&self) -> Option<T>;
    fn current_pos(&self) -> &usize;
    fn current_index(&self) -> Option<&usize>;
    fn is_over(&self) -> bool {
        self.current_index().is_none()
    }
}

/// File Group
#[derive(Clone)]
pub(crate) struct FileGroup {
    /// location of the file group
    location: PathBuf,
    /// data_type. With the data_type it is possible to find the files in the location
    data_type: VerifierDataType,
    /// The numbers for which the files are defined
    indexes: Vec<usize>,
}

#[derive(Clone)]
pub(crate) struct FileGroupIter<T> {
    pub(crate) file_group: FileGroup,
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
    pub(crate) fn new(file_group: &FileGroup) -> Self {
        FileGroupIter {
            file_group: file_group.clone(),
            pos: 0,
            not_used: PhantomData,
        }
    }

    /// Get the current position
    pub(crate) fn current_pos_impl(&self) -> &usize {
        &self.pos
    }

    /// Get the current index
    pub(crate) fn current_index_impl(&self) -> Option<&usize> {
        match &self.pos < &self.file_group.get_numbers().len() {
            true => Some(&self.file_group.get_numbers()[self.pos]),
            false => None,
        }
    }

    /// Get the current file
    pub(crate) fn current_file(&self) -> Option<File> {
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
/// ```ignore
/// impl_iterator_over_data_payload!(
///     ControlComponentPublicKeysPayload,
///     control_component_public_keys_payload,
///     ControlComponentPublicKeysPayloadAsResult,
///     ControlComponentPublicKeysPayloadAsResultIter
/// );
/// ```
macro_rules! impl_iterator_over_data_payload {
    ($p: ty, $fct: ident, $pread: ident, $preaditer: ident) => {
        type $pread = anyhow::Result<Box<$p>>;
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
    pub(crate) fn new(location: &Path, data_type: VerifierDataType) -> Self {
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
    #[allow(dead_code)]
    pub(crate) fn get_location(&self) -> &Path {
        self.location.as_path()
    }

    /// Get the name of the file group
    pub(crate) fn get_file_name(&self) -> String {
        self.data_type.get_raw_file_name()
    }

    /// Get the data type
    #[allow(dead_code)]
    pub(crate) fn get_data_type(&self) -> &VerifierDataType {
        &self.data_type
    }

    /// Test if the location exist
    pub(crate) fn location_exists(&self) -> bool {
        self.location.is_dir()
    }

    /// Test if the file group has elements, i.e. it exists files
    pub(crate) fn has_elements(&self) -> bool {
        !self.indexes.is_empty()
    }

    /// Get the paths of the files
    #[allow(dead_code)]
    pub(crate) fn get_paths(&self) -> Vec<PathBuf> {
        self.iter().map(|(_, f)| f.get_path()).collect()
    }

    /// Get all the valid numbers of the files
    pub(crate) fn get_numbers(&self) -> &Vec<usize> {
        &self.indexes
    }

    /// Get the file with the given number
    pub(crate) fn get_file_with_number(&self, number: usize) -> File {
        File::new(&self.location, &self.data_type, Some(number))
    }

    /// Iterate over the files
    pub(crate) fn iter(&self) -> FileGroupIter<File> {
        FileGroupIter::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::test::dataset_setup_path;
    use crate::data_structures::setup::VerifierSetupDataType;
    use std::path::PathBuf;

    fn get_location() -> PathBuf {
        dataset_setup_path().join("setup")
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
#[allow(dead_code)]
pub(crate) mod mock {
    //! Module defining mocking structure [FileGroupTrait]
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone)]
    pub(crate) struct MockFileGroupIter<T, I: FileGroupIterTrait<T>> {
        orig: I,
        mocked_data: HashMap<usize, T>,
    }
    /// Macro implementing an iterator over the data covered by the
    /// FileGroup
    ///
    /// parameters:
    /// - $p: The struct
    /// - $pread: The result of reading data $p from the file
    /// - $preaditer: The iterator name over $pread
    /// - $mockpreaditer: The mocked iterator name over $pread
    ///
    /// Example:
    /// ```ignore
    /// impl_iterator_over_data_payload_mock!(
    ///     ControlComponentPublicKeysPayload,
    ///     ControlComponentPublicKeysPayloadAsResult,
    ///     ControlComponentPublicKeysPayloadAsResultIter,
    ///     MockControlComponentPublicKeysPayloadAsResultIter
    /// );
    /// ```
    macro_rules! impl_iterator_over_data_payload_mock {
        ($p: ty, $pread: ident, $preaditer: ident,$mockpreaditer: ident) => {
            type $pread = anyhow::Result<Box<$p>>;
            type $mockpreaditer = MockFileGroupIter<$pread, $preaditer>;
            impl FileGroupIterTrait<$pread> for $mockpreaditer {
                fn current_elt(&self) -> Option<$pread> {
                    match self.current_index() {
                        Some(i) => match self.mocked_data().get(i) {
                            Some(data) => match data {
                                Ok(d) => Some(Ok(d.clone().to_owned())),
                                Err(e) => Some(Err(anyhow!(format!("{}", e)))),
                            },
                            None => match self.orig().current_elt().unwrap() {
                                Ok(d) => Some(Ok((d.clone().to_owned()))),
                                Err(e) => Some(Err(anyhow!(e))),
                            },
                        },
                        None => None,
                    }
                }

                fn current_pos(&self) -> &usize {
                    self.orig().current_pos()
                }

                fn current_index(&self) -> Option<&usize> {
                    self.orig().current_index()
                }
            }
        };
    }
    pub(in super::super) use impl_iterator_over_data_payload_mock;

    impl<T, I: FileGroupIterTrait<T>> MockFileGroupIter<T, I> {
        /// New [MockFileGroupIter]
        ///
        /// fg_iter is the original iterator and mock_data contains the mocked data
        ///
        /// During the iteration, the data of the mocked data will be return if the index exists in the hashmap,
        /// else the original data will be returned
        pub(crate) fn new(fg_iter: I, mock_data: HashMap<usize, T>) -> Self {
            MockFileGroupIter {
                orig: fg_iter,
                mocked_data: mock_data,
            }
        }

        /// Get the original iterator
        pub(crate) fn orig(&self) -> &I {
            &self.orig
        }

        /// Get the original iterator as mutable
        pub(crate) fn orig_mut(&mut self) -> &mut I {
            &mut self.orig
        }

        /// Get the mocked data
        pub(crate) fn mocked_data(&self) -> &HashMap<usize, T> {
            &self.mocked_data
        }

        ///
        pub(crate) fn current_pos(&self) -> &usize {
            self.orig.current_pos()
        }

        pub(crate) fn current_index(&self) -> Option<&usize> {
            self.orig.current_index()
        }

        pub(crate) fn is_over(&self) -> bool {
            self.orig.is_over()
        }
    }

    // Implement iterator for all the [FileGroupIter] as generic type
    impl<'a, 'b, T, I: FileGroupIterTrait<T>> Iterator for MockFileGroupIter<T, I>
    where
        Self: FileGroupIterTrait<T>,
    {
        type Item = (usize, T);

        fn next(&mut self) -> Option<Self::Item> {
            match self.current_index() {
                Some(i) => {
                    let res = (*i, self.current_elt().unwrap());
                    self.orig_mut().next();
                    Some(res)
                }
                None => None,
            }
        }
    }

    /// Macro to implement a function to the mocked structures wrapping the iterator
    /// If some of the values are mocked, then return the mocked value, else return the original.
    ///
    /// Parameters:
    /// - $fct: The name of the function
    /// - $itertype: The type of the iterator in the trait
    /// - $iter: The iter obkect to be created
    /// - $mock: The name of the hashmap containintg the mocked data
    ///
    /// Example:
    /// ```ignore
    /// wrap_payload_iter!(
    ///     control_component_public_keys_payload_iter,
    ///     ControlComponentPublicKeysPayloadAsResultIterType,
    ///     MockControlComponentPublicKeysPayloadAsResultIter,
    ///     mocked_control_component_public_keys_payloads
    /// );
    /// ```
    macro_rules! wrap_payload_iter {
        ($fct: ident, $itertype: ident, $iter: ident, $mock: ident) => {
            fn $fct(&self) -> Self::$itertype {
                let mut hash_map = HashMap::new();
                for (i, elt) in self.$mock.iter() {
                    hash_map.insert(
                        i.to_owned(),
                        match elt {
                            Ok(d) => Ok(d.clone().to_owned()),
                            Err(e) => Err(anyhow!(format!("{}", e))),
                        },
                    );
                }
                $iter::new(self.dir.$fct(), hash_map)
            }
        };
    }
    pub(in super::super) use wrap_payload_iter;

    /// Macro to implement a function to mock an iterator
    ///
    /// Parameters:
    /// - $fct: The name of the function
    /// - $mock: The name of the mocked structure field to update
    /// - $payload: Type of the payload
    macro_rules! mock_payload_iter {
        ($fct: ident, $mock: ident, $payload: ty) => {
            pub(crate) fn $fct(&mut self, index: usize, data: &anyhow::Result<&$payload>) {
                self.$mock.insert(
                    index,
                    match data {
                        Ok(d) => Ok(Box::new(d.clone().to_owned())),
                        Err(e) => Err(anyhow!(format!("{}", e))),
                    },
                );
            }
        };
    }
    pub(in super::super) use mock_payload_iter;
}
