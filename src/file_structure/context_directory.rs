//! Module to implement the context directory

use super::{
    file::{create_file, File},
    file_group::{
        add_type_for_file_group_iter_trait, impl_iterator_over_data_payload, FileGroup,
        FileGroupIter, FileGroupIterTrait,
    },
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
pub trait ContextDirectoryTrait {
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
    ) -> anyhow::Result<Box<SetupComponentPublicKeysPayload>>;

    fn election_event_context_payload(&self) -> anyhow::Result<Box<ElectionEventContextPayload>>;
    fn election_event_configuration(&self) -> anyhow::Result<Box<ElectionEventConfiguration>>;

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> Self::ControlComponentPublicKeysPayloadAsResultIterType;
}

/// Trait to set the necessary functions for the struct [VCSDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait ContextVCSDirectoryTrait {
    fn setup_component_tally_data_payload_file(&self) -> &File;
    fn setup_component_tally_data_payload(
        &self,
    ) -> anyhow::Result<Box<SetupComponentTallyDataPayload>>;
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
    ) -> anyhow::Result<Box<SetupComponentPublicKeysPayload>> {
        self.setup_component_public_keys_payload_file
            .get_data()
            .map_err(|e| e.context("in setup_component_public_keys_payload"))
            .map(|d| Box::new(d.setup_component_public_keys_payload().unwrap().clone()))
    }

    fn election_event_context_payload(&self) -> anyhow::Result<Box<ElectionEventContextPayload>> {
        self.election_event_context_payload_file
            .get_data()
            .map_err(|e| e.context("in election_event_context_payload"))
            .map(|d| Box::new(d.election_event_context_payload().unwrap().clone()))
    }

    fn election_event_configuration(&self) -> anyhow::Result<Box<ElectionEventConfiguration>> {
        self.election_event_configuration_file
            .get_data()
            .map_err(|e| e.context("in election_event_configuration"))
            .map(|d| Box::new(d.election_event_configuration().unwrap().clone()))
    }

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> Self::ControlComponentPublicKeysPayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_public_keys_payload_group)
    }
}

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
    ) -> anyhow::Result<Box<SetupComponentTallyDataPayload>> {
        self.setup_component_tally_data_payload_file
            .get_data()
            .map_err(|e| e.context("in setup_component_tally_data_payload"))
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
        test_dataset_tally_path as get_location, test_verification_card_set_path,
    };

    #[test]
    fn test_setup_dir() {
        let location = get_location();
        let setup_location = location.join("setup");
        let vcs_location = setup_location.join("verification_card_sets");
        let dir = ContextDirectory::new(&location);
        assert_eq!(dir.get_location(), setup_location);
        assert!(dir.setup_component_public_keys_payload().is_ok());
        assert!(dir.election_event_context_payload().is_ok());
        for (i, p) in dir.control_component_public_keys_payload_iter() {
            assert!(p.is_ok());
            assert_eq!(p.unwrap().control_component_public_keys.node_id, i)
        }
        let expected = [
            "1B3775CB351C64AC33B754BA3A02AED2",
            "6F00E7676CF3D20E19346C7CBDF62A0A",
            "01983BA322FAA6C9365267EDF16DD323",
            "E29CAEF477BD4AE4519025542D510985",
        ];
        for d in dir.vcs_directories().iter() {
            let j = expected.iter().position(|l| &d.get_name() == l).unwrap();
            assert_eq!(d.get_location(), vcs_location.join(expected[j]))
        }
    }

    #[test]
    fn test_vcs_dir() {
        let location = test_verification_card_set_path();
        let dir = ContextVCSDirectory::new(&location);
        assert_eq!(dir.get_location(), location);
        assert!(dir.setup_component_tally_data_payload().is_ok());
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
    use anyhow::anyhow;

    /// Mock for [MockContextVCSDirectory]
    pub struct MockContextVCSDirectory {
        dir: ContextVCSDirectory,
        mocked_setup_component_tally_data_payload_file: Option<File>,
        mocked_setup_component_tally_data_payload:
            Option<anyhow::Result<Box<SetupComponentTallyDataPayload>>>,
        mocked_get_name: Option<String>,
    }

    /// Mock for [MockContextDirectory]
    pub struct MockContextDirectory {
        dir: ContextDirectory,
        mocked_setup_component_public_keys_payload_file: Option<File>,
        mocked_election_event_context_payload_file: Option<File>,
        mocked_election_event_configuration_file: Option<File>,
        mocked_control_component_public_keys_payload_group: Option<FileGroup>,
        mocked_setup_component_public_keys_payload:
            Option<anyhow::Result<Box<SetupComponentPublicKeysPayload>>>,
        mocked_election_event_context_payload:
            Option<anyhow::Result<Box<ElectionEventContextPayload>>>,
        mocked_election_event_configuration:
            Option<anyhow::Result<Box<ElectionEventConfiguration>>>,
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

    impl MockContextVCSDirectory {
        /// New [MockVCSDirectory]
        pub fn new(location: &Path) -> Self {
            MockContextVCSDirectory {
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
