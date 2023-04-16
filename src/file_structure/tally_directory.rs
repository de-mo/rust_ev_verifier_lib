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
pub trait TallyDirectoryTrait<B>
where
    B: BBDirectoryTrait,
{
    fn e_voting_decrypt_file(&self) -> &File;
    fn ech_0110_file(&self) -> &File;
    fn ech_0222_file(&self) -> &File;
    fn bb_directories(&self) -> &Vec<B>;
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

impl TallyDirectoryTrait<BBDirectory> for TallyDirectory {
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
