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
    file::{create_file, File},
    file_group::{FileGroup, FileGroupDataIter, FileGroupFileIter},
    CompletnessTestTrait, FileStructureError,
};
use crate::{
    config::VerifierConfig,
    data_structures::tally::{
        control_component_ballot_box_payload::ControlComponentBallotBoxPayload,
        control_component_shuffle_payload::ControlComponentShufflePayload,
        e_voting_decrypt::EVotingDecrypt, ech_0110::ECH0110, ech_0222::ECH0222,
        tally_component_shuffle_payload::TallyComponentShufflePayload,
        tally_component_votes_payload::TallyComponentVotesPayload,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

//#[derive(Clone)]
pub struct TallyDirectory {
    location: PathBuf,
    e_voting_decrypt_file: File<EVotingDecrypt>,
    ech_0110_file: File<ECH0110>,
    ech_0222_file: File<ECH0222>,
    bb_directories: Vec<BBDirectory>,
}

//#[derive(Clone)]
pub struct BBDirectory {
    location: PathBuf,
    tally_component_votes_payload_file: File<TallyComponentVotesPayload>,
    tally_component_shuffle_payload_file: File<TallyComponentShufflePayload>,
    control_component_ballot_box_payload_group: FileGroup<ControlComponentBallotBoxPayload>,
    control_component_shuffle_payload_group: FileGroup<ControlComponentShufflePayload>,
}

/// Trait to set the necessary functions for the struct `Tally Directory` that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait TallyDirectoryTrait: CompletnessTestTrait + Send + Sync {
    type BBDirType: BBDirectoryTrait;

    fn e_voting_decrypt_file(&self) -> &File<EVotingDecrypt>;
    fn ech_0110_file(&self) -> &File<ECH0110>;
    fn ech_0222_file(&self) -> &File<ECH0222>;
    fn bb_directories(&self) -> &[Self::BBDirType];

    /// Collect the names of the ballot box directories
    fn bb_directory_names(&self) -> Vec<String> {
        self.bb_directories().iter().map(|d| d.name()).collect()
    }

    fn location(&self) -> &Path;
}

/// Trait to set the necessary functions for the struct [BBDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait BBDirectoryTrait: CompletnessTestTrait + Send + Sync {
    fn tally_component_votes_payload_file(&self) -> &File<TallyComponentVotesPayload>;
    fn tally_component_shuffle_payload_file(&self) -> &File<TallyComponentShufflePayload>;
    fn control_component_ballot_box_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentBallotBoxPayload>;
    fn control_component_shuffle_payload_group(&self)
        -> &FileGroup<ControlComponentShufflePayload>;
    fn tally_component_votes_payload(
        &self,
    ) -> Result<Arc<TallyComponentVotesPayload>, FileStructureError>;
    fn tally_component_shuffle_payload(
        &self,
    ) -> Result<Arc<TallyComponentShufflePayload>, FileStructureError>;
    fn control_component_ballot_box_payload_iter(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            Result<Arc<ControlComponentBallotBoxPayload>, FileStructureError>,
        ),
    >;
    fn control_component_shuffle_payload_iter(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            Result<Arc<ControlComponentShufflePayload>, FileStructureError>,
        ),
    >;

    fn name(&self) -> String;
    fn location(&self) -> &Path;
}

impl TallyDirectoryTrait for TallyDirectory {
    type BBDirType = BBDirectory;

    fn e_voting_decrypt_file(&self) -> &File<EVotingDecrypt> {
        &self.e_voting_decrypt_file
    }
    fn ech_0110_file(&self) -> &File<ECH0110> {
        &self.ech_0110_file
    }
    fn ech_0222_file(&self) -> &File<ECH0222> {
        &self.ech_0222_file
    }
    fn bb_directories(&self) -> &[BBDirectory] {
        &self.bb_directories
    }

    fn location(&self) -> &Path {
        self.location.as_path()
    }
}

macro_rules! impl_completness_test_trait_for_tally {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                let mut missings = vec![];
                if !self.ech_0110_file().exists() {
                    missings.push("ech_0110 does not exist".to_string())
                }
                if !self.ech_0222_file().exists() {
                    missings.push("ech_0222 does not exist".to_string())
                }
                if !self.e_voting_decrypt_file().exists() {
                    missings.push("e_voting_decrypt does not exist".to_string())
                }
                if self.bb_directories().is_empty() {
                    missings.push("No bb directory found".to_string());
                }
                for d in self.bb_directories().iter() {
                    missings.extend(d.test_completness()?)
                }
                Ok(missings)
            }
        }
    };
}
pub(crate) use impl_completness_test_trait_for_tally;

impl_completness_test_trait_for_tally!(TallyDirectory);

impl BBDirectoryTrait for BBDirectory {
    fn tally_component_votes_payload_file(&self) -> &File<TallyComponentVotesPayload> {
        &self.tally_component_votes_payload_file
    }
    fn tally_component_shuffle_payload_file(&self) -> &File<TallyComponentShufflePayload> {
        &self.tally_component_shuffle_payload_file
    }
    fn control_component_ballot_box_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentBallotBoxPayload> {
        &self.control_component_ballot_box_payload_group
    }
    fn control_component_shuffle_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentShufflePayload> {
        &self.control_component_shuffle_payload_group
    }
    fn tally_component_votes_payload(
        &self,
    ) -> Result<Arc<TallyComponentVotesPayload>, FileStructureError> {
        self.tally_component_votes_payload_file
            .decode_verifier_data()
    }
    fn tally_component_shuffle_payload(
        &self,
    ) -> Result<Arc<TallyComponentShufflePayload>, FileStructureError> {
        self.tally_component_shuffle_payload_file
            .decode_verifier_data()
    }

    fn control_component_ballot_box_payload_iter(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            Result<Arc<ControlComponentBallotBoxPayload>, FileStructureError>,
        ),
    > {
        FileGroupDataIter::from(FileGroupFileIter::new(
            &self.control_component_ballot_box_payload_group,
        ))
    }

    fn control_component_shuffle_payload_iter(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            Result<Arc<ControlComponentShufflePayload>, FileStructureError>,
        ),
    > {
        FileGroupDataIter::from(FileGroupFileIter::new(
            &self.control_component_shuffle_payload_group,
        ))
    }

    fn name(&self) -> String {
        self.location
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn location(&self) -> &Path {
        self.location.as_path()
    }
}

macro_rules! impl_completness_test_trait_for_tally_bb {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                let mut missings = vec![];
                if !self.tally_component_shuffle_payload_file().exists() {
                    missings.push(format!(
                        "{:?}/tally_component_shuffle_payload does not exist",
                        self.location().file_name().unwrap()
                    ))
                }
                if !self.tally_component_shuffle_payload_file().exists() {
                    missings.push(format!(
                        "{:?}/tally_component_shuffle_payload does not exist",
                        self.location().file_name().unwrap()
                    ))
                }
                if self
                    .control_component_ballot_box_payload_group()
                    .get_numbers()
                    != &vec![1, 2, 3, 4]
                {
                    missings.push(format!(
                        "{:?}/control_component_ballot_box_payload missing. only these parts are present: {:?}",
                        self.location().file_name().unwrap(),
                        self
                            .control_component_ballot_box_payload_group()
                            .get_numbers()
                    ))
                }
                if self
                    .control_component_shuffle_payload_group()
                    .get_numbers()
                    != &vec![1, 2, 3, 4]
                {
                    missings.push(format!(
                        "{:?}/control_component_shuffle_payload_group missing. only these parts are present: {:?}",
                        self.location().file_name().unwrap(),
                        self
                            .control_component_shuffle_payload_group()
                            .get_numbers()
                    ))
                }
                Ok(missings)
            }
        }
    };
}
pub(crate) use impl_completness_test_trait_for_tally_bb;

impl_completness_test_trait_for_tally_bb!(BBDirectory);

impl TallyDirectory {
    #[allow(clippy::redundant_clone)]
    pub fn new(data_location: &Path) -> TallyDirectory {
        let location = data_location.join(VerifierConfig::tally_dir_name());
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
        let bb_path = location.join(VerifierConfig::bb_dir_name());
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
            control_component_ballot_box_payload_group: FileGroup::new(location),
            control_component_shuffle_payload_group: FileGroup::new(location),
        }
    }

    pub fn get_location(&self) -> &Path {
        self.location.as_path()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_datasets_path;

    #[test]
    fn test_completness() {
        let dir = TallyDirectory::new(&test_datasets_path());
        let c = dir.test_completness();
        assert!(c.is_ok());
        assert!(c.unwrap().is_empty());
    }
}
