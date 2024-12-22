use super::{
    file::{create_file, File},
    file_group::{
        add_type_for_file_group_iter_trait, impl_iterator_over_data_payload, FileGroup,
        FileGroupIter, FileGroupIterTrait,
    },
    CompletnessTestTrait, FileStructureError,
};
use crate::{
    config::Config,
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
    control_component_shuffle_payload_group: FileGroup,
}

/// Trait to set the necessary functions for the struct [TallyDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait TallyDirectoryTrait: CompletnessTestTrait + Send + Sync {
    type BBDirType: BBDirectoryTrait;

    fn e_voting_decrypt_file(&self) -> &File;
    fn ech_0110_file(&self) -> &File;
    fn ech_0222_file(&self) -> &File;
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
    add_type_for_file_group_iter_trait!(
        ControlComponentBallotBoxPayloadAsResultIterType,
        ControlComponentBallotBoxPayloadAsResult
    );
    add_type_for_file_group_iter_trait!(
        ControlComponentShufflePayloadAsResultIterType,
        ControlComponentShufflePayloadAsResult
    );
    fn tally_component_votes_payload_file(&self) -> &File;
    fn tally_component_shuffle_payload_file(&self) -> &File;
    fn control_component_ballot_box_payload_group(&self) -> &FileGroup;
    fn control_component_shuffle_payload_group(&self) -> &FileGroup;
    fn tally_component_votes_payload(
        &self,
    ) -> Result<Box<TallyComponentVotesPayload>, FileStructureError>;
    fn tally_component_shuffle_payload(
        &self,
    ) -> Result<Box<TallyComponentShufflePayload>, FileStructureError>;
    fn control_component_ballot_box_payload_iter(
        &self,
    ) -> Self::ControlComponentBallotBoxPayloadAsResultIterType;
    fn control_component_shuffle_payload_iter(
        &self,
    ) -> Self::ControlComponentShufflePayloadAsResultIterType;

    fn name(&self) -> String;
    fn location(&self) -> &Path;
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
    ControlComponentShufflePayloadAsResult,
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
    type ControlComponentBallotBoxPayloadAsResultIterType =
        ControlComponentBallotBoxPayloadAsResultIter;
    type ControlComponentShufflePayloadAsResultIterType =
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
    fn tally_component_votes_payload(
        &self,
    ) -> Result<Box<TallyComponentVotesPayload>, FileStructureError> {
        self.tally_component_votes_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.tally_component_votes_payload().unwrap().clone()))
    }
    fn tally_component_shuffle_payload(
        &self,
    ) -> Result<Box<TallyComponentShufflePayload>, FileStructureError> {
        self.tally_component_shuffle_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.tally_component_shuffle_payload().unwrap().clone()))
    }

    fn control_component_ballot_box_payload_iter(
        &self,
    ) -> Self::ControlComponentBallotBoxPayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_ballot_box_payload_group)
    }

    fn control_component_shuffle_payload_iter(
        &self,
    ) -> Self::ControlComponentShufflePayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_shuffle_payload_group)
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
        let location = data_location.join(Config::tally_dir_name());
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
        let bb_path = location.join(Config::bb_dir_name());
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
                location,
                create_verifier_tally_data_type!(Tally, ControlComponentBallotBoxPayload),
            ),
            control_component_shuffle_payload_group: FileGroup::new(
                location,
                create_verifier_tally_data_type!(Tally, ControlComponentShufflePayload),
            ),
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

/*
#[cfg(any(test, doc))]

pub mod mock {
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

    use std::collections::HashMap;

    /// Mock for [BBDirectory]
    pub struct MockBBDirectory {
        location: PathBuf,
        dir: BBDirectory,
        mocked_tally_component_votes_payload_file: Option<File>,
        mocked_tally_component_shuffle_payload_file: Option<File>,
        mocked_control_component_ballot_box_payload_group: Option<FileGroup>,
        mocked_control_component_shuffle_payload_group: Option<FileGroup>,
        mocked_tally_component_votes_payload:
            Option<Result<Box<TallyComponentVotesPayload>, FileStructureError>>,
        mocked_tally_component_shuffle_payload:
            Option<Result<Box<TallyComponentShufflePayload>, FileStructureError>>,
        mocked_control_component_ballot_box_payloads:
            HashMap<usize, ControlComponentBallotBoxPayloadAsResult>,
        mocked_control_component_shuffle_payloads:
            HashMap<usize, ControlComponentShufflePayloadloadAsResult>,

        mocked_get_name: Option<String>,
    }

    /// Mock for [TallyDirectory]
    pub struct MockTallyDirectory {
        location: PathBuf,
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

    impl_completness_test_trait_for_tally!(MockTallyDirectory);

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

    impl_completness_test_trait_for_tally_bb!(MockBBDirectory);

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
                location: location.to_path_buf(),
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
        pub fn mock_tally_component_shuffle_payload_file(&mut self, data: &File) {
            self.mocked_tally_component_shuffle_payload_file = Some(data.clone());
        }
        pub fn mock_tally_component_votes_payload_file(&mut self, data: &File) {
            self.mocked_tally_component_votes_payload_file = Some(data.clone());
        }
        pub fn mock_control_component_ballot_box_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_ballot_box_payload_group = Some(data.clone());
        }
        pub fn mock_control_component_shuffle_payload_group(&mut self, data: &FileGroup) {
            self.mocked_control_component_shuffle_payload_group = Some(data.clone());
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
                location: tally_dir.location.to_owned(),
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
 */
