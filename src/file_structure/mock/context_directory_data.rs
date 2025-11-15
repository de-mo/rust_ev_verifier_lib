// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use super::{
    FileGroupFileIter, MockFileGroupDataIter, MockFileGroupElement, MockedDataType,
    impl_mock_methods_for_mocked_data, impl_mock_methods_for_mocked_group,
    impl_trait_get_method_for_mocked_data, impl_trait_get_method_for_mocked_group,
};
use crate::{
    data_structures::{
        ElectionEventContextPayload,
        context::{
            control_component_public_keys_payload::ControlComponentPublicKeysPayload,
            election_event_configuration::{
                ElectionEventConfiguration, ElectionEventConfigurationData,
            },
            setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
            setup_component_tally_data_payload::SetupComponentTallyDataPayload,
        },
        mock::MockXmlTrait,
    },
    file_structure::{
        CompletnessTestTrait, ContextDirectory, ContextDirectoryTrait, FileStructureError,
        FileStructureErrorImpl,
        context_directory::{ContextVCSDirectory, ContextVCSDirectoryTrait},
        file::File,
        file_group::FileGroup,
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

impl CompletnessTestTrait for MockContextDirectory {
    fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
        self.dir.test_completness()
    }
}

impl CompletnessTestTrait for MockContextVCSDirectory {
    fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
        self.dir.test_completness()
    }
}

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

    #[allow(dead_code)]
    /// Mock ElectionEventConfiguration data
    pub fn mock_election_event_configuration_data(
        &mut self,
        closure: impl FnMut(&mut ElectionEventConfigurationData) + Clone,
    ) {
        self.mock_election_event_configuration(|d| d.set_data(closure.clone()));
    }

    #[allow(dead_code)]
    /// Mock ElectionEventConfiguration raw data (string)
    pub fn mock_election_event_configuration_string(&mut self, new_str: String) {
        self.mock_election_event_configuration(|d| d.set_raw(new_str.clone()));
    }

    impl_mock_methods_for_mocked_group!(
        control_component_public_keys_payload,
        ControlComponentPublicKeysPayload
    );

    pub fn vcs_directories_mut(&mut self) -> &mut [MockContextVCSDirectory] {
        &mut self.vcs_directories
    }

    pub fn vcs_directory_mut(&mut self, name: &str) -> Option<&mut MockContextVCSDirectory> {
        self.vcs_directories
            .iter_mut()
            .find(|d| d.name().as_str() == name)
    }
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
    use crate::{config::test::test_datasets_context_path, consts::NUMBER_CONTROL_COMPONENTS};

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
        mock_dir.mock_election_event_context_payload_error(FileStructureError::from(
            FileStructureErrorImpl::Mock("test error".to_string()),
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
            NUMBER_CONTROL_COMPONENTS
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
                NUMBER_CONTROL_COMPONENTS
            );
            assert!(it.next().is_none());
        }
        mock_dir.mock_control_component_public_keys_payload_reset(2);
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            NUMBER_CONTROL_COMPONENTS
        );
    }

    #[test]
    fn test_mock_group_context_error() {
        let mut mock_dir =
            MockContextDirectory::new(test_datasets_context_path().as_path().parent().unwrap());
        mock_dir.mock_control_component_public_keys_payload_error(
            2,
            FileStructureError::from(FileStructureErrorImpl::Mock("Test".to_string())),
        );
        let mut it = mock_dir.control_component_public_keys_payload_iter();
        assert_eq!(
            mock_dir
                .control_component_public_keys_payload_iter()
                .count(),
            NUMBER_CONTROL_COMPONENTS
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
            NUMBER_CONTROL_COMPONENTS
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
            NUMBER_CONTROL_COMPONENTS
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
            NUMBER_CONTROL_COMPONENTS
        );
        assert!(it.next().is_none());
    }

    #[test]
    fn test_mock_config() {
        let mut mock_dir =
            MockContextDirectory::new(test_datasets_context_path().as_path().parent().unwrap());
        mock_dir.mock_election_event_configuration_data(|d| d.header.voter_total = 10000);
        assert_eq!(
            mock_dir
                .election_event_configuration()
                .unwrap()
                .get_data()
                .unwrap()
                .header
                .voter_total,
            10000
        );
    }
}
