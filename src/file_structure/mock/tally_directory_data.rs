use super::{
    impl_itertor_for_mocked_group_type, impl_mock_methods_for_mocked_data,
    impl_mock_methods_for_mocked_group, impl_trait_get_method_for_mocked_data,
    impl_trait_get_method_for_mocked_group, MockFileGroupIter,
};
use crate::{
    data_structures::{
        tally::tally_component_votes_payload::TallyComponentVotesPayload,
        ControlComponentBallotBoxPayload, ControlComponentShufflePayload,
        TallyComponentShufflePayload,
    },
    file_structure::{
        file::File,
        file_group::{FileGroup, FileGroupIterTrait},
        tally_directory::{
            impl_completness_test_trait_for_tally, impl_completness_test_trait_for_tally_bb,
            BBDirectory, BBDirectoryTrait, ControlComponentBallotBoxPayloadAsResultIter,
            ControlComponentShufflePayloadAsResultIter, TallyDirectory,
        },
        CompletnessTestTrait, FileStructureError, TallyDirectoryTrait,
    },
};
use paste::paste;
use std::{collections::HashMap, path::Path};

/// Mock for [BBDirectory]
pub struct MockBBDirectory {
    dir: BBDirectory,
    mocked_tally_component_votes_payload: Option<Box<TallyComponentVotesPayload>>,
    mocked_tally_component_votes_payload_error: Option<FileStructureError>,
    mocked_tally_component_shuffle_payload: Option<Box<TallyComponentShufflePayload>>,
    mocked_tally_component_shuffle_payload_error: Option<FileStructureError>,
    mocked_control_component_ballot_box_payload:
        HashMap<usize, Box<ControlComponentBallotBoxPayload>>,
    mocked_control_component_ballot_box_payload_deleted: Vec<usize>,
    mocked_control_component_ballot_box_payload_errors: HashMap<usize, String>,
    mocked_control_component_shuffle_payload: HashMap<usize, Box<ControlComponentShufflePayload>>,
    mocked_control_component_shuffle_payload_deleted: Vec<usize>,
    mocked_control_component_shuffle_payload_errors: HashMap<usize, String>,
}

/// Mock for [TallyDirectory]
pub struct MockTallyDirectory {
    dir: TallyDirectory,
    bb_directories: Vec<MockBBDirectory>,
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
        }
    }
}

impl TallyDirectoryTrait for MockTallyDirectory {
    type BBDirType = MockBBDirectory;

    fn e_voting_decrypt_file(&self) -> &File {
        self.dir.e_voting_decrypt_file()
    }

    fn ech_0110_file(&self) -> &File {
        self.dir.ech_0110_file()
    }

    fn ech_0222_file(&self) -> &File {
        self.dir.ech_0222_file()
    }

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
            mocked_tally_component_votes_payload_error: None,
            mocked_tally_component_shuffle_payload: None,
            mocked_tally_component_shuffle_payload_error: None,
            mocked_control_component_ballot_box_payload: HashMap::new(),
            mocked_control_component_ballot_box_payload_deleted: vec![],
            mocked_control_component_ballot_box_payload_errors: HashMap::new(),
            mocked_control_component_shuffle_payload: HashMap::new(),
            mocked_control_component_shuffle_payload_deleted: vec![],
            mocked_control_component_shuffle_payload_errors: HashMap::new(),
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

impl_itertor_for_mocked_group_type!(ControlComponentBallotBoxPayload);
impl_itertor_for_mocked_group_type!(ControlComponentShufflePayload);

impl BBDirectoryTrait for MockBBDirectory {
    type ControlComponentBallotBoxPayloadAsResultIterType =
        MockControlComponentBallotBoxPayloadAsResultIter;
    type ControlComponentShufflePayloadAsResultIterType =
        MockControlComponentShufflePayloadAsResultIter;

    fn tally_component_votes_payload_file(&self) -> &File {
        self.dir.tally_component_votes_payload_file()
    }

    fn tally_component_shuffle_payload_file(&self) -> &File {
        self.dir.tally_component_shuffle_payload_file()
    }

    fn control_component_ballot_box_payload_group(&self) -> &FileGroup {
        self.dir.control_component_ballot_box_payload_group()
    }

    fn control_component_shuffle_payload_group(&self) -> &FileGroup {
        self.dir.control_component_ballot_box_payload_group()
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
