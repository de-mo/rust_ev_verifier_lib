//! Trait implementing group of files with the same structure (in particular for the files from the control components)
use super::{file::File, FileStructureError, GetFileNameTrait};
use crate::data_structures::{VerifierDataDecode, VerifierDataToTypeTrait, VerifierDataType};
use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

/// File Group
#[derive(Clone)]
pub struct FileGroup<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> {
    /// location of the file group
    location: PathBuf,
    /// The numbers for which the files are defined
    indexes: Vec<usize>,
    phantom: PhantomData<D>,
}

/// Iterator for the files in a file group
pub struct FileGroupFileIter<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> {
    pub file_group: FileGroup<D>,
    pos: usize,
}

/// Iterator for the data in a file group
pub struct FileGroupDataIter<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> {
    pub file_group_iter: FileGroupFileIter<D>,
}

impl<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> From<FileGroupFileIter<D>>
    for FileGroupDataIter<D>
{
    fn from(value: FileGroupFileIter<D>) -> Self {
        Self {
            file_group_iter: value,
        }
    }
}

/// Implement iterator for the [FileGroupFileIter] as generic type
impl<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> Iterator for FileGroupFileIter<D> {
    type Item = (usize, File<D>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.is_over() {
            true => None,
            false => {
                let res = (*self.current_index().unwrap(), self.current_file().unwrap());
                self.pos += 1;
                Some(res)
            }
        }
    }
}

/// Implement iterator for the [FileGroupDataIter] as generic type
impl<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> Iterator for FileGroupDataIter<D>
//where
//    Self: GenericFileGroupIterTrait<Result<D, FileStructureError>>,
{
    type Item = (usize, Result<D, FileStructureError>);

    fn next(&mut self) -> Option<Self::Item> {
        self.file_group_iter
            .next()
            .map(|(i, f)| (i, f.decode_verifier_data()))
    }
}

impl<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> FileGroupFileIter<D> {
    /// Create a new [FileGroupIter<T>]
    pub fn new(file_group: &FileGroup<D>) -> Self {
        FileGroupFileIter {
            file_group: file_group.clone(),
            pos: 0,
        }
    }

    /// Get the current  of the ireation
    pub fn current_pos(&self) -> &usize {
        &self.pos
    }

    /// Get the current index
    pub fn current_index(&self) -> Option<&usize> {
        match self.pos < self.file_group.get_numbers().len() {
            true => Some(&self.file_group.get_numbers()[self.pos]),
            false => None,
        }
    }

    /// Get the current file
    pub fn current_file(&self) -> Option<File<D>> {
        self.current_index()
            .map(|i| File::new(&self.file_group.location, Some(*i)))
    }

    /// Is iterator over
    fn is_over(&self) -> bool {
        self.current_index().is_none()
    }
}

impl<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> FileGroup<D> {
    /// New [FileGroup]
    pub fn new(location: &Path) -> Self {
        let mut res = Self {
            location: location.to_path_buf(),
            indexes: vec![],
            phantom: PhantomData,
        };
        res.set_numbers();
        res
    }

    fn set_numbers(&mut self) {
        if self.location_exists() {
            for e in fs::read_dir(&self.location).unwrap() {
                let name = e.unwrap().file_name().to_str().unwrap().to_string();
                let matching = Self::data_type().get_raw_file_name();
                let matching_splitted: Vec<&str> = matching.split("{}").collect();
                let tmp = name
                    .replace(matching_splitted[0], "")
                    .replace(matching_splitted[1], "")
                    .parse::<usize>();
                if let Ok(i) = tmp {
                    self.indexes.push(i)
                }
            }
            self.indexes.sort()
        }
    }

    /// get data type bbased on the generic
    pub fn data_type() -> VerifierDataType {
        D::data_type()
    }

    /// Get the location
    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }

    /// Get the name of the file group
    pub fn get_file_name(&self) -> String {
        Self::data_type().get_raw_file_name()
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
        self.iter_file().map(|(_, f)| f.path()).collect()
    }

    /// Get all the valid numbers of the files
    pub fn get_numbers(&self) -> &Vec<usize> {
        &self.indexes
    }

    /// Get the file with the given number
    pub fn get_file_with_number(&self, number: usize) -> File<D> {
        File::new(&self.location, Some(number))
    }

    /// Iterate over the files
    pub fn iter_file(&self) -> FileGroupFileIter<D> {
        FileGroupFileIter::new(self)
    }

    /// Iterate over the data
    pub fn iter(&self) -> FileGroupDataIter<D> {
        FileGroupDataIter::from(self.iter_file())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_datasets_context_path;
    use crate::data_structures::ControlComponentPublicKeysPayload;

    #[test]
    fn test_file_group() {
        let location = test_datasets_context_path();
        let fg = FileGroup::<ControlComponentPublicKeysPayload>::new(&location);
        assert!(fg.location_exists());
        assert!(fg.has_elements());
        assert_eq!(fg.get_location(), location);
        assert_eq!(fg.get_numbers(), &[1, 2, 3, 4]);
        for (i, f) in fg.iter_file() {
            let name = format!("controlComponentPublicKeysPayload.{}.json", i);
            assert_eq!(f.path(), location.join(name));
        }
    }

    #[test]
    fn test_get_file_with_number() {
        let location = test_datasets_context_path();
        let fg = FileGroup::<ControlComponentPublicKeysPayload>::new(&location);
        let f = fg.get_file_with_number(1);
        assert_eq!(
            f.path(),
            location.join("controlComponentPublicKeysPayload.1.json")
        );
    }

    #[test]
    fn test_file_group_not_exist() {
        let location = test_datasets_context_path().join("toto");
        let fg = FileGroup::<ControlComponentPublicKeysPayload>::new(&location);
        assert!(!fg.location_exists());
        assert!(!fg.has_elements());
        assert_eq!(fg.get_location(), location);
        assert!(fg.get_numbers().is_empty());
        assert!(fg.iter_file().next().is_none());
    }
}
