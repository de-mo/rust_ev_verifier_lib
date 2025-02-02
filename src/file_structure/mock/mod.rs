//! Module defining mocking structure for [VerificationDirectory]
//!
//! Example of usage:
//! ```ignore
//!     let mut mock_dir = MockVerificationDirectory::new(&VerificationPeriod::Setup, &location);
//!     // Change the data
//!     mock_dir
//!         .context_mut()
//!         .mock_control_component_public_keys_payload(2, |d| {
//!             d.encryption_group.set_p(&Integer::from(1234usize));
//!             d.encryption_group.set_q(&Integer::from(1234usize))
//!     });
//!     // Test the verification that should generate failures
//!     fn_verification(&mock_dir, &mut result);
//! ```

mod context_directory_data;
mod setup_directory_data;
mod tally_directory_data;

use super::VerificationDirectoryTrait;
use super::{file_group::FileGroupFileIter, ContextDirectoryTrait, FileStructureError};
use crate::{
    data_structures::{VerifierDataDecode, VerifierDataToTypeTrait},
    verification::VerificationPeriod,
};
pub(crate) use context_directory_data::MockContextDirectory;
pub(crate) use setup_directory_data::MockSetupDirectory;
use std::{collections::HashMap, path::Path, sync::Arc};
pub(crate) use tally_directory_data::MockTallyDirectory;

/// Mock for [VerificationDirectory]
pub(crate) struct MockVerificationDirectory {
    context: MockContextDirectory,
    setup: Option<MockSetupDirectory>,
    tally: Option<MockTallyDirectory>,
}

impl VerificationDirectoryTrait for MockVerificationDirectory {
    type ContextDirType = MockContextDirectory;
    type SetupDirType = MockSetupDirectory;
    type TallyDirType = MockTallyDirectory;

    fn unwrap_setup(&self) -> &MockSetupDirectory {
        match &self.setup {
            Some(t) => t,
            None => panic!("called `unwrap_setup()` on a `Tally` value"),
        }
    }

    fn unwrap_tally(&self) -> &MockTallyDirectory {
        match &self.tally {
            Some(t) => t,
            None => panic!("called `unwrap_tally()` on a `Setup` value"),
        }
    }

    fn context(&self) -> &Self::ContextDirType {
        &self.context
    }

    fn path(&self) -> &Path {
        self.context().dir.location().parent().unwrap()
    }
}

impl MockVerificationDirectory {
    /// Create a new [MockVerificationDirectory]
    pub fn new(period: &VerificationPeriod, location: &Path) -> Self {
        let context = MockContextDirectory::new(location);
        match period {
            VerificationPeriod::Setup => MockVerificationDirectory {
                context,
                setup: Some(MockSetupDirectory::new(location)),
                tally: None,
            },
            VerificationPeriod::Tally => MockVerificationDirectory {
                context,
                setup: None,
                tally: Some(MockTallyDirectory::new(location)),
            },
        }
    }

    /// Context mut
    pub fn context_mut(&mut self) -> &mut MockContextDirectory {
        &mut self.context
    }

    /// Unwrap [MockSetupDirectory] as mutable
    #[allow(dead_code)]
    pub fn unwrap_setup_mut(&mut self) -> &mut MockSetupDirectory {
        match &mut self.setup {
            Some(t) => t,
            None => panic!("called `unwrap_tally()` on a `Setup` value"),
        }
    }

    /// Unwrap [TallyDirectory] as mutable
    #[allow(dead_code)]
    pub fn unwrap_tally_mut(&mut self) -> &mut MockTallyDirectory {
        match &mut self.tally {
            Some(t) => t,
            None => panic!("called `unwrap_tally()` on a `Setup` value"),
        }
    }
}

/// Macro to add the mock methods to mock the data
///
/// The following methods will be generated (example with `setup_component_public_keys_payload`
/// and `SetupComponentPublicKeysPayload`):
/// ```ignore
/// pub fn mock_setup_component_public_keys_payload(
///     &mut self,
///     mut closure: impl FnMut(&mut SetupComponentPublicKeysPayload),
/// ) {todo!()}
/// pub fn mock_setup_component_public_keys_payload_error(&mut self, error: FileStructureError) {
///     todo!()
/// }
/// pub fn mock_setup_component_public_keys_payload_remove_error(&mut self) {
///     todo!()
/// }
/// ```
///
/// Parameters:
/// - $data_name: The name of the data
/// - $data_type: The type of the data
macro_rules! impl_mock_methods_for_mocked_data {
    ($data_name: ident, $data_type: ident) => {
        paste! {
            #[allow(dead_code)]
            #[doc = "Mock `$data_name`"]
            pub fn [<mock_ $data_name>](
                &mut self,
                mut closure: impl FnMut(&mut $data_type),
            ) {
                let orig_payload = match self.dir.[<$data_name>]() {
                    Ok(p) => p.as_ref().clone(),
                    Err(_) => return
                };
                let mut payload = match self.[<mocked_ $data_name>].as_ref() {
                    Some(p) => match p.as_ref() {
                        MockedDataType::Data(p) => Some(p.clone()),
                        _ => None
                    }
                    None => None
                }.unwrap_or_else(|| orig_payload);
                closure(
                    &mut payload
                );
                self.[<mocked_ $data_name>] = Some(Box::new(MockedDataType::Data(payload.clone())));
            }
            #[doc = "Mock `$data_name` with error"]
            #[allow(dead_code)]
            pub fn [<mock_ $data_name _error>](
                &mut self,
                error: FileStructureError,
            ) {
                self.[<mocked_ $data_name>] = Some(Box::new(MockedDataType::Error(error.to_string())))
            }
            #[doc = "Reset the original data for `$data_name`"]
            #[allow(dead_code)]
            pub fn  [<mock_ $data_name _reset>](&mut self) {
                self.[<mocked_ $data_name>] = None;
            }
        }
    };
}
use impl_mock_methods_for_mocked_data;

/// Macro to add the trait method to the get the data in the directory traits.
///
/// The following methods will be generated (example with `setup_component_public_keys_payload`
/// and `SetupComponentPublicKeysPayload`):
/// ```ignore
/// pub fn setup_component_public_keys_payload(
///     &mut self,
/// ) Result<Arc<SetupComponentPublicKeysPayload>, FileStructureError>
/// {todo!()}
/// ```
///
/// Parameters:
/// - $data_name: The name of the data
/// - $data_type: The type of the data
macro_rules! impl_trait_get_method_for_mocked_data {
    ($data_name: ident, $data_type: ident) => {
        paste! {
            fn $data_name(
                &self,
            ) -> Result<Arc<$data_type>, FileStructureError> {
                match &self.[<mocked_ $data_name>] {
                    None => self.dir.$data_name(),
                    Some(e) => match e.as_ref() {
                        MockedDataType::Data(d) => Ok(Arc::new(d.clone())),
                        MockedDataType::Error(e) => Err(FileStructureError::Mock(e.to_string())),
                        MockedDataType::Deleted => Err(FileStructureError::Mock("Something wrong. Data cannot be deleted".to_string()))
                    }
                }
            }
        }
    };
}
use impl_trait_get_method_for_mocked_data;

/// Macro to add the mock methods to mock the data group
///
/// The following methods will be generated (example with `setup_component_public_keys_payload`
/// and `SetupComponentPublicKeysPayload`):
/// ```ignore
/// pub fn mock_[<$data_type>](
///     &mut self,
///     pos: usize,
///     mut closure: impl FnMut(&mut ControlComponentPublicKeysPayload),
/// ) { todo!()}
/// pub fn mock_[<$data_type>]_as_deleted(&mut self, i: usize) {
///     todo!() }
/// pub fn mock_[<$data_type>]_remove_deleted(&mut self, i: usize) {
///     todo!()}
/// pub fn mock_[<$data_type>]_error(
///     &mut self,
///     i: usize,
///     error: FileStructureError,
/// ) {todo!()}
/// pub fn mock_[<$data_type>]_remove_error(&mut self, i: usize) {
///     todo!()}
/// pub fn mock_[<$data_type>]_reset(&mut self, i: usize) {
///     todo!()
/// }
/// ```
///
/// Parameters:
/// - $data_name: The name of the data
/// - $data_type: The type of the data
macro_rules! impl_mock_methods_for_mocked_group {
    ($data_name: ident, $data_type: ident) => {
        paste! {
            #[allow(dead_code)]
            pub fn [<mock_ $data_name>](
                &mut self,
                pos: usize,
                mut closure: impl FnMut(&mut $data_type),
            ) {
                let orig_payload = match self.dir.[<$data_name _iter>]().find(|(i, _)| i == &pos) {
                    Some((_, res)) => match res {
                        Ok(p) => p.as_ref().clone(),
                        Err(_) => return
                    },
                    None => return
                };
                let mut payload = match self.[<mocked_ $data_name>].get(&pos) {
                    Some(p) => match &p.element_type {
                        MockedDataType::Data(p) => Some(p.clone()),
                        _ => None
                    }
                    None => None
                }.unwrap_or_else(|| orig_payload);
                closure(
                    &mut payload
                );
                let _ = self.[<mocked_ $data_name>].insert(pos, Box::new(MockFileGroupElement::new(
                    MockedDataType::Data(payload.clone()))));
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _as_deleted>](&mut self, pos: usize) {
                let _ = self.[<mocked_ $data_name>].insert(pos, Box::new(MockFileGroupElement::new(
                    MockedDataType::Deleted)));
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _error>](
                &mut self,
                pos: usize,
                error: FileStructureError,
            ) {
                let _ = self.[<mocked_ $data_name>].insert(pos, Box::new(MockFileGroupElement::new(
                    MockedDataType::Error(error.to_string()))));
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _reset>](&mut self, pos: usize) {
                let _ = self.[<mocked_ $data_name>].remove(&pos);
            }
        }
    };
}
use impl_mock_methods_for_mocked_group;

/// Macro to add the trait method to the get the data in the directory traits.
///
/// The following methods will be generated (example with `setup_component_public_keys_payload`
/// and `SetupComponentPublicKeysPayload`):
/// ```ignore
/// fn control_component_public_keys_payload_iter(
///     &self,
/// ) -> Self::ControlComponentPublicKeysPayloadAsResultIterType {todo!()}
/// ```
///
/// Parameters:
/// - $data_name: The name of the data
/// - $data_type: The type of the data
macro_rules! impl_trait_get_method_for_mocked_group {
    ($data_name: ident, $data_type: ident) => {
        paste! {
            fn [<$data_name _iter>](
                &self,
            ) -> impl Iterator<
            Item = (
                usize,
                Result<Arc<$data_type>, FileStructureError>,
            ),
        > {
            MockFileGroupDataIter::new(FileGroupFileIter::new(
                &self.dir.[<$data_name _group>]()), &self.[<mocked_ $data_name>])
            }
        }
    };
}
use impl_trait_get_method_for_mocked_group;

/// Mocked data type
#[derive(Clone)]
pub enum MockedDataType<D>
where
    D: VerifierDataDecode + VerifierDataToTypeTrait,
{
    Data(D),
    Deleted,
    Error(String),
}

/// File group element
#[derive(Clone)]
pub struct MockFileGroupElement<D>
where
    D: VerifierDataDecode + VerifierDataToTypeTrait + Clone,
{
    element_type: MockedDataType<D>,
}

impl<D> MockFileGroupElement<D>
where
    D: VerifierDataDecode + VerifierDataToTypeTrait + Clone,
{
    /// Transform to result
    ///
    /// Return `None` if the element is mocked as deleted
    fn to_data_res(&self) -> Option<Result<D, FileStructureError>> {
        match &self.element_type {
            MockedDataType::Data(d) => Some(Ok(d.clone())),
            MockedDataType::Deleted => None,
            MockedDataType::Error(e) => Some(Err(FileStructureError::Mock(e.to_string()))),
        }
    }
}

impl<D> MockFileGroupElement<D>
where
    D: VerifierDataDecode + VerifierDataToTypeTrait + Clone,
{
    /// New [MockFileGroupIter]
    ///
    /// fg_iter is the original iterator and mock_data contains the mocked data
    ///
    /// During the iteration, the data of the mocked data will be return if the index exists in the hashmap,
    /// else the original data will be returned
    pub fn new(element_type: MockedDataType<D>) -> Self {
        MockFileGroupElement { element_type }
    }
}

/// Iterator for the mock data in a file group
pub struct MockFileGroupDataIter<'a, D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> {
    pub file_group_iter: FileGroupFileIter<D>,
    pub mocked: &'a HashMap<usize, Box<MockFileGroupElement<D>>>,
}

impl<'a, D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> MockFileGroupDataIter<'a, D> {
    pub fn new(
        file_group_iter: FileGroupFileIter<D>,
        mocked: &'a HashMap<usize, Box<MockFileGroupElement<D>>>,
    ) -> Self {
        Self {
            file_group_iter,
            mocked,
        }
    }
}

/// Implement iterator for all the [MockFileGroupDataIter]
impl<D: VerifierDataDecode + VerifierDataToTypeTrait + Clone> Iterator
    for MockFileGroupDataIter<'_, D>
{
    type Item = (usize, Result<Arc<D>, FileStructureError>);

    fn next(&mut self) -> Option<Self::Item> {
        let (pos, file_res) = self.file_group_iter.next()?;
        match self.mocked.get(&pos) {
            Some(m) => match m.to_data_res() {
                Some(res) => Some((pos, res.map(Arc::new))),
                None => self.next(),
            },
            None => Some((pos, file_res.decode_verifier_data())),
        }
    }
}
