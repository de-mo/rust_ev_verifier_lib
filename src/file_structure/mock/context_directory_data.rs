use super::{
    impl_itertor_for_mocked_group_type, impl_mock_methods_for_mocked_data,
    impl_mock_methods_for_mocked_group, impl_trait_get_method_for_mocked_data,
    impl_trait_get_method_for_mocked_group, MockFileGroupIter,
};
use crate::{
    data_structures::{
        ControlComponentPublicKeysPayload, ElectionEventConfiguration, ElectionEventContextPayload,
        SetupComponentPublicKeysPayload, SetupComponentTallyDataPayload,
    },
    file_structure::{
        context_directory::{
            impl_completness_test_trait_for_context, impl_completness_test_trait_for_context_vcs,
            ContextVCSDirectory, ContextVCSDirectoryTrait,
            ControlComponentPublicKeysPayloadAsResultIter,
        },
        file::File,
        file_group::{FileGroup, FileGroupIterTrait},
        CompletnessTestTrait, ContextDirectory, ContextDirectoryTrait, FileStructureError,
    },
};
use paste::paste;
use std::{collections::HashMap, path::Path};

/// Mock for [ContextVCSDirectory]
pub struct MockContextVCSDirectory {
    dir: ContextVCSDirectory,
    mocked_setup_component_tally_data_payload: Option<Box<SetupComponentTallyDataPayload>>,
    mocked_setup_component_tally_data_payload_error: Option<FileStructureError>,
}

/// Mock for [ContextDirectory]
pub struct MockContextDirectory {
    dir: ContextDirectory,
    mocked_setup_component_public_keys_payload: Option<Box<SetupComponentPublicKeysPayload>>,
    mocked_setup_component_public_keys_payload_error: Option<FileStructureError>,
    mocked_election_event_context_payload: Option<Box<ElectionEventContextPayload>>,
    mocked_election_event_context_payload_error: Option<FileStructureError>,
    mocked_election_event_configuration: Option<Box<ElectionEventConfiguration>>,
    mocked_election_event_configuration_error: Option<FileStructureError>,
    mocked_control_component_public_keys_payload:
        HashMap<usize, Box<ControlComponentPublicKeysPayload>>,
    mocked_control_component_public_keys_payload_deleted: Vec<usize>,
    mocked_control_component_public_keys_payload_errors: HashMap<usize, String>,
    vcs_directories: Vec<MockContextVCSDirectory>,
}

impl_completness_test_trait_for_context_vcs!(MockContextVCSDirectory);
impl_completness_test_trait_for_context!(MockContextDirectory);

impl_itertor_for_mocked_group_type!(ControlComponentPublicKeysPayload);

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
            mocked_setup_component_public_keys_payload_error: None,
            mocked_election_event_context_payload: None,
            mocked_election_event_context_payload_error: None,
            mocked_election_event_configuration: None,
            mocked_election_event_configuration_error: None,
            mocked_control_component_public_keys_payload: HashMap::new(),
            mocked_control_component_public_keys_payload_deleted: Vec::new(),
            mocked_control_component_public_keys_payload_errors: HashMap::new(),
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

    type ControlComponentPublicKeysPayloadAsResultIterType =
        MockControlComponentPublicKeysPayloadAsResultIter;

    fn setup_component_public_keys_payload_file(&self) -> &File {
        self.dir.setup_component_public_keys_payload_file()
    }

    fn election_event_context_payload_file(&self) -> &File {
        self.dir.election_event_context_payload_file()
    }

    fn election_event_configuration_file(&self) -> &File {
        self.dir.election_event_configuration_file()
    }

    fn control_component_public_keys_payload_group(&self) -> &FileGroup {
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
    fn setup_component_tally_data_payload_file(&self) -> &File {
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
            mocked_setup_component_tally_data_payload_error: None,
        }
    }

    impl_mock_methods_for_mocked_data!(
        setup_component_tally_data_payload,
        SetupComponentTallyDataPayload
    );
}

#[cfg(test)]
mod test {
    use rust_ev_crypto_primitives::Integer;

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
        mock_dir.mock_election_event_context_payload_remove_error();
        assert_eq!(
            mock_dir
                .election_event_context_payload()
                .unwrap()
                .seed
                .as_str(),
            "TOTO"
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
            mock_dir.mocked_control_component_public_keys_payload_deleted,
            vec![2]
        );
        let mut it = mock_dir.control_component_public_keys_payload_iter();
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            3
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
        mock_dir.mock_control_component_public_keys_payload_remove_deleted(2);
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
