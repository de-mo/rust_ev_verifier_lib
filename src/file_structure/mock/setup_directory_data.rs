use super::{
    impl_itertor_for_mocked_group_type, impl_mock_methods_for_mocked_group,
    impl_trait_get_method_for_mocked_group, MockFileGroupIter,
};
use crate::{
    data_structures::{ControlComponentCodeSharesPayload, SetupComponentVerificationDataPayload},
    file_structure::{
        file_group::FileGroupIterTrait,
        setup_directory::{
            impl_completness_test_trait_for_setup, impl_completness_test_trait_for_setup_vcs,
            ControlComponentCodeSharesPayloadAsResultIter,
            SetupComponentVerificationDataPayloadAsResultIter, SetupDirectory, SetupVCSDirectory,
            SetupVCSDirectoryTrait,
        },
        CompletnessTestTrait, FileStructureError, SetupDirectoryTrait,
    },
};
use paste::paste;
use std::{collections::HashMap, path::Path};

/// Mock for [SetupDirectory]
pub struct MockSetupDirectory {
    dir: SetupDirectory,
    vcs_directories: Vec<MockSetupVCSDirectory>,
}

/// Mock for [SetupVCSDirectory]
pub struct MockSetupVCSDirectory {
    dir: SetupVCSDirectory,
    mocked_setup_component_verification_data_payload:
        HashMap<usize, Box<SetupComponentVerificationDataPayload>>,
    mocked_setup_component_verification_data_payload_deleted: Vec<usize>,
    mocked_setup_component_verification_data_payload_errors: HashMap<usize, String>,
    mocked_control_component_code_shares_payload:
        HashMap<usize, Box<ControlComponentCodeSharesPayload>>,
    mocked_control_component_code_shares_payload_deleted: Vec<usize>,
    mocked_control_component_code_shares_payload_errors: HashMap<usize, String>,
}

impl_completness_test_trait_for_setup_vcs!(MockSetupVCSDirectory);
impl_completness_test_trait_for_setup!(MockSetupDirectory);

impl SetupDirectoryTrait for MockSetupDirectory {
    type VCSDirType = MockSetupVCSDirectory;

    fn vcs_directories(&self) -> &[Self::VCSDirType] {
        &self.vcs_directories
    }

    fn location(&self) -> &Path {
        self.dir.location()
    }
}

impl MockSetupDirectory {
    /// New
    pub fn new(data_location: &Path) -> Self {
        let setup_dir = SetupDirectory::new(data_location);
        let vcs_dirs: Vec<MockSetupVCSDirectory> = setup_dir
            .vcs_directories()
            .iter()
            .map(|d| MockSetupVCSDirectory::new(d.location()))
            .collect();
        MockSetupDirectory {
            dir: setup_dir,
            vcs_directories: vcs_dirs,
        }
    }
}

impl_itertor_for_mocked_group_type!(SetupComponentVerificationDataPayload);
impl_itertor_for_mocked_group_type!(ControlComponentCodeSharesPayload);

impl SetupVCSDirectoryTrait for MockSetupVCSDirectory {
    type SetupComponentVerificationDataPayloadAsResultIterType =
        MockSetupComponentVerificationDataPayloadAsResultIter;
    type ControlComponentCodeSharesPayloadAsResultIterType =
        MockControlComponentCodeSharesPayloadAsResultIter;

    fn setup_component_verification_data_payload_group(
        &self,
    ) -> &crate::file_structure::file_group::FileGroup {
        todo!()
    }

    fn control_component_code_shares_payload_group(
        &self,
    ) -> &crate::file_structure::file_group::FileGroup {
        todo!()
    }

    impl_trait_get_method_for_mocked_group!(
        setup_component_verification_data_payload,
        SetupComponentVerificationDataPayload
    );

    impl_trait_get_method_for_mocked_group!(
        control_component_code_shares_payload,
        ControlComponentCodeSharesPayload
    );

    fn name(&self) -> String {
        self.dir.name()
    }

    fn location(&self) -> &Path {
        self.dir.location()
    }
}

impl MockSetupVCSDirectory {
    pub fn new(location: &Path) -> Self {
        MockSetupVCSDirectory {
            dir: SetupVCSDirectory::new(location),
            mocked_setup_component_verification_data_payload: HashMap::new(),
            mocked_setup_component_verification_data_payload_deleted: vec![],
            mocked_control_component_code_shares_payload_errors: HashMap::new(),
            mocked_control_component_code_shares_payload: HashMap::new(),
            mocked_control_component_code_shares_payload_deleted: vec![],
            mocked_setup_component_verification_data_payload_errors: HashMap::new(),
        }
    }

    impl_mock_methods_for_mocked_group!(
        setup_component_verification_data_payload,
        SetupComponentVerificationDataPayload
    );

    impl_mock_methods_for_mocked_group!(
        control_component_code_shares_payload,
        ControlComponentCodeSharesPayload
    );
}
