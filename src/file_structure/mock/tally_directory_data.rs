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
        ControlComponentBallotBoxPayload, ControlComponentShufflePayload,
        TallyComponentShufflePayload,
        tally::{ech_0222::ECH0222, tally_component_votes_payload::TallyComponentVotesPayload},
    },
    file_structure::{
        CompletnessTestTrait, FileStructureError, FileStructureErrorImpl, TallyDirectoryTrait,
        file::File,
        file_group::FileGroup,
        tally_directory::{
            BBDirectory, BBDirectoryTrait, TallyDirectory, impl_completness_test_trait_for_tally,
            impl_completness_test_trait_for_tally_bb,
        },
    },
};
use paste::paste;
use std::{collections::HashMap, path::Path, sync::Arc};

/// Mock for [BBDirectory]
pub struct MockBBDirectory {
    dir: BBDirectory,
    mocked_tally_component_votes_payload: Option<Box<MockedDataType<TallyComponentVotesPayload>>>,
    mocked_tally_component_shuffle_payload:
        Option<Box<MockedDataType<TallyComponentShufflePayload>>>,
    mocked_control_component_ballot_box_payload:
        HashMap<usize, Box<MockFileGroupElement<ControlComponentBallotBoxPayload>>>,
    mocked_control_component_shuffle_payload:
        HashMap<usize, Box<MockFileGroupElement<ControlComponentShufflePayload>>>,
}

/// Mock for [TallyDirectory]
pub struct MockTallyDirectory {
    dir: TallyDirectory,
    bb_directories: Vec<MockBBDirectory>,
    mocked_ech_0222: Option<Box<MockedDataType<ECH0222>>>,
}

impl_completness_test_trait_for_tally_bb!(MockBBDirectory);
impl_completness_test_trait_for_tally!(MockTallyDirectory);

impl MockTallyDirectory {
    pub fn new(data_location: &Path) -> Self {
        let tally_dir = TallyDirectory::new(data_location);
        let bb_dirs: Vec<MockBBDirectory> = tally_dir
            .bb_directories()
            .iter()
            .map(|d| MockBBDirectory::new(d.location()))
            .collect();
        MockTallyDirectory {
            dir: tally_dir,
            bb_directories: bb_dirs,
            mocked_ech_0222: None,
        }
    }

    pub fn bb_directories_mut(&mut self) -> &mut [MockBBDirectory] {
        &mut self.bb_directories
    }
}

impl TallyDirectoryTrait for MockTallyDirectory {
    type BBDirType = MockBBDirectory;

    fn ech_0222_file(&self) -> &File<ECH0222> {
        self.dir.ech_0222_file()
    }

    impl_trait_get_method_for_mocked_data!(ech_0222, ECH0222);

    fn bb_directories(&self) -> &[Self::BBDirType] {
        &self.bb_directories
    }

    fn location(&self) -> &Path {
        self.dir.location()
    }
}

impl MockBBDirectory {
    pub fn new(location: &Path) -> Self {
        MockBBDirectory {
            dir: BBDirectory::new(location),
            mocked_tally_component_votes_payload: None,
            mocked_tally_component_shuffle_payload: None,
            mocked_control_component_ballot_box_payload: HashMap::new(),
            mocked_control_component_shuffle_payload: HashMap::new(),
        }
    }

    impl_mock_methods_for_mocked_data!(tally_component_votes_payload, TallyComponentVotesPayload);

    impl_mock_methods_for_mocked_data!(
        tally_component_shuffle_payload,
        TallyComponentShufflePayload
    );

    impl_mock_methods_for_mocked_group!(
        control_component_ballot_box_payload,
        ControlComponentBallotBoxPayload
    );

    impl_mock_methods_for_mocked_group!(
        control_component_shuffle_payload,
        ControlComponentShufflePayload
    );
}

//impl_itertor_for_mocked_group_type!(ControlComponentBallotBoxPayload);
//impl_itertor_for_mocked_group_type!(ControlComponentShufflePayload);

impl BBDirectoryTrait for MockBBDirectory {
    fn tally_component_votes_payload_file(&self) -> &File<TallyComponentVotesPayload> {
        self.dir.tally_component_votes_payload_file()
    }

    fn tally_component_shuffle_payload_file(&self) -> &File<TallyComponentShufflePayload> {
        self.dir.tally_component_shuffle_payload_file()
    }

    fn control_component_ballot_box_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentBallotBoxPayload> {
        self.dir.control_component_ballot_box_payload_group()
    }

    fn control_component_shuffle_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentShufflePayload> {
        self.dir.control_component_shuffle_payload_group()
    }

    impl_trait_get_method_for_mocked_data!(
        tally_component_votes_payload,
        TallyComponentVotesPayload
    );

    impl_trait_get_method_for_mocked_data!(
        tally_component_shuffle_payload,
        TallyComponentShufflePayload
    );

    impl_trait_get_method_for_mocked_group!(
        control_component_ballot_box_payload,
        ControlComponentBallotBoxPayload
    );

    impl_trait_get_method_for_mocked_group!(
        control_component_shuffle_payload,
        ControlComponentShufflePayload
    );

    fn name(&self) -> String {
        self.dir.name()
    }

    fn location(&self) -> &Path {
        self.dir.location()
    }
}
