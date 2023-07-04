use super::{
    file::{create_file, File},
    file_group::{
        add_type_for_file_group_iter_trait, impl_iterator_over_data_payload, FileGroup,
        FileGroupIter, FileGroupIterTrait,
    },
};
use crate::{
    constants::{SETUP_DIR_NAME, VCS_DIR_NAME},
    data_structures::{
        create_verifier_setup_data_type,
        setup::{
            control_component_code_shares_payload::ControlComponentCodeSharesPayload,
            control_component_public_keys_payload::ControlComponentPublicKeysPayload,
            election_event_configuration::ElectionEventConfiguration,
            election_event_context_payload::ElectionEventContextPayload,
            encryption_parameters_payload::EncryptionParametersPayload,
            setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
            setup_component_tally_data_payload::SetupComponentTallyDataPayload,
            setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
            VerifierSetupDataType,
        },
        VerifierDataType, VerifierSetupDataTrait,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct SetupDirectory {
    location: PathBuf,
    encryption_parameters_payload_file: File,
    setup_component_public_keys_payload_file: File,
    election_event_context_payload_file: File,
    election_event_configuration_file: File,
    control_component_public_keys_payload_group: FileGroup,
    vcs_directories: Vec<VCSDirectory>,
}

#[derive(Clone)]
pub struct VCSDirectory {
    location: PathBuf,
    setup_component_tally_data_payload_file: File,
    setup_component_verification_data_payload_group: FileGroup,
    control_component_code_shares_payload_group: FileGroup,
}

/// Trait to set the necessary functions for the struct [SetupDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait SetupDirectoryTrait {
    type VCSDirType: VCSDirectoryTrait;
    add_type_for_file_group_iter_trait!(
        ControlComponentPublicKeysPayloadAsResultIterType,
        ControlComponentPublicKeysPayloadAsResult
    );

    fn encryption_parameters_payload_file(&self) -> &File;
    fn setup_component_public_keys_payload_file(&self) -> &File;
    fn election_event_context_payload_file(&self) -> &File;
    fn election_event_configuration_file(&self) -> &File;
    fn control_component_public_keys_payload_group(&self) -> &FileGroup;
    fn vcs_directories(&self) -> &Vec<Self::VCSDirType>;
    fn encryption_parameters_payload(&self) -> anyhow::Result<Box<EncryptionParametersPayload>>;
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
pub trait VCSDirectoryTrait {
    add_type_for_file_group_iter_trait!(
        SetupComponentVerificationDataPayloadAsResultIterType,
        SetupComponentVerificationDataPayloadAsResult
    );
    add_type_for_file_group_iter_trait!(
        ControlComponentCodeSharesPayloadAsResultIterType,
        ControlComponentCodeSharesPayloadAsResult
    );

    fn setup_component_tally_data_payload_file(&self) -> &File;
    fn setup_component_verification_data_payload_group(&self) -> &FileGroup;
    fn control_component_code_shares_payload_group(&self) -> &FileGroup;
    fn setup_component_tally_data_payload(
        &self,
    ) -> anyhow::Result<Box<SetupComponentTallyDataPayload>>;
    fn setup_component_verification_data_payload_iter(
        &self,
    ) -> Self::SetupComponentVerificationDataPayloadAsResultIterType;

    fn control_component_code_shares_payload_iter(
        &self,
    ) -> Self::ControlComponentCodeSharesPayloadAsResultIterType;
    fn get_name(&self) -> String;
}

impl_iterator_over_data_payload!(
    ControlComponentPublicKeysPayload,
    control_component_public_keys_payload,
    ControlComponentPublicKeysPayloadAsResult,
    ControlComponentPublicKeysPayloadAsResultIter
);

impl_iterator_over_data_payload!(
    SetupComponentVerificationDataPayload,
    setup_component_verification_data_payload,
    SetupComponentVerificationDataPayloadAsResult,
    SetupComponentVerificationDataPayloadAsResultIter
);

impl_iterator_over_data_payload!(
    ControlComponentCodeSharesPayload,
    control_component_code_shares_payload,
    ControlComponentCodeSharesPayloadAsResult,
    ControlComponentCodeSharesPayloadAsResultIter
);

impl SetupDirectory {
    /// New [SetupDirectory]
    #[allow(clippy::redundant_clone)]
    pub fn new(data_location: &Path) -> Self {
        let location = data_location.join(SETUP_DIR_NAME);
        let mut res = Self {
            location: location.to_path_buf(),
            encryption_parameters_payload_file: create_file!(
                location,
                Setup,
                VerifierSetupDataType::EncryptionParametersPayload
            ),
            setup_component_public_keys_payload_file: create_file!(
                location,
                Setup,
                VerifierSetupDataType::SetupComponentPublicKeysPayload
            ),
            election_event_context_payload_file: create_file!(
                location,
                Setup,
                VerifierSetupDataType::ElectionEventContextPayload
            ),
            election_event_configuration_file: create_file!(
                location,
                Setup,
                VerifierSetupDataType::ElectionEventConfiguration
            ),
            control_component_public_keys_payload_group: FileGroup::new(
                &location,
                create_verifier_setup_data_type!(Setup, ControlComponentPublicKeysPayload),
            ),
            vcs_directories: vec![],
        };
        let vcs_path = location.join(VCS_DIR_NAME);
        if vcs_path.is_dir() {
            for re in fs::read_dir(&vcs_path).unwrap() {
                let e = re.unwrap().path();
                if e.is_dir() {
                    res.vcs_directories.push(VCSDirectory::new(&e))
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

impl SetupDirectoryTrait for SetupDirectory {
    type VCSDirType = VCSDirectory;
    type ControlComponentPublicKeysPayloadAsResultIterType =
        ControlComponentPublicKeysPayloadAsResultIter;

    fn encryption_parameters_payload_file(&self) -> &File {
        &self.encryption_parameters_payload_file
    }
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
    fn vcs_directories(&self) -> &Vec<VCSDirectory> {
        &self.vcs_directories
    }
    fn encryption_parameters_payload(&self) -> anyhow::Result<Box<EncryptionParametersPayload>> {
        self.encryption_parameters_payload_file
            .get_data()
            .map_err(|e| e.context("in encryption_parameters_payload"))
            .map(|d| Box::new(d.encryption_parameters_payload().unwrap().clone()))
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

impl VCSDirectory {
    /// New [VCSDirectory]
    pub fn new(location: &Path) -> Self {
        Self {
            location: location.to_path_buf(),
            setup_component_tally_data_payload_file: create_file!(
                location,
                Setup,
                VerifierSetupDataType::SetupComponentTallyDataPayload
            ),
            setup_component_verification_data_payload_group: FileGroup::new(
                location,
                create_verifier_setup_data_type!(Setup, SetupComponentVerificationDataPayload),
            ),
            control_component_code_shares_payload_group: FileGroup::new(
                location,
                create_verifier_setup_data_type!(Setup, ControlComponentCodeSharesPayload),
            ),
        }
    }

    /// Get location
    #[allow(dead_code)]
    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

impl VCSDirectoryTrait for VCSDirectory {
    type SetupComponentVerificationDataPayloadAsResultIterType =
        SetupComponentVerificationDataPayloadAsResultIter;
    type ControlComponentCodeSharesPayloadAsResultIterType =
        ControlComponentCodeSharesPayloadAsResultIter;

    fn setup_component_tally_data_payload_file(&self) -> &File {
        &self.setup_component_tally_data_payload_file
    }
    fn setup_component_verification_data_payload_group(&self) -> &FileGroup {
        &self.setup_component_verification_data_payload_group
    }
    fn control_component_code_shares_payload_group(&self) -> &FileGroup {
        &self.control_component_code_shares_payload_group
    }
    fn setup_component_tally_data_payload(
        &self,
    ) -> anyhow::Result<Box<SetupComponentTallyDataPayload>> {
        self.setup_component_tally_data_payload_file
            .get_data()
            .map_err(|e| e.context("in setup_component_tally_data_payload"))
            .map(|d| Box::new(d.setup_component_tally_data_payload().unwrap().clone()))
    }

    fn setup_component_verification_data_payload_iter(
        &self,
    ) -> Self::SetupComponentVerificationDataPayloadAsResultIterType {
        FileGroupIter::new(&self.setup_component_verification_data_payload_group)
    }

    fn control_component_code_shares_payload_iter(
        &self,
    ) -> Self::ControlComponentCodeSharesPayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_code_shares_payload_group)
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
    use crate::constants::test::dataset_tally_path as get_location;

    #[test]
    fn test_setup_dir() {
        let location = get_location();
        let setup_location = location.join("setup");
        let vcs_location = setup_location.join("verification_card_sets");
        let dir = SetupDirectory::new(&location);
        assert_eq!(dir.get_location(), setup_location);
        assert!(dir.encryption_parameters_payload().is_ok());
        assert!(dir.setup_component_public_keys_payload().is_ok());
        assert!(dir.election_event_context_payload().is_ok());
        for (i, p) in dir.control_component_public_keys_payload_iter() {
            assert!(p.is_ok());
            assert_eq!(p.unwrap().control_component_public_keys.node_id, i)
        }
        let expected = vec![
            "A4C66F60561BE5EEFD017E5EF6650563",
            "C165B503B8A3DDD99402EF4DC4267079",
            "681B3488DE4CD4AD7FCED14B7A654169",
            "3800FC91D8652090731C69F807565BEF",
        ];
        for (i, d) in dir.vcs_directories().iter().enumerate() {
            assert_eq!(d.get_location(), vcs_location.join(expected[i]))
        }
    }

    #[test]
    fn test_vcs_dir() {
        let location = get_location()
            .join("setup")
            .join("verification_card_sets")
            .join("3800FC91D8652090731C69F807565BEF");
        let dir = VCSDirectory::new(&location);
        assert_eq!(dir.get_location(), location);
        assert!(dir.setup_component_tally_data_payload().is_ok());
        for (i, p) in dir.control_component_code_shares_payload_iter() {
            assert!(p.is_ok());
            for k in p.unwrap().iter() {
                assert_eq!(k.chunk_id, i)
            }
        }
        for (i, p) in dir.setup_component_verification_data_payload_iter() {
            assert!(p.is_ok());
            assert_eq!(p.unwrap().chunk_id, i)
        }
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

    /// Mock for [VCSDirectory]
    pub struct MockVCSDirectory {
        dir: VCSDirectory,
        mocked_setup_component_tally_data_payload_file: Option<File>,
        mocked_setup_component_verification_data_payload_group: Option<FileGroup>,
        mocked_control_component_code_shares_payload_group: Option<FileGroup>,
        mocked_setup_component_tally_data_payload:
            Option<anyhow::Result<Box<SetupComponentTallyDataPayload>>>,
        mocked_setup_component_verification_data_payloads:
            HashMap<usize, SetupComponentVerificationDataPayloadAsResult>,
        mocked_control_component_code_shares_payloads:
            HashMap<usize, ControlComponentCodeSharesPayloadAsResult>,
        mocked_get_name: Option<String>,
    }

    impl_iterator_over_data_payload_mock!(
        SetupComponentVerificationDataPayload,
        SetupComponentVerificationDataPayloadAsResult,
        SetupComponentVerificationDataPayloadAsResultIter,
        MockSetupComponentVerificationDataPayloadAsResultIter
    );

    impl_iterator_over_data_payload_mock!(
        ControlComponentCodeSharesPayload,
        ControlComponentCodeSharesPayloadAsResult,
        ControlComponentCodeSharesPayloadAsResultIter,
        MockControlComponentCodeSharesPayloadAsResultIter
    );

    /// Mock for [SetupDirectory]
    pub struct MockSetupDirectory {
        dir: SetupDirectory,
        mocked_encryption_parameters_payload_file: Option<File>,
        mocked_setup_component_public_keys_payload_file: Option<File>,
        mocked_election_event_context_payload_file: Option<File>,
        mocked_election_event_configuration_file: Option<File>,
        mocked_control_component_public_keys_payload_group: Option<FileGroup>,
        mocked_encryption_parameters_payload:
            Option<anyhow::Result<Box<EncryptionParametersPayload>>>,
        mocked_setup_component_public_keys_payload:
            Option<anyhow::Result<Box<SetupComponentPublicKeysPayload>>>,
        mocked_election_event_context_payload:
            Option<anyhow::Result<Box<ElectionEventContextPayload>>>,
        mocked_election_event_configuration:
            Option<anyhow::Result<Box<ElectionEventConfiguration>>>,
        mocked_control_component_public_keys_payloads:
            HashMap<usize, ControlComponentPublicKeysPayloadAsResult>,
        vcs_directories: Vec<MockVCSDirectory>,
    }

    impl_iterator_over_data_payload_mock!(
        ControlComponentPublicKeysPayload,
        ControlComponentPublicKeysPayloadAsResult,
        ControlComponentPublicKeysPayloadAsResultIter,
        MockControlComponentPublicKeysPayloadAsResultIter
    );

    impl VCSDirectoryTrait for MockVCSDirectory {
        type SetupComponentVerificationDataPayloadAsResultIterType =
            MockSetupComponentVerificationDataPayloadAsResultIter;
        type ControlComponentCodeSharesPayloadAsResultIterType =
            MockControlComponentCodeSharesPayloadAsResultIter;

        wrap_file_group_getter!(
            setup_component_tally_data_payload_file,
            mocked_setup_component_tally_data_payload_file,
            File
        );
        wrap_file_group_getter!(
            setup_component_verification_data_payload_group,
            mocked_setup_component_verification_data_payload_group,
            FileGroup
        );
        wrap_file_group_getter!(
            control_component_code_shares_payload_group,
            mocked_control_component_code_shares_payload_group,
            FileGroup
        );
        wrap_payload_getter!(
            setup_component_tally_data_payload,
            mocked_setup_component_tally_data_payload,
            SetupComponentTallyDataPayload
        );

        wrap_payload_iter!(
            setup_component_verification_data_payload_iter,
            SetupComponentVerificationDataPayloadAsResultIterType,
            MockSetupComponentVerificationDataPayloadAsResultIter,
            mocked_setup_component_verification_data_payloads
        );

        wrap_payload_iter!(
            control_component_code_shares_payload_iter,
            ControlComponentCodeSharesPayloadAsResultIterType,
            MockControlComponentCodeSharesPayloadAsResultIter,
            mocked_control_component_code_shares_payloads
        );

        fn get_name(&self) -> String {
            match &self.mocked_get_name {
                Some(e) => e.clone(),
                None => self.dir.get_name(),
            }
        }
    }

    impl SetupDirectoryTrait for MockSetupDirectory {
        type VCSDirType = MockVCSDirectory;
        type ControlComponentPublicKeysPayloadAsResultIterType =
            MockControlComponentPublicKeysPayloadAsResultIter;

        wrap_file_group_getter!(
            encryption_parameters_payload_file,
            mocked_encryption_parameters_payload_file,
            File
        );
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

        fn vcs_directories(&self) -> &Vec<MockVCSDirectory> {
            &self.vcs_directories
        }

        wrap_payload_getter!(
            encryption_parameters_payload,
            mocked_encryption_parameters_payload,
            EncryptionParametersPayload
        );
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

    impl MockVCSDirectory {
        /// New [MockVCSDirectory]
        pub fn new(location: &Path) -> Self {
            MockVCSDirectory {
                dir: VCSDirectory::new(location),
                mocked_setup_component_tally_data_payload_file: None,
                mocked_setup_component_verification_data_payload_group: None,
                mocked_control_component_code_shares_payload_group: None,
                mocked_setup_component_tally_data_payload: None,
                mocked_setup_component_verification_data_payloads: HashMap::new(),
                mocked_control_component_code_shares_payloads: HashMap::new(),
                mocked_get_name: None,
            }
        }

        pub fn mock_setup_component_tally_data_payload_file(&mut self, data: &File) {
            self.mocked_setup_component_tally_data_payload_file = Some(data.clone());
        }
        pub fn mock_setup_component_verification_data_payload_group(&mut self, data: &FileGroup) {
            self.mocked_setup_component_verification_data_payload_group = Some(data.clone());
        }
        pub fn mock_control_component_code_shares_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_code_shares_payload_group = Some(data.clone());
        }
        mock_payload!(
            mock_setup_component_tally_data_payload,
            mocked_setup_component_tally_data_payload,
            SetupComponentTallyDataPayload
        );

        mock_payload_iter!(
            mock_setup_component_verification_data_payloads,
            mocked_setup_component_verification_data_payloads,
            SetupComponentVerificationDataPayload
        );

        mock_payload_iter!(
            mock_control_component_code_shares_payloads,
            mocked_control_component_code_shares_payloads,
            ControlComponentCodeSharesPayload
        );

        pub fn mock_get_name(&mut self, data: &str) {
            self.mocked_get_name = Some(data.to_string())
        }
    }

    impl MockSetupDirectory {
        /// New
        pub fn new(data_location: &Path) -> Self {
            let setup_dir = SetupDirectory::new(data_location);
            let vcs_dirs: Vec<MockVCSDirectory> = setup_dir
                .vcs_directories
                .iter()
                .map(|d| MockVCSDirectory::new(&d.location))
                .collect();
            MockSetupDirectory {
                dir: setup_dir,
                mocked_encryption_parameters_payload_file: None,
                mocked_setup_component_public_keys_payload_file: None,
                mocked_election_event_context_payload_file: None,
                mocked_election_event_configuration_file: None,
                mocked_control_component_public_keys_payload_group: None,
                mocked_encryption_parameters_payload: None,
                mocked_setup_component_public_keys_payload: None,
                mocked_election_event_context_payload: None,
                mocked_election_event_configuration: None,
                mocked_control_component_public_keys_payloads: HashMap::new(),
                vcs_directories: vcs_dirs,
            }
        }

        /// Get the vcs_directories mutable in order to mock them
        pub fn vcs_directories_mut(&mut self) -> Vec<&mut MockVCSDirectory> {
            self.vcs_directories.iter_mut().collect()
        }

        pub fn mock_encryption_parameters_payload_file(&mut self, data: &File) {
            self.mocked_encryption_parameters_payload_file = Some(data.clone());
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
            mock_encryption_parameters_payload,
            mocked_encryption_parameters_payload,
            EncryptionParametersPayload
        );
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
