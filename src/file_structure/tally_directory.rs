use super::file::{create_file, File};
use crate::{
    constants::{BB_DIR_NAME, TALLY_DIR_NAME},
    data_structures::{tally::VerifierTallyDataType, VerifierDataType},
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
    use super::*;

    /// Mock for [BBDirectory]
    pub struct MockBBDirectory {
        dir: BBDirectory,
        mock_tally_component_votes_payload_file: Option<File>,
        mock_tally_component_shuffle_payload_file: Option<File>,
        mock_get_name: Option<String>,
    }

    /// Mock for [TallyDirectory]
    pub struct MockTallyDirectory {
        dir: TallyDirectory,
        mock_e_voting_decrypt_file: Option<File>,
        mock_ech_0110_file: Option<File>,
        mock_ech_0222_file: Option<File>,
        bb_directories: Vec<MockBBDirectory>,
    }

    impl BBDirectoryTrait for MockBBDirectory {
        fn tally_component_votes_payload_file(&self) -> &File {
            match &self.mock_tally_component_votes_payload_file {
                Some(e) => e,
                None => self.dir.tally_component_votes_payload_file(),
            }
        }

        fn tally_component_shuffle_payload_file(&self) -> &File {
            match &self.mock_tally_component_shuffle_payload_file {
                Some(e) => e,
                None => self.dir.tally_component_shuffle_payload_file(),
            }
        }
        fn get_name(&self) -> String {
            match &self.mock_get_name {
                Some(e) => e.clone(),
                None => self.dir.get_name(),
            }
        }
    }

    impl TallyDirectoryTrait for MockTallyDirectory {
        type BBDirType = MockBBDirectory;

        fn e_voting_decrypt_file(&self) -> &File {
            match &self.mock_e_voting_decrypt_file {
                Some(e) => e,
                None => self.dir.e_voting_decrypt_file(),
            }
        }

        fn ech_0110_file(&self) -> &File {
            match &self.mock_ech_0110_file {
                Some(e) => e,
                None => self.dir.ech_0110_file(),
            }
        }

        fn ech_0222_file(&self) -> &File {
            match &self.mock_ech_0222_file {
                Some(e) => e,
                None => self.dir.ech_0222_file(),
            }
        }

        fn bb_directories(&self) -> &Vec<MockBBDirectory> {
            &self.bb_directories
        }
    }

    impl MockBBDirectory {
        pub fn new(location: &Path) -> Self {
            MockBBDirectory {
                dir: BBDirectory::new(location),
                mock_tally_component_shuffle_payload_file: None,
                mock_tally_component_votes_payload_file: None,
                mock_get_name: None,
            }
        }
        pub fn mock_tally_component_shuffle_payload_file(&mut self, data: &File) {
            self.mock_tally_component_shuffle_payload_file = Some(data.clone());
        }
        pub fn mock_tally_component_votes_payload_file(&mut self, data: &File) {
            self.mock_tally_component_votes_payload_file = Some(data.clone());
        }
        pub fn mock_get_name(&mut self, data: &str) {
            self.mock_get_name = Some(data.to_string())
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
                mock_e_voting_decrypt_file: None,
                mock_ech_0110_file: None,
                mock_ech_0222_file: None,
                bb_directories: bb_dirs,
            }
        }
        pub fn bb_directories_mut(&mut self) -> Vec<&mut MockBBDirectory> {
            self.bb_directories.iter_mut().collect()
        }

        pub fn mock_e_voting_decrypt_file(&mut self, data: &File) {
            self.mock_e_voting_decrypt_file = Some(data.clone());
        }
        pub fn mock_ech_0110_file(&mut self, data: &File) {
            self.mock_ech_0110_file = Some(data.clone());
        }
        pub fn mock_ech_0222_file(&mut self, data: &File) {
            self.mock_ech_0222_file = Some(data.clone());
        }
    }
}
