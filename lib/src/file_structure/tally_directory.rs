use super::{
    file::{create_file, File},
    file_group::{
        add_type_for_file_group_iter_trait, impl_iterator_over_data_payload, FileGroup,
        FileGroupIter, FileGroupIterTrait,
    },
};
use crate::{
    constants::{BB_DIR_NAME, TALLY_DIR_NAME},
    data_structures::{
        create_verifier_tally_data_type,
        tally::{
            control_component_ballot_box_payload::ControlComponentBallotBoxPayload,
            control_component_shuffle_payload::ControlComponentShufflePayload,
            tally_component_shuffle_payload::TallyComponentShufflePayload,
            tally_component_votes_payload::TallyComponentVotesPayload, VerifierTallyDataType,
        },
        VerifierDataType, VerifierTallyDataTrait,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub(crate) struct TallyDirectory {
    location: PathBuf,
    e_voting_decrypt_file: File,
    ech_0110_file: File,
    ech_0222_file: File,
    bb_directories: Vec<BBDirectory>,
}

#[derive(Clone)]
pub(crate) struct BBDirectory {
    location: PathBuf,
    tally_component_votes_payload_file: File,
    tally_component_shuffle_payload_file: File,
    control_component_ballot_box_payload_group: FileGroup,
    control_component_shuffle_payload_group: FileGroup,
}

/// Trait to set the necessary functions for the struct [TallyDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub(crate) trait TallyDirectoryTrait {
    type BBDirType: BBDirectoryTrait;

    fn e_voting_decrypt_file(&self) -> &File;
    fn ech_0110_file(&self) -> &File;
    fn ech_0222_file(&self) -> &File;
    fn bb_directories(&self) -> &Vec<Self::BBDirType>;
}

/// Trait to set the necessary functions for the struct [BBDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub(crate) trait BBDirectoryTrait {
    add_type_for_file_group_iter_trait!(
        ControlComponentBallotBoxPayloadAsResultIterType,
        ControlComponentBallotBoxPayloadAsResult
    );
    add_type_for_file_group_iter_trait!(
        ControlComponentShufflePayloadloadAsResultIterType,
        ControlComponentShufflePayloadloadAsResult
    );
    fn tally_component_votes_payload_file(&self) -> &File;
    fn tally_component_shuffle_payload_file(&self) -> &File;
    fn control_component_ballot_box_payload_group(&self) -> &FileGroup;
    fn control_component_shuffle_payload_group(&self) -> &FileGroup;
    fn tally_component_votes_payload(&self) -> anyhow::Result<Box<TallyComponentVotesPayload>>;
    fn tally_component_shuffle_payload(&self) -> anyhow::Result<Box<TallyComponentShufflePayload>>;
    fn control_component_ballot_box_payload_iter(
        &self,
    ) -> Self::ControlComponentBallotBoxPayloadAsResultIterType;
    fn control_component_shuffle_payload_iter(
        &self,
    ) -> Self::ControlComponentShufflePayloadloadAsResultIterType;

    fn get_name(&self) -> String;
}

impl_iterator_over_data_payload!(
    ControlComponentBallotBoxPayload,
    control_component_ballot_box_payload,
    ControlComponentBallotBoxPayloadAsResult,
    ControlComponentBallotBoxPayloadAsResultIter
);

impl_iterator_over_data_payload!(
    ControlComponentShufflePayload,
    control_component_shuffle_payload,
    ControlComponentShufflePayloadloadAsResult,
    ControlComponentShufflePayloadAsResultIter
);

impl TallyDirectoryTrait for TallyDirectory {
    type BBDirType = BBDirectory;

    fn e_voting_decrypt_file(&self) -> &File {
        &self.e_voting_decrypt_file
    }
    fn ech_0110_file(&self) -> &File {
        &self.ech_0110_file
    }
    fn ech_0222_file(&self) -> &File {
        &self.ech_0222_file
    }
    fn bb_directories(&self) -> &Vec<BBDirectory> {
        &self.bb_directories
    }
}

impl BBDirectoryTrait for BBDirectory {
    type ControlComponentBallotBoxPayloadAsResultIterType =
        ControlComponentBallotBoxPayloadAsResultIter;
    type ControlComponentShufflePayloadloadAsResultIterType =
        ControlComponentShufflePayloadAsResultIter;
    fn tally_component_votes_payload_file(&self) -> &File {
        &self.tally_component_votes_payload_file
    }
    fn tally_component_shuffle_payload_file(&self) -> &File {
        &self.tally_component_shuffle_payload_file
    }
    fn control_component_ballot_box_payload_group(&self) -> &FileGroup {
        &self.control_component_ballot_box_payload_group
    }
    fn control_component_shuffle_payload_group(&self) -> &FileGroup {
        &self.control_component_shuffle_payload_group
    }
    fn tally_component_votes_payload(&self) -> anyhow::Result<Box<TallyComponentVotesPayload>> {
        self.tally_component_votes_payload_file
            .get_data()
            .map_err(|e| e.context("in tally_component_votes_payload"))
            .map(|d| Box::new(d.tally_component_votes_payload().unwrap().clone()))
    }
    fn tally_component_shuffle_payload(&self) -> anyhow::Result<Box<TallyComponentShufflePayload>> {
        self.tally_component_shuffle_payload_file
            .get_data()
            .map_err(|e| e.context("in tally_component_shuffle_payload"))
            .map(|d| Box::new(d.tally_component_shuffle_payload().unwrap().clone()))
    }

    fn control_component_ballot_box_payload_iter(
        &self,
    ) -> Self::ControlComponentBallotBoxPayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_ballot_box_payload_group)
    }

    fn control_component_shuffle_payload_iter(
        &self,
    ) -> Self::ControlComponentShufflePayloadloadAsResultIterType {
        FileGroupIter::new(&self.control_component_shuffle_payload_group)
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

impl TallyDirectory {
    #[allow(clippy::redundant_clone)]
    pub(crate) fn new(data_location: &Path) -> TallyDirectory {
        let location = data_location.join(TALLY_DIR_NAME);
        let mut res = TallyDirectory {
            location: location.to_path_buf(),
            e_voting_decrypt_file: create_file!(
                location,
                Tally,
                VerifierTallyDataType::EVotingDecrypt
            ),
            ech_0110_file: create_file!(location, Tally, VerifierTallyDataType::ECH0110),
            ech_0222_file: create_file!(location, Tally, VerifierTallyDataType::ECH0222),
            bb_directories: vec![],
        };
        let bb_path = location.join(BB_DIR_NAME);
        if bb_path.is_dir() {
            for re in fs::read_dir(&bb_path).unwrap() {
                let e = re.unwrap().path();
                if e.is_dir() {
                    res.bb_directories.push(BBDirectory::new(&e))
                }
            }
        }
        res
    }

    #[allow(dead_code)]
    pub(crate) fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

impl BBDirectory {
    pub(crate) fn new(location: &Path) -> Self {
        Self {
            location: location.to_path_buf(),
            tally_component_votes_payload_file: create_file!(
                location,
                Tally,
                VerifierTallyDataType::TallyComponentVotesPayload
            ),
            tally_component_shuffle_payload_file: create_file!(
                location,
                Tally,
                VerifierTallyDataType::TallyComponentShufflePayload
            ),
            control_component_ballot_box_payload_group: FileGroup::new(
                location,
                create_verifier_tally_data_type!(Tally, ControlComponentBallotBoxPayload),
            ),
            control_component_shuffle_payload_group: FileGroup::new(
                location,
                create_verifier_tally_data_type!(Tally, ControlComponentShufflePayload),
            ),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

#[cfg(any(test, doc))]
#[allow(dead_code)]
pub(crate) mod mock {
    //! Module defining mocking structure for [VCSDirectory] and [SetupDirectory]
    //!
    //! The mocks read the correct data from the file. It is possible to change any data
    //! with the functions mock_
    use super::super::file_group::mock::MockFileGroupIter;
    use super::{super::mock::wrap_file_group_getter, *};
    use crate::file_structure::file_group::mock::{
        impl_iterator_over_data_payload_mock, wrap_payload_iter,
    };
    use crate::file_structure::mock::wrap_payload_getter;
    use anyhow::anyhow;
    use std::collections::HashMap;

    /// Mock for [BBDirectory]
    pub(crate) struct MockBBDirectory {
        dir: BBDirectory,
        mocked_tally_component_votes_payload_file: Option<File>,
        mocked_tally_component_shuffle_payload_file: Option<File>,
        mocked_control_component_ballot_box_payload_group: Option<FileGroup>,
        mocked_control_component_shuffle_payload_group: Option<FileGroup>,
        mocked_tally_component_votes_payload:
            Option<anyhow::Result<Box<TallyComponentVotesPayload>>>,
        mocked_tally_component_shuffle_payload:
            Option<anyhow::Result<Box<TallyComponentShufflePayload>>>,
        mocked_control_component_ballot_box_payloads:
            HashMap<usize, ControlComponentBallotBoxPayloadAsResult>,
        mocked_control_component_shuffle_payloads:
            HashMap<usize, ControlComponentShufflePayloadloadAsResult>,

        mocked_get_name: Option<String>,
    }

    /// Mock for [TallyDirectory]
    pub(crate) struct MockTallyDirectory {
        dir: TallyDirectory,
        mocked_e_voting_decrypt_file: Option<File>,
        mocked_ech_0110_file: Option<File>,
        mocked_ech_0222_file: Option<File>,
        bb_directories: Vec<MockBBDirectory>,
    }

    impl_iterator_over_data_payload_mock!(
        ControlComponentBallotBoxPayload,
        ControlComponentBallotBoxPayloadAsResult,
        ControlComponentBallotBoxPayloadAsResultIter,
        MockControlComponentBallotBoxPayloadAsResultIter
    );

    impl_iterator_over_data_payload_mock!(
        ControlComponentShufflePayload,
        ControlComponentShufflePayloadAsResult,
        ControlComponentShufflePayloadAsResultIter,
        MockControlComponentShufflePayloadAsResultIter
    );

    impl BBDirectoryTrait for MockBBDirectory {
        type ControlComponentBallotBoxPayloadAsResultIterType =
            MockControlComponentBallotBoxPayloadAsResultIter;
        type ControlComponentShufflePayloadloadAsResultIterType =
            MockControlComponentShufflePayloadAsResultIter;
        wrap_file_group_getter!(
            tally_component_votes_payload_file,
            mocked_tally_component_votes_payload_file,
            File
        );
        wrap_file_group_getter!(
            tally_component_shuffle_payload_file,
            mocked_tally_component_shuffle_payload_file,
            File
        );
        wrap_file_group_getter!(
            control_component_ballot_box_payload_group,
            mocked_control_component_ballot_box_payload_group,
            FileGroup
        );
        wrap_file_group_getter!(
            control_component_shuffle_payload_group,
            mocked_control_component_shuffle_payload_group,
            FileGroup
        );
        wrap_payload_getter!(
            tally_component_votes_payload,
            mocked_tally_component_votes_payload,
            TallyComponentVotesPayload
        );
        wrap_payload_getter!(
            tally_component_shuffle_payload,
            mocked_tally_component_shuffle_payload,
            TallyComponentShufflePayload
        );
        wrap_payload_iter!(
            control_component_ballot_box_payload_iter,
            ControlComponentBallotBoxPayloadAsResultIterType,
            MockControlComponentBallotBoxPayloadAsResultIter,
            mocked_control_component_ballot_box_payloads
        );
        wrap_payload_iter!(
            control_component_shuffle_payload_iter,
            ControlComponentShufflePayloadloadAsResultIterType,
            MockControlComponentShufflePayloadAsResultIter,
            mocked_control_component_shuffle_payloads
        );

        fn get_name(&self) -> String {
            match &self.mocked_get_name {
                Some(e) => e.clone(),
                None => self.dir.get_name(),
            }
        }
    }

    impl TallyDirectoryTrait for MockTallyDirectory {
        type BBDirType = MockBBDirectory;
        wrap_file_group_getter!(e_voting_decrypt_file, mocked_e_voting_decrypt_file, File);
        wrap_file_group_getter!(ech_0110_file, mocked_ech_0110_file, File);
        wrap_file_group_getter!(ech_0222_file, mocked_ech_0222_file, File);

        fn bb_directories(&self) -> &Vec<MockBBDirectory> {
            &self.bb_directories
        }
    }

    impl MockBBDirectory {
        pub(crate) fn new(location: &Path) -> Self {
            MockBBDirectory {
                dir: BBDirectory::new(location),
                mocked_tally_component_shuffle_payload_file: None,
                mocked_tally_component_votes_payload_file: None,
                mocked_control_component_ballot_box_payload_group: None,
                mocked_control_component_shuffle_payload_group: None,
                mocked_tally_component_votes_payload: None,
                mocked_tally_component_shuffle_payload: None,
                mocked_control_component_ballot_box_payloads: HashMap::new(),
                mocked_control_component_shuffle_payloads: HashMap::new(),
                mocked_get_name: None,
            }
        }
        pub(crate) fn mock_tally_component_shuffle_payload_file(&mut self, data: &File) {
            self.mocked_tally_component_shuffle_payload_file = Some(data.clone());
        }
        pub(crate) fn mock_tally_component_votes_payload_file(&mut self, data: &File) {
            self.mocked_tally_component_votes_payload_file = Some(data.clone());
        }
        pub(crate) fn mock_control_component_ballot_box_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_ballot_box_payload_group = Some(data.clone());
        }
        pub(crate) fn mock_control_component_shuffle_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_shuffle_payload_group = Some(data.clone());
        }
        pub(crate) fn mock_get_name(&mut self, data: &str) {
            self.mocked_get_name = Some(data.to_string())
        }
    }

    impl MockTallyDirectory {
        pub(crate) fn new(data_location: &Path) -> Self {
            let tally_dir = TallyDirectory::new(data_location);
            let bb_dirs: Vec<MockBBDirectory> = tally_dir
                .bb_directories
                .iter()
                .map(|d| MockBBDirectory::new(&d.location))
                .collect();
            MockTallyDirectory {
                dir: tally_dir,
                mocked_e_voting_decrypt_file: None,
                mocked_ech_0110_file: None,
                mocked_ech_0222_file: None,
                bb_directories: bb_dirs,
            }
        }
        pub(crate) fn bb_directories_mut(&mut self) -> Vec<&mut MockBBDirectory> {
            self.bb_directories.iter_mut().collect()
        }

        pub(crate) fn mock_e_voting_decrypt_file(&mut self, data: &File) {
            self.mocked_e_voting_decrypt_file = Some(data.clone());
        }
        pub(crate) fn mock_ech_0110_file(&mut self, data: &File) {
            self.mocked_ech_0110_file = Some(data.clone());
        }
        pub(crate) fn mock_ech_0222_file(&mut self, data: &File) {
            self.mocked_ech_0222_file = Some(data.clone());
        }
    }
}
