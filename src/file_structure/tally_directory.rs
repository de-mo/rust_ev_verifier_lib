use super::{
    file::{create_file, File},
    file_group::FileGroup,
};
use crate::{
    constants::{BB_DIR_NAME, TALLY_DIR_NAME},
    data_structures::{
        create_verifier_tally_data_type, tally::VerifierTallyDataType, VerifierDataType,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct TallyDirectory {
    location: PathBuf,
    e_voting_decrypt_file: File,
    ech_0110_file: File,
    ech_0222_file: File,
    bb_directories: Vec<BBDirectory>,
}

#[derive(Clone)]
pub struct BBDirectory {
    location: PathBuf,
    tally_component_votes_payload_file: File,
    tally_component_shuffle_payload_file: File,
    control_component_ballot_box_payload_group: FileGroup,
}

/// Trait to set the necessary functions for the struct [TallyDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait TallyDirectoryTrait {
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
pub trait BBDirectoryTrait {
    fn tally_component_votes_payload_file(&self) -> &File;
    fn tally_component_shuffle_payload_file(&self) -> &File;
    fn control_component_ballot_box_payload_group(&self) -> &FileGroup;
    fn get_name(&self) -> String;
}

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
    fn tally_component_votes_payload_file(&self) -> &File {
        &self.tally_component_votes_payload_file
    }
    fn tally_component_shuffle_payload_file(&self) -> &File {
        &self.tally_component_shuffle_payload_file
    }
    fn control_component_ballot_box_payload_group(&self) -> &FileGroup {
        &self.control_component_ballot_box_payload_group
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
    pub fn new(data_location: &Path) -> TallyDirectory {
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

    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

impl BBDirectory {
    pub fn new(location: &Path) -> Self {
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
                &location,
                create_verifier_tally_data_type!(Tally, ControlComponentBallotBoxPayload),
            ),
        }
    }

    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

#[cfg(any(test, doc))]
pub mod mock {
    //! Module defining mocking structure for [VCSDirectory] and [SetupDirectory]
    //!
    //! The mocks read the correct data from the file. It is possible to change any data
    //! with the functions mock_
    use super::{super::mock::wrap_file_group_getter, *};

    /// Mock for [BBDirectory]
    pub struct MockBBDirectory {
        dir: BBDirectory,
        mocked_tally_component_votes_payload_file: Option<File>,
        mocked_tally_component_shuffle_payload_file: Option<File>,
        mocked_control_component_ballot_box_payload_group: Option<FileGroup>,
        mocked_get_name: Option<String>,
    }

    /// Mock for [TallyDirectory]
    pub struct MockTallyDirectory {
        dir: TallyDirectory,
        mocked_e_voting_decrypt_file: Option<File>,
        mocked_ech_0110_file: Option<File>,
        mocked_ech_0222_file: Option<File>,
        bb_directories: Vec<MockBBDirectory>,
    }

    impl BBDirectoryTrait for MockBBDirectory {
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
        pub fn new(location: &Path) -> Self {
            MockBBDirectory {
                dir: BBDirectory::new(location),
                mocked_tally_component_shuffle_payload_file: None,
                mocked_tally_component_votes_payload_file: None,
                mocked_control_component_ballot_box_payload_group: None,
                mocked_get_name: None,
            }
        }
        pub fn mock_tally_component_shuffle_payload_file(&mut self, data: &File) {
            self.mocked_tally_component_shuffle_payload_file = Some(data.clone());
        }
        pub fn mock_tally_component_votes_payload_file(&mut self, data: &File) {
            self.mocked_tally_component_votes_payload_file = Some(data.clone());
        }
        pub fn mock_control_component_ballot_box_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_ballot_box_payload_group = Some(data.clone());
        }
        pub fn mock_get_name(&mut self, data: &str) {
            self.mocked_get_name = Some(data.to_string())
        }
    }

    impl MockTallyDirectory {
        pub fn new(data_location: &Path) -> Self {
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
        pub fn bb_directories_mut(&mut self) -> Vec<&mut MockBBDirectory> {
            self.bb_directories.iter_mut().collect()
        }

        pub fn mock_e_voting_decrypt_file(&mut self, data: &File) {
            self.mocked_e_voting_decrypt_file = Some(data.clone());
        }
        pub fn mock_ech_0110_file(&mut self, data: &File) {
            self.mocked_ech_0110_file = Some(data.clone());
        }
        pub fn mock_ech_0222_file(&mut self, data: &File) {
            self.mocked_ech_0222_file = Some(data.clone());
        }
    }
}
