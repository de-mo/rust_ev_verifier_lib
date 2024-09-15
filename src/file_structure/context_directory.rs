//! Module to implement the context directory

use super::{
    file::{create_file, File},
    file_group::{
        add_type_for_file_group_iter_trait, impl_iterator_over_data_payload, FileGroup,
        FileGroupIter, FileGroupIterTrait,
    },
    CompletnessTestTrait, FileStructureError,
};
use crate::{
    config::Config,
    data_structures::{
        context::{
            control_component_public_keys_payload::ControlComponentPublicKeysPayload,
            election_event_configuration::ElectionEventConfiguration,
            election_event_context_payload::ElectionEventContextPayload,
            setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
            setup_component_tally_data_payload::SetupComponentTallyDataPayload,
            VerifierContextDataType,
        },
        create_verifier_context_data_type, VerifierContextDataTrait, VerifierDataType,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The context directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct ContextDirectory {
    location: PathBuf,
    setup_component_public_keys_payload_file: File,
    election_event_context_payload_file: File,
    election_event_configuration_file: File,
    control_component_public_keys_payload_group: FileGroup,
    vcs_directories: Vec<ContextVCSDirectory>,
}

/// The vcs directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct ContextVCSDirectory {
    location: PathBuf,
    setup_component_tally_data_payload_file: File,
}

/// Trait to set the necessary functions for the struct [ContextDirectory] that
/// are used during the verifications
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait ContextDirectoryTrait: CompletnessTestTrait {
    type VCSDirType: ContextVCSDirectoryTrait;
    add_type_for_file_group_iter_trait!(
        ControlComponentPublicKeysPayloadAsResultIterType,
        ControlComponentPublicKeysPayloadAsResult
    );

    fn setup_component_public_keys_payload_file(&self) -> &File;
    fn election_event_context_payload_file(&self) -> &File;
    fn election_event_configuration_file(&self) -> &File;
    fn control_component_public_keys_payload_group(&self) -> &FileGroup;
    fn vcs_directories(&self) -> &Vec<Self::VCSDirType>;
    fn setup_component_public_keys_payload(
        &self,
    ) -> Result<Box<SetupComponentPublicKeysPayload>, FileStructureError>;

    fn election_event_context_payload(
        &self,
    ) -> Result<Box<ElectionEventContextPayload>, FileStructureError>;
    fn election_event_configuration(
        &self,
    ) -> Result<Box<ElectionEventConfiguration>, FileStructureError>;

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> Self::ControlComponentPublicKeysPayloadAsResultIterType;

    /// Collect the names of the vcs directories
    fn vcs_directory_names(&self) -> Vec<String> {
        self.vcs_directories()
            .iter()
            .map(|d| d.get_name())
            .collect()
    }
}

/// Trait to set the necessary functions for the struct [VCSDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait ContextVCSDirectoryTrait: CompletnessTestTrait {
    fn setup_component_tally_data_payload_file(&self) -> &File;
    fn setup_component_tally_data_payload(
        &self,
    ) -> Result<Box<SetupComponentTallyDataPayload>, FileStructureError>;
    fn get_name(&self) -> String;
}

impl_iterator_over_data_payload!(
    ControlComponentPublicKeysPayload,
    control_component_public_keys_payload,
    ControlComponentPublicKeysPayloadAsResult,
    ControlComponentPublicKeysPayloadAsResultIter
);

impl ContextDirectory {
    /// New [ContextDirectory]
    #[allow(clippy::redundant_clone)]
    pub fn new(data_location: &Path) -> Self {
        let location = data_location.join(Config::context_dir_name());
        let mut res = Self {
            location: location.to_path_buf(),
            setup_component_public_keys_payload_file: create_file!(
                location,
                Context,
                VerifierContextDataType::SetupComponentPublicKeysPayload
            ),
            election_event_context_payload_file: create_file!(
                location,
                Context,
                VerifierContextDataType::ElectionEventContextPayload
            ),
            election_event_configuration_file: create_file!(
                location,
                Context,
                VerifierContextDataType::ElectionEventConfiguration
            ),
            control_component_public_keys_payload_group: FileGroup::new(
                &location,
                create_verifier_context_data_type!(Context, ControlComponentPublicKeysPayload),
            ),
            vcs_directories: vec![],
        };
        let vcs_path = location.join(Config::vcs_dir_name());
        if vcs_path.is_dir() {
            for re in fs::read_dir(&vcs_path).unwrap() {
                let e = re.unwrap().path();
                if e.is_dir() {
                    res.vcs_directories.push(ContextVCSDirectory::new(&e))
                }
            }
        }
        res
    }

    /// Get location
    #[allow(dead_code)]
    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

macro_rules! impl_completness_test_trait_for_context {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                if !self.location.is_dir() {
                    return Err(FileStructureError::PathIsNotDir(self.location.to_path_buf()))
                }
                let mut missings = vec![];
                if !self.election_event_context_payload_file().exists() {
                    missings.push("election_event_context_payload does not exist".to_string());
                }
                if !self.setup_component_public_keys_payload_file().exists() {
                    missings.push("setup_component_public_keys_payload_file does not exist".to_string());
                }
                if !self.election_event_configuration_file().exists() {
                    missings.push("setup_component_public_keys_payload_file does not exist".to_string());
                }
                if self
                    .control_component_public_keys_payload_group()
                    .get_numbers()
                    != &vec![1, 2, 3, 4]
                {
                    missings.push(format!(
                        "control_component_public_keys_payload_group missing. only these parts are present: {:?}",
                        self
                            .control_component_public_keys_payload_group()
                            .get_numbers()
                    ))
                }
                if self.vcs_directories().is_empty() {
                    missings.push("No vcs directory found".to_string());
                }
                for d in self.vcs_directories().iter() {
                    missings.extend(d.test_completness()?);
                }
                Ok(missings)
            }
        }
    }
}
impl_completness_test_trait_for_context!(ContextDirectory);

impl ContextDirectoryTrait for ContextDirectory {
    type VCSDirType = ContextVCSDirectory;
    type ControlComponentPublicKeysPayloadAsResultIterType =
        ControlComponentPublicKeysPayloadAsResultIter;

    fn setup_component_public_keys_payload_file(&self) -> &File {
        &self.setup_component_public_keys_payload_file
    }
    fn election_event_context_payload_file(&self) -> &File {
        &self.election_event_context_payload_file
    }
    fn election_event_configuration_file(&self) -> &File {
        &self.election_event_configuration_file
    }
    fn control_component_public_keys_payload_group(&self) -> &FileGroup {
        &self.control_component_public_keys_payload_group
    }
    fn vcs_directories(&self) -> &Vec<ContextVCSDirectory> {
        &self.vcs_directories
    }

    fn setup_component_public_keys_payload(
        &self,
    ) -> Result<Box<SetupComponentPublicKeysPayload>, FileStructureError> {
        self.setup_component_public_keys_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.setup_component_public_keys_payload().unwrap().clone()))
    }

    fn election_event_context_payload(
        &self,
    ) -> Result<Box<ElectionEventContextPayload>, FileStructureError> {
        self.election_event_context_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.election_event_context_payload().unwrap().clone()))
    }

    fn election_event_configuration(
        &self,
    ) -> Result<Box<ElectionEventConfiguration>, FileStructureError> {
        self.election_event_configuration_file
            .get_verifier_data()
            .map(|d| Box::new(d.election_event_configuration().unwrap().clone()))
    }

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> Self::ControlComponentPublicKeysPayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_public_keys_payload_group)
    }
}

macro_rules! impl_completness_test_trait_for_context_vcs {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                if !self.location.is_dir() {
                    return Err(FileStructureError::PathIsNotDir(
                        self.location.to_path_buf(),
                    ));
                }
                let mut missings = vec![];
                if !self.setup_component_tally_data_payload_file().exists() {
                    missings.push(format!(
                        "setup_component_tally_data_payload does not exist in {:?}",
                        self.location.file_name().unwrap()
                    ))
                }
                Ok(missings)
            }
        }
    };
}

impl_completness_test_trait_for_context_vcs!(ContextVCSDirectory);

impl ContextVCSDirectory {
    /// New [VCSDirectory]
    pub fn new(location: &Path) -> Self {
        Self {
            location: location.to_path_buf(),
            setup_component_tally_data_payload_file: create_file!(
                location,
                Context,
                VerifierContextDataType::SetupComponentTallyDataPayload
            ),
        }
    }

    /// Get location
    #[allow(dead_code)]
    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

impl ContextVCSDirectoryTrait for ContextVCSDirectory {
    fn setup_component_tally_data_payload_file(&self) -> &File {
        &self.setup_component_tally_data_payload_file
    }
    fn setup_component_tally_data_payload(
        &self,
    ) -> Result<Box<SetupComponentTallyDataPayload>, FileStructureError> {
        self.setup_component_tally_data_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.setup_component_tally_data_payload().unwrap().clone()))
    }

    fn get_name(&self) -> String {
        self.location
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        test_all_context_vcs_paths, test_context_verification_card_set_path,
        test_datasets_context_path, test_datasets_path,
    };

    #[test]
    fn test_context_dir() {
        let context_location = test_datasets_context_path();
        let dir = ContextDirectory::new(context_location.parent().unwrap());
        assert_eq!(dir.get_location(), context_location);
        assert!(dir.setup_component_public_keys_payload().is_ok());
        assert!(dir.election_event_context_payload().is_ok());
        for (i, p) in dir.control_component_public_keys_payload_iter() {
            assert!(p.is_ok());
            assert_eq!(p.unwrap().control_component_public_keys.node_id, i)
        }
        let expected = test_all_context_vcs_paths();
        for d in dir.vcs_directories().iter() {
            let j = expected
                .iter()
                .position(|l| &d.get_location() == l)
                .unwrap();
            assert_eq!(d.get_location(), expected[j])
        }
    }

    #[test]
    fn test_context_vcs_dir() {
        let location = test_context_verification_card_set_path();
        let dir = ContextVCSDirectory::new(&location);
        assert_eq!(dir.get_location(), location);
        assert!(dir.setup_component_tally_data_payload().is_ok());
    }

    #[test]
    fn test_completness() {
        let dir = ContextDirectory::new(&test_datasets_path());
        let c = dir.test_completness();
        assert!(c.is_ok());
        assert!(c.unwrap().is_empty());
    }
}

#[cfg(any(test, doc))]
#[allow(dead_code)]
pub mod mock {
    //! Module defining mocking structure for [VCSDirectory] and [SetupDirectory]
    //!
    //! The mocks read the correct data from the file. It is possible to change any data
    //! with the functions mock_
    use std::collections::HashMap;

    use super::{
        super::file_group::mock::{
            impl_iterator_over_data_payload_mock, mock_payload_iter, wrap_payload_iter,
            MockFileGroupIter,
        },
        super::mock::{mock_payload, wrap_file_group_getter, wrap_payload_getter},
        *,
    };

    /// Mock for [MockContextVCSDirectory]
    pub struct MockContextVCSDirectory {
        location: PathBuf,
        dir: ContextVCSDirectory,
        mocked_setup_component_tally_data_payload_file: Option<File>,
        mocked_setup_component_tally_data_payload:
            Option<Result<Box<SetupComponentTallyDataPayload>, FileStructureError>>,
        mocked_get_name: Option<String>,
    }

    /// Mock for [MockContextDirectory]
    pub struct MockContextDirectory {
        location: PathBuf,
        dir: ContextDirectory,
        mocked_setup_component_public_keys_payload_file: Option<File>,
        mocked_election_event_context_payload_file: Option<File>,
        mocked_election_event_configuration_file: Option<File>,
        mocked_control_component_public_keys_payload_group: Option<FileGroup>,
        mocked_setup_component_public_keys_payload:
            Option<Result<Box<SetupComponentPublicKeysPayload>, FileStructureError>>,
        mocked_election_event_context_payload:
            Option<Result<Box<ElectionEventContextPayload>, FileStructureError>>,
        mocked_election_event_configuration:
            Option<Result<Box<ElectionEventConfiguration>, FileStructureError>>,
        mocked_control_component_public_keys_payloads:
            HashMap<usize, ControlComponentPublicKeysPayloadAsResult>,
        vcs_directories: Vec<MockContextVCSDirectory>,
    }

    impl_iterator_over_data_payload_mock!(
        ControlComponentPublicKeysPayload,
        ControlComponentPublicKeysPayloadAsResult,
        ControlComponentPublicKeysPayloadAsResultIter,
        MockControlComponentPublicKeysPayloadAsResultIter
    );

    impl ContextVCSDirectoryTrait for MockContextVCSDirectory {
        wrap_file_group_getter!(
            setup_component_tally_data_payload_file,
            mocked_setup_component_tally_data_payload_file,
            File
        );
        wrap_payload_getter!(
            setup_component_tally_data_payload,
            mocked_setup_component_tally_data_payload,
            SetupComponentTallyDataPayload
        );

        fn get_name(&self) -> String {
            match &self.mocked_get_name {
                Some(e) => e.clone(),
                None => self.dir.get_name(),
            }
        }
    }

    impl_completness_test_trait_for_context_vcs!(MockContextVCSDirectory);

    impl ContextDirectoryTrait for MockContextDirectory {
        type VCSDirType = MockContextVCSDirectory;
        type ControlComponentPublicKeysPayloadAsResultIterType =
            MockControlComponentPublicKeysPayloadAsResultIter;

        wrap_file_group_getter!(
            setup_component_public_keys_payload_file,
            mocked_setup_component_public_keys_payload_file,
            File
        );
        wrap_file_group_getter!(
            election_event_context_payload_file,
            mocked_election_event_context_payload_file,
            File
        );
        wrap_file_group_getter!(
            election_event_configuration_file,
            mocked_election_event_configuration_file,
            File
        );
        wrap_file_group_getter!(
            control_component_public_keys_payload_group,
            mocked_control_component_public_keys_payload_group,
            FileGroup
        );

        fn vcs_directories(&self) -> &Vec<MockContextVCSDirectory> {
            &self.vcs_directories
        }

        wrap_payload_getter!(
            setup_component_public_keys_payload,
            mocked_setup_component_public_keys_payload,
            SetupComponentPublicKeysPayload
        );
        wrap_payload_getter!(
            election_event_context_payload,
            mocked_election_event_context_payload,
            ElectionEventContextPayload
        );
        wrap_payload_getter!(
            election_event_configuration,
            mocked_election_event_configuration,
            ElectionEventConfiguration
        );

        wrap_payload_iter!(
            control_component_public_keys_payload_iter,
            ControlComponentPublicKeysPayloadAsResultIterType,
            MockControlComponentPublicKeysPayloadAsResultIter,
            mocked_control_component_public_keys_payloads
        );
    }

    impl_completness_test_trait_for_context!(MockContextDirectory);

    impl MockContextVCSDirectory {
        /// New [MockVCSDirectory]
        pub fn new(location: &Path) -> Self {
            MockContextVCSDirectory {
                location: location.to_path_buf(),
                dir: ContextVCSDirectory::new(location),
                mocked_setup_component_tally_data_payload_file: None,
                mocked_setup_component_tally_data_payload: None,
                mocked_get_name: None,
            }
        }

        pub fn mock_setup_component_tally_data_payload_file(&mut self, data: &File) {
            self.mocked_setup_component_tally_data_payload_file = Some(data.clone());
        }
        mock_payload!(
            mock_setup_component_tally_data_payload,
            mocked_setup_component_tally_data_payload,
            SetupComponentTallyDataPayload
        );

        pub fn mock_get_name(&mut self, data: &str) {
            self.mocked_get_name = Some(data.to_string())
        }
    }

    impl MockContextDirectory {
        /// New
        pub fn new(data_location: &Path) -> Self {
            let setup_dir = ContextDirectory::new(data_location);
            let vcs_dirs: Vec<MockContextVCSDirectory> = setup_dir
                .vcs_directories
                .iter()
                .map(|d| MockContextVCSDirectory::new(&d.location))
                .collect();
            MockContextDirectory {
                location: setup_dir.location.to_owned(),
                dir: setup_dir,
                mocked_setup_component_public_keys_payload_file: None,
                mocked_election_event_context_payload_file: None,
                mocked_election_event_configuration_file: None,
                mocked_control_component_public_keys_payload_group: None,
                mocked_setup_component_public_keys_payload: None,
                mocked_election_event_context_payload: None,
                mocked_election_event_configuration: None,
                mocked_control_component_public_keys_payloads: HashMap::new(),
                vcs_directories: vcs_dirs,
            }
        }

        /// Get the vcs_directories mutable in order to mock them
        pub fn vcs_directories_mut(&mut self) -> Vec<&mut MockContextVCSDirectory> {
            self.vcs_directories.iter_mut().collect()
        }

        pub fn mock_setup_component_public_keys_payload_file(&mut self, data: &File) {
            self.mocked_setup_component_public_keys_payload_file = Some(data.clone());
        }
        pub fn mock_election_event_context_payload_file(&mut self, data: &File) {
            self.mocked_election_event_context_payload_file = Some(data.clone());
        }
        pub fn mock_election_event_configuration_file(&mut self, data: &File) {
            self.mocked_election_event_configuration_file = Some(data.clone());
        }
        pub fn mock_control_component_public_keys_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_public_keys_payload_group = Some(data.clone());
        }

        mock_payload!(
            mock_setup_component_public_keys_payload,
            mocked_setup_component_public_keys_payload,
            SetupComponentPublicKeysPayload
        );
        mock_payload!(
            mock_election_event_context_payload,
            mocked_election_event_context_payload,
            ElectionEventContextPayload
        );
        mock_payload!(
            mock_election_event_configuration,
            mocked_election_event_configuration,
            ElectionEventConfiguration
        );

        mock_payload_iter!(
            mock_control_component_public_keys_payloads,
            mocked_control_component_public_keys_payloads,
            ControlComponentPublicKeysPayload
        );
    }
}
