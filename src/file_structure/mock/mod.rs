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

pub(crate) use context_directory_data::MockContextDirectory;
pub(crate) use setup_directory_data::MockSetupDirectory;
use std::{collections::HashMap, path::Path};
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
                if self.[<mocked_ $data_name>].is_none() {
                    self.[<mocked_ $data_name>] =
                        Some(self.dir.$data_name().unwrap());
                }
                closure(
                    self.[<mocked_ $data_name>]
                        .as_mut()
                        .unwrap(),
                );
            }
            #[doc = "Mock `$data_name` with error"]
            #[allow(dead_code)]
            pub fn [<mock_ $data_name _error>](
                &mut self,
                error: FileStructureError,
            ) {
                self.[<mocked_ $data_name _error>] = Some(error)
            }
            #[doc = "Remove the error for `$data_name`"]
            #[allow(dead_code)]
            pub fn  [<mock_ $data_name _remove_error>](&mut self) {
                self.[<mocked_ $data_name _error>] = None
            }
            #[doc = "Reset the original data for `$data_name`"]
            #[allow(dead_code)]
            pub fn  [<mock_ $data_name _reset>](&mut self) {
                self.[<mocked_ $data_name _error>] = None;
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
/// ) Result<Box<SetupComponentPublicKeysPayload>, FileStructureError>
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
            ) -> Result<Box<$data_type>, FileStructureError> {
                match &self.[<mocked_ $data_name _error>] {
                    Some(e) => Err(FileStructureError::Mock(e.to_string())),
                    None => match &self.[<mocked_ $data_name>] {
                        Some(v) => Ok(Box::new(v.as_ref().clone())),
                        None => self.dir.$data_name(),
                    },
                }
            }
        }
    };
}
use impl_trait_get_method_for_mocked_data;

/// Macro to implement the iterator over the mocked group of data given as parameter
///
/// Parameters:
/// - $data_type: The type of the data group
macro_rules! impl_itertor_for_mocked_group_type {
    ($data_type: ident) => {
        paste! {
            type [<Mock $data_type AsResultIter>] = MockFileGroupIter<
                $data_type,
                [<$data_type AsResultIter>]
            >;
            // Implement iterator for all the [FileGroupIter] as generic type
            impl Iterator for [<Mock $data_type AsResultIter>] {
                type Item = (
                    usize,
                    Result<Box<$data_type>, FileStructureError>,
                );
                fn next(&mut self) -> Option<Self::Item> {
                    match self.current_index() {
                        Some(i) => {
                            if self.is_current_element_deleted() {
                                self.orig_mut().next();
                                return self.next();
                            }
                            let res = (*i, self.current_elt().unwrap());
                            self.orig_mut().next();
                            Some(res)
                        }
                        None => None,
                    }
                }
            }
            impl FileGroupIterTrait<Result<Box<$data_type>, FileStructureError>>
                for [<Mock $data_type AsResultIter>]
            {
                fn current_elt(
                    &self,
                ) -> Option<Result<Box<$data_type>, FileStructureError>> {
                    self.current_elt_impl()
                }

                fn current_pos(&self) -> &usize {
                    self.orig().current_pos()
                }

                fn current_index(&self) -> Option<&usize> {
                    self.orig().current_index()
                }
            }
        }
    };
}
use impl_itertor_for_mocked_group_type;

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
                let cc_payload_pos = match self
                    .[<$data_name _iter>]()
                    .find(|(i, _)| i == &pos)
                {
                    Some((_, d)) => d.unwrap(),
                    None => return,
                };
                self.[<mocked_ $data_name>]
                    .entry(pos)
                    .or_insert(cc_payload_pos);
                closure(
                    self.[<mocked_ $data_name>]
                        .get_mut(&pos)
                        .unwrap(),
                );
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _as_deleted>](&mut self, i: usize) {
                self.[<mocked_ $data_name _deleted>]
                    .push(i);
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _remove_deleted>](&mut self, i: usize) {
                if let Some(pos) = self
                    .[<mocked_ $data_name _deleted>]
                    .iter()
                    .position(|d| d == &i)
                {
                    self.[<mocked_ $data_name _deleted>]
                        .remove(pos);
                }
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _error>](
                &mut self,
                i: usize,
                error: FileStructureError,
            ) {
                self.[<mocked_ $data_name _errors>]
                    .insert(i, error.to_string());
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _remove_error>](&mut self, i: usize) {
                self.[<mocked_ $data_name _errors>]
                    .remove(&i);
            }

            #[allow(dead_code)]
            pub fn [<mock_ $data_name _reset>](&mut self, _i: usize) {
                self.[<mocked_ $data_name _errors>] = HashMap::new();
                self.[<mocked_ $data_name _deleted>] = vec![];
                self.[<mocked_ $data_name>] = HashMap::new();
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
            ) -> Self::[<$data_type AsResultIterType>] {
                Self::[<$data_type AsResultIterType>]::new(
                    self.dir.[<$data_name _iter>](),
                    &self.[<mocked_ $data_name>],
                    &self.[<mocked_ $data_name _deleted>],
                    &self.[<mocked_ $data_name _errors>],
                )
            }
        }
    };
}
use impl_trait_get_method_for_mocked_group;

use crate::verification::VerificationPeriod;

use super::{
    file_group::FileGroupIterTrait, ContextDirectoryTrait, FileStructureError,
    VerificationDirectoryTrait,
};

#[derive(Debug)]

pub struct MockFileGroupIter<T: Clone, I: FileGroupIterTrait<Result<Box<T>, FileStructureError>>> {
    orig: I,
    mocked_data: HashMap<usize, Box<T>>,
    mocked_deleted: Vec<usize>,
    mocked_errors: HashMap<usize, String>,
}

impl<T: Clone, I: FileGroupIterTrait<Result<Box<T>, FileStructureError>>> MockFileGroupIter<T, I> {
    /// New [MockFileGroupIter]
    ///
    /// fg_iter is the original iterator and mock_data contains the mocked data
    ///
    /// During the iteration, the data of the mocked data will be return if the index exists in the hashmap,
    /// else the original data will be returned
    pub fn new(
        fg_iter: I,
        mocked_data: &HashMap<usize, Box<T>>,
        mocked_deleted: &[usize],
        mocked_errors: &HashMap<usize, String>,
    ) -> Self {
        MockFileGroupIter {
            orig: fg_iter,
            mocked_data: mocked_data.clone(),
            mocked_deleted: mocked_deleted.to_vec(),
            mocked_errors: mocked_errors.clone(),
        }
    }

    /// Get the original iterator
    pub fn orig(&self) -> &I {
        &self.orig
    }

    /// Get the original iterator as mutable
    pub fn orig_mut(&mut self) -> &mut I {
        &mut self.orig
    }

    #[allow(dead_code)]
    pub fn current_pos(&self) -> &usize {
        self.orig.current_pos()
    }

    pub fn current_index(&self) -> Option<&usize> {
        self.orig.current_index()
    }

    #[allow(dead_code)]
    pub fn is_over(&self) -> bool {
        self.orig.is_over()
    }

    pub fn is_current_element_deleted(&self) -> bool {
        match self.current_index() {
            Some(i) => match self.mocked_errors.get(i) {
                Some(_) => false,
                None => self.mocked_deleted.iter().any(|pos| pos == i),
            },
            None => false,
        }
    }

    pub fn current_elt_impl(&self) -> Option<Result<Box<T>, FileStructureError>> {
        match self.current_index() {
            Some(i) => match self.mocked_errors.get(i) {
                Some(e) => Some(Err(FileStructureError::Mock(e.to_owned()))),
                None => match self.mocked_data.get(i) {
                    Some(data) => Some(Ok(data.clone())),
                    None => self.orig.current_elt(),
                },
            },
            None => None,
        }
    }
}
