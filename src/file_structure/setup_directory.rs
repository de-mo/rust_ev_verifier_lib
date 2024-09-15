//! Module to implement the setup directory

use super::{
    file_group::{
        add_type_for_file_group_iter_trait, impl_iterator_over_data_payload, FileGroup,
        FileGroupIter, FileGroupIterTrait,
    },
    CompletnessTestTrait, FileStructureError,
};
use crate::{
    config::Config,
    data_structures::{
        create_verifier_setup_data_type,
        setup::{
            control_component_code_shares_payload::ControlComponentCodeSharesPayload,
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

/// The setup directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct SetupDirectory {
    location: PathBuf,
    vcs_directories: Vec<SetupVCSDirectory>,
}

/// The vcs directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct SetupVCSDirectory {
    location: PathBuf,
    setup_component_verification_data_payload_group: FileGroup,
    control_component_code_shares_payload_group: FileGroup,
}

/// Trait to set the necessary functions for the struct [SetupDirectory] that
/// are used during the verifications
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait SetupDirectoryTrait: CompletnessTestTrait {
    type VCSDirType: SetupVCSDirectoryTrait;
    fn vcs_directories(&self) -> &Vec<Self::VCSDirType>;

    /// Collect the names of the vcs directories
    fn vcs_directory_names(&self) -> Vec<String> {
        self.vcs_directories()
            .iter()
            .map(|d| d.get_name())
            .collect()
    }
}

/// Trait to set the necessary functions for the struct [SetupVCSDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait SetupVCSDirectoryTrait: CompletnessTestTrait {
    add_type_for_file_group_iter_trait!(
        SetupComponentVerificationDataPayloadAsResultIterType,
        SetupComponentVerificationDataPayloadAsResult
    );
    add_type_for_file_group_iter_trait!(
        ControlComponentCodeSharesPayloadAsResultIterType,
        ControlComponentCodeSharesPayloadAsResult
    );

    fn setup_component_verification_data_payload_group(&self) -> &FileGroup;
    fn control_component_code_shares_payload_group(&self) -> &FileGroup;
    fn setup_component_verification_data_payload_iter(
        &self,
    ) -> Self::SetupComponentVerificationDataPayloadAsResultIterType;

    fn control_component_code_shares_payload_iter(
        &self,
    ) -> Self::ControlComponentCodeSharesPayloadAsResultIterType;
    fn get_name(&self) -> String;
}

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
        let location = data_location.join(Config::setup_dir_name());
        let mut res = Self {
            location: location.to_path_buf(),
            vcs_directories: vec![],
        };
        let vcs_path = location.join(Config::vcs_dir_name());
        if vcs_path.is_dir() {
            for re in fs::read_dir(&vcs_path).unwrap() {
                let e = re.unwrap().path();
                if e.is_dir() {
                    res.vcs_directories.push(SetupVCSDirectory::new(&e))
                }
            }
        }
        res
    }

    /// Get location

    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

impl SetupDirectoryTrait for SetupDirectory {
    type VCSDirType = SetupVCSDirectory;
    fn vcs_directories(&self) -> &Vec<SetupVCSDirectory> {
        &self.vcs_directories
    }
}

macro_rules! impl_completness_test_trait_for_setup {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                let mut missings = vec![];
                if self.vcs_directories().is_empty() {
                    missings.push("No vcs directory found".to_string());
                }
                for d in self.vcs_directories().iter() {
                    missings.extend(d.test_completness()?)
                }
                Ok(missings)
            }
        }
    };
}
impl_completness_test_trait_for_setup!(SetupDirectory);

impl SetupVCSDirectory {
    /// New [VCSDirectory]
    pub fn new(location: &Path) -> Self {
        Self {
            location: location.to_path_buf(),
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

    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

macro_rules! impl_completness_test_trait_for_setup_vcs {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                let mut missings = vec![];
                if !self
                    .setup_component_verification_data_payload_group()
                    .has_elements()
                {
                    missings.push(format!(
                        "{:?}/setup_component_verification_data_payload does not exist",
                        self.location.file_name().unwrap()
                    ))
                }
                if !self
                    .control_component_code_shares_payload_group()
                    .has_elements()
                {
                    missings.push(format!(
                        "{:?}/control_component_code_shares_payload does not exist",
                        self.location.file_name().unwrap()
                    ))
                }
                Ok(missings)
            }
        }
    };
}
impl_completness_test_trait_for_setup_vcs!(SetupVCSDirectory);

impl SetupVCSDirectoryTrait for SetupVCSDirectory {
    type SetupComponentVerificationDataPayloadAsResultIterType =
        SetupComponentVerificationDataPayloadAsResultIter;
    type ControlComponentCodeSharesPayloadAsResultIterType =
        ControlComponentCodeSharesPayloadAsResultIter;

    fn setup_component_verification_data_payload_group(&self) -> &FileGroup {
        &self.setup_component_verification_data_payload_group
    }
    fn control_component_code_shares_payload_group(&self) -> &FileGroup {
        &self.control_component_code_shares_payload_group
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
    use crate::config::test::{
        test_all_setup_vcs_paths, test_datasets_path, test_datasets_setup_path,
        test_setup_verification_card_set_path,
    };

    #[test]
    fn test_setup_dir() {
        let setup_location = test_datasets_setup_path().join("setup");
        let dir = SetupDirectory::new(setup_location.parent().unwrap());
        assert_eq!(dir.get_location(), setup_location);
        let expected = test_all_setup_vcs_paths();
        for d in dir.vcs_directories().iter() {
            let j = expected.iter().position(|l| d.get_location() == l).unwrap();
            assert_eq!(d.get_location(), expected[j])
        }
    }

    #[test]
    fn test_vcs_dir() {
        let location = test_setup_verification_card_set_path();
        let dir = SetupVCSDirectory::new(&location);
        assert_eq!(dir.get_location(), location);
        for (i, p) in dir.control_component_code_shares_payload_iter() {
            assert!(p.is_ok());
            for k in p.unwrap().0.iter() {
                assert_eq!(k.chunk_id, i)
            }
        }
        for (i, p) in dir.setup_component_verification_data_payload_iter() {
            assert!(p.is_ok());
            assert_eq!(p.unwrap().chunk_id, i)
        }
    }

    #[test]
    fn test_completness() {
        let dir = SetupDirectory::new(&test_datasets_path());
        let c = dir.test_completness();
        assert!(c.is_ok());
        assert!(c.unwrap().is_empty());
    }
}

#[cfg(any(test, doc))]

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
        super::mock::wrap_file_group_getter,
        *,
    };

    /// Mock for [SetupVCSDirectory]
    pub struct MockSetupVCSDirectory {
        location: PathBuf,
        dir: SetupVCSDirectory,
        mocked_setup_component_verification_data_payload_group: Option<FileGroup>,
        mocked_control_component_code_shares_payload_group: Option<FileGroup>,
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
        location: PathBuf,
        dir: SetupDirectory,
        vcs_directories: Vec<MockSetupVCSDirectory>,
    }

    impl_completness_test_trait_for_setup!(MockSetupDirectory);
    impl_completness_test_trait_for_setup_vcs!(MockSetupVCSDirectory);

    impl SetupVCSDirectoryTrait for MockSetupVCSDirectory {
        type SetupComponentVerificationDataPayloadAsResultIterType =
            MockSetupComponentVerificationDataPayloadAsResultIter;
        type ControlComponentCodeSharesPayloadAsResultIterType =
            MockControlComponentCodeSharesPayloadAsResultIter;

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
        type VCSDirType = MockSetupVCSDirectory;

        fn vcs_directories(&self) -> &Vec<MockSetupVCSDirectory> {
            &self.vcs_directories
        }
    }

    impl MockSetupVCSDirectory {
        /// New [MockSetupVCSDirectory]
        pub fn new(location: &Path) -> Self {
            MockSetupVCSDirectory {
                location: location.to_path_buf(),
                dir: SetupVCSDirectory::new(location),
                mocked_setup_component_verification_data_payload_group: None,
                mocked_control_component_code_shares_payload_group: None,
                mocked_setup_component_verification_data_payloads: HashMap::new(),
                mocked_control_component_code_shares_payloads: HashMap::new(),
                mocked_get_name: None,
            }
        }

        pub fn mock_setup_component_verification_data_payload_group(&mut self, data: &FileGroup) {
            self.mocked_setup_component_verification_data_payload_group = Some(data.clone());
        }
        pub fn mock_control_component_code_shares_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_code_shares_payload_group = Some(data.clone());
        }

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
            let vcs_dirs: Vec<MockSetupVCSDirectory> = setup_dir
                .vcs_directories
                .iter()
                .map(|d| MockSetupVCSDirectory::new(&d.location))
                .collect();
            MockSetupDirectory {
                location: setup_dir.location.to_owned(),
                dir: setup_dir,
                vcs_directories: vcs_dirs,
            }
        }

        /// Get the vcs_directories mutable in order to mock them
        pub fn vcs_directories_mut(&mut self) -> Vec<&mut MockSetupVCSDirectory> {
            self.vcs_directories.iter_mut().collect()
        }
    }
}
