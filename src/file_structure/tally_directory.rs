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
    pub e_voting_decrypt_file: File,
    pub ech_0110_file: File,
    pub ech_0222_file: File,
    pub bb_directories: Box<Vec<BBDirectory>>,
}

#[derive(Clone)]
pub struct BBDirectory {
    location: PathBuf,
    pub tally_component_votes_payload_file: File,
    pub tally_component_shuffle_payload_file: File,
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
            bb_directories: Box::new(vec![]),
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

    pub fn get_location(&self) -> PathBuf {
        self.location.to_path_buf()
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

    pub fn get_location(&self) -> PathBuf {
        self.location.to_path_buf()
    }

    pub fn get_name(&self) -> String {
        self.location
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}
