use super::file::{create_file, File};
use crate::{
    constants::{BB_DIR_NAME, TALLY_DIR_NAME},
    data_structures::{
        tally::{
            e_voting_decrypt::EVotingDecrypt, ech_0110::ECH0110, VerifierTallyData,
            VerifierTallyDataType,
        },
        VerifierDataType,
    },
};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct TallyDirectory {
    location: PathBuf,
    pub e_voting_decrypt: File,
    pub ech_110: File,
    pub bb_directories: Box<Vec<BBDirectory>>,
}

#[derive(Clone)]
pub struct BBDirectory {
    location: PathBuf,
}

impl TallyDirectory {
    pub fn new(data_location: &Path) -> TallyDirectory {
        let location = data_location.join(TALLY_DIR_NAME);
        TallyDirectory {
            location: location.to_path_buf(),
            e_voting_decrypt: create_file!(location, Tally, VerifierTallyDataType::EVotingDecrypt),
            ech_110: create_file!(location, Tally, VerifierTallyDataType::ECH0110),
            bb_directories: Box::new(vec![]),
        }
    }

    pub fn get_location(&self) -> PathBuf {
        self.location.to_path_buf()
    }
}
