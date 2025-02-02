use super::{
    impl_mock_methods_for_mocked_data, impl_mock_methods_for_mocked_group,
    impl_trait_get_method_for_mocked_data, impl_trait_get_method_for_mocked_group,
    FileGroupFileIter, MockFileGroupDataIter, MockFileGroupElement, MockedDataType,
};
use crate::{
    data_structures::{
        context::{
            setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
            setup_component_tally_data_payload::SetupComponentTallyDataPayload,
        },
        ControlComponentPublicKeysPayload, ElectionEventConfiguration, ElectionEventContextPayload,
    },
    file_structure::{
        context_directory::{
            impl_completness_test_trait_for_context, impl_completness_test_trait_for_context_vcs,
            ContextVCSDirectory, ContextVCSDirectoryTrait,
        },
        file::File,
        file_group::FileGroup,
        CompletnessTestTrait, ContextDirectory, ContextDirectoryTrait, FileStructureError,
    },
};
use paste::paste;
use std::{collections::HashMap, path::Path, sync::Arc};

/// Mock for [ContextVCSDirectory]
pub struct MockContextVCSDirectory {
    dir: ContextVCSDirectory,
    mocked_setup_component_tally_data_payload:
        Option<Box<MockedDataType<SetupComponentTallyDataPayload>>>,
}

/// Mock for [ContextDirectory]
pub struct MockContextDirectory {
    pub dir: ContextDirectory,
    mocked_setup_component_public_keys_payload:
        Option<Box<MockedDataType<SetupComponentPublicKeysPayload>>>,
    mocked_election_event_context_payload: Option<Box<MockedDataType<ElectionEventContextPayload>>>,
    mocked_election_event_configuration: Option<Box<MockedDataType<ElectionEventConfiguration>>>,
    mocked_control_component_public_keys_payload:
        HashMap<usize, Box<MockFileGroupElement<ControlComponentPublicKeysPayload>>>,
    vcs_directories: Vec<MockContextVCSDirectory>,
}

impl_completness_test_trait_for_context_vcs!(MockContextVCSDirectory);
impl_completness_test_trait_for_context!(MockContextDirectory);

impl MockContextDirectory {
    pub fn new(location: &Path) -> Self {
        let setup_dir = ContextDirectory::new(location);
        let vcs_dirs: Vec<MockContextVCSDirectory> = setup_dir
            .vcs_directories()
            .iter()
            .map(|d| MockContextVCSDirectory::new(d.location()))
            .collect();
        MockContextDirectory {
            dir: ContextDirectory::new(location),
            mocked_setup_component_public_keys_payload: None,
            mocked_election_event_context_payload: None,
            mocked_election_event_configuration: None,
            mocked_control_component_public_keys_payload: HashMap::new(),
            vcs_directories: vcs_dirs,
        }
    }

    impl_mock_methods_for_mocked_data!(
        setup_component_public_keys_payload,
        SetupComponentPublicKeysPayload
    );

    impl_mock_methods_for_mocked_data!(election_event_context_payload, ElectionEventContextPayload);

    impl_mock_methods_for_mocked_data!(election_event_configuration, ElectionEventConfiguration);

    impl_mock_methods_for_mocked_group!(
        control_component_public_keys_payload,
        ControlComponentPublicKeysPayload
    );
}

impl ContextDirectoryTrait for MockContextDirectory {
    type VCSDirType = MockContextVCSDirectory;

    fn setup_component_public_keys_payload_file(&self) -> &File<SetupComponentPublicKeysPayload> {
        self.dir.setup_component_public_keys_payload_file()
    }

    fn election_event_context_payload_file(&self) -> &File<ElectionEventContextPayload> {
        self.dir.election_event_context_payload_file()
    }

    fn election_event_configuration_file(&self) -> &File<ElectionEventConfiguration> {
        self.dir.election_event_configuration_file()
    }

    fn control_component_public_keys_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentPublicKeysPayload> {
        self.dir.control_component_public_keys_payload_group()
    }

    fn vcs_directories(&self) -> &[Self::VCSDirType] {
        &self.vcs_directories
    }

    impl_trait_get_method_for_mocked_data!(
        setup_component_public_keys_payload,
        SetupComponentPublicKeysPayload
    );

    impl_trait_get_method_for_mocked_data!(
        election_event_context_payload,
        ElectionEventContextPayload
    );

    impl_trait_get_method_for_mocked_data!(
        election_event_configuration,
        ElectionEventConfiguration
    );

    impl_trait_get_method_for_mocked_group!(
        control_component_public_keys_payload,
        ControlComponentPublicKeysPayload
    );

    fn location(&self) -> &std::path::Path {
        self.dir.location()
    }
}

impl ContextVCSDirectoryTrait for MockContextVCSDirectory {
    fn setup_component_tally_data_payload_file(&self) -> &File<SetupComponentTallyDataPayload> {
        todo!()
    }

    impl_trait_get_method_for_mocked_data!(
        setup_component_tally_data_payload,
        SetupComponentTallyDataPayload
    );

    fn name(&self) -> String {
        self.dir.name()
    }

    fn location(&self) -> &std::path::Path {
        self.dir.location()
    }
}

impl MockContextVCSDirectory {
    /// New [MockVCSDirectory]
    pub fn new(location: &Path) -> Self {
        MockContextVCSDirectory {
            dir: ContextVCSDirectory::new(location),
            mocked_setup_component_tally_data_payload: None,
        }
    }

    impl_mock_methods_for_mocked_data!(
        setup_component_tally_data_payload,
        SetupComponentTallyDataPayload
    );
}

#[cfg(test)]
mod test {
    use rust_ev_system_library::rust_ev_crypto_primitives::prelude::Integer;

    use super::*;
    use crate::config::test::test_datasets_context_path;

    #[test]
    fn test_mock_context() {
        let mut mock_dir =
            MockContextDirectory::new(test_datasets_context_path().as_path().parent().unwrap());
        mock_dir.mock_election_event_context_payload(|data| data.seed = "TOTO".to_string());
        assert_eq!(
            mock_dir
                .election_event_context_payload()
                .unwrap()
                .seed
                .as_str(),
            "TOTO"
        );
        mock_dir.mock_election_event_context_payload(|data| {
            data.encryption_group.set_p(&Integer::from(1u8))
        });
        assert_eq!(
            mock_dir
                .election_event_context_payload()
                .unwrap()
                .seed
                .as_str(),
            "TOTO"
        );
        assert_eq!(
            mock_dir
                .election_event_context_payload()
                .unwrap()
                .encryption_group
                .p(),
            &Integer::from(1u8)
        );
        mock_dir.mock_election_event_context_payload_reset();
        assert_ne!(
            mock_dir
                .election_event_context_payload()
                .unwrap()
                .seed
                .as_str(),
            "TOTO"
        );
        assert_ne!(
            mock_dir
                .election_event_context_payload()
                .unwrap()
                .encryption_group
                .p(),
            &Integer::from(1u8)
        );
    }

    #[test]
    fn test_mock_context_error() {
        let mut mock_dir =
            MockContextDirectory::new(test_datasets_context_path().as_path().parent().unwrap());
        mock_dir.mock_election_event_context_payload(|data| data.seed = "TOTO".to_string());
        assert_eq!(
            mock_dir
                .election_event_context_payload()
                .unwrap()
                .seed
                .as_str(),
            "TOTO"
        );
        mock_dir.mock_election_event_context_payload_error(FileStructureError::Mock(
            "test error".to_string(),
        ));
        assert!(mock_dir.election_event_context_payload().is_err());
        mock_dir.mock_election_event_context_payload_reset();
        assert_eq!(
            mock_dir.election_event_context_payload().unwrap().seed,
            mock_dir.dir.election_event_context_payload().unwrap().seed
        );
    }

    #[test]
    fn test_mock_group_context_delete() {
        let mut mock_dir =
            MockContextDirectory::new(test_datasets_context_path().as_path().parent().unwrap());
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            4
        );
        mock_dir.mock_control_component_public_keys_payload_as_deleted(2);
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            3
        );
        {
            let mut it = mock_dir.control_component_public_keys_payload_iter();
            assert_eq!(
                it.next()
                    .unwrap()
                    .1
                    .unwrap()
                    .control_component_public_keys
                    .node_id,
                1
            );
            assert_eq!(
                it.next()
                    .unwrap()
                    .1
                    .unwrap()
                    .control_component_public_keys
                    .node_id,
                3
            );
            assert_eq!(
                it.next()
                    .unwrap()
                    .1
                    .unwrap()
                    .control_component_public_keys
                    .node_id,
                4
            );
            assert!(it.next().is_none());
        }
        mock_dir.mock_control_component_public_keys_payload_reset(2);
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            4
        );
    }

    #[test]
    fn test_mock_group_context_error() {
        let mut mock_dir =
            MockContextDirectory::new(test_datasets_context_path().as_path().parent().unwrap());
        mock_dir.mock_control_component_public_keys_payload_error(
            2,
            FileStructureError::Mock("Test".to_string()),
        );
        let mut it = mock_dir.control_component_public_keys_payload_iter();
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            4
        );
        assert_eq!(
            it.next()
                .unwrap()
                .1
                .unwrap()
                .control_component_public_keys
                .node_id,
            1
        );
        assert!(it.next().unwrap().1.is_err());
        assert_eq!(
            it.next()
                .unwrap()
                .1
                .unwrap()
                .control_component_public_keys
                .node_id,
            3
        );
        assert_eq!(
            it.next()
                .unwrap()
                .1
                .unwrap()
                .control_component_public_keys
                .node_id,
            4
        );
        assert!(it.next().is_none());
    }

    #[test]
    fn test_mock_group_context() {
        let mut mock_dir =
            MockContextDirectory::new(test_datasets_context_path().as_path().parent().unwrap());
        mock_dir.mock_control_component_public_keys_payload(2, |data| {
            data.control_component_public_keys.node_id = 10
        });
        let mut it = mock_dir.control_component_public_keys_payload_iter();
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            4
        );
        assert_eq!(
            it.next()
                .unwrap()
                .1
                .unwrap()
                .control_component_public_keys
                .node_id,
            1
        );
        assert_eq!(
            it.next()
                .unwrap()
                .1
                .unwrap()
                .control_component_public_keys
                .node_id,
            10
        );
        assert_eq!(
            it.next()
                .unwrap()
                .1
                .unwrap()
                .control_component_public_keys
                .node_id,
            3
        );
        assert_eq!(
            it.next()
                .unwrap()
                .1
                .unwrap()
                .control_component_public_keys
                .node_id,
            4
        );
        assert!(it.next().is_none());
    }
}
