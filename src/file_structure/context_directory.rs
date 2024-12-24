//! Module to implement the context directory

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
        context::{
            control_component_public_keys_payload::ControlComponentPublicKeysPayload,
            election_event_configuration::ElectionEventConfiguration,
            election_event_context_payload::ElectionEventContextPayload,
            setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
            setup_component_tally_data_payload::SetupComponentTallyDataPayload,
            VerifierContextDataType,
        },
        create_verifier_context_data_type, VerifierContextDataTrait, VerifierDataType,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The context directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct ContextDirectory {
    location: PathBuf,
    setup_component_public_keys_payload_file: File,
    election_event_context_payload_file: File,
    election_event_configuration_file: File,
    control_component_public_keys_payload_group: FileGroup,
    vcs_directories: Vec<ContextVCSDirectory>,
}

/// The vcs directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct ContextVCSDirectory {
    location: PathBuf,
    setup_component_tally_data_payload_file: File,
}

/// Trait to set the necessary functions for the struct [ContextDirectory] that
/// are used during the verifications
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait ContextDirectoryTrait: CompletnessTestTrait + Send + Sync {
    type VCSDirType: ContextVCSDirectoryTrait;
    add_type_for_file_group_iter_trait!(
        ControlComponentPublicKeysPayloadAsResultIterType,
        ControlComponentPublicKeysPayloadAsResult
    );

    fn setup_component_public_keys_payload_file(&self) -> &File;
    fn election_event_context_payload_file(&self) -> &File;
    fn election_event_configuration_file(&self) -> &File;
    fn control_component_public_keys_payload_group(&self) -> &FileGroup;
    fn vcs_directories(&self) -> &[Self::VCSDirType];
    fn setup_component_public_keys_payload(
        &self,
    ) -> Result<Box<SetupComponentPublicKeysPayload>, FileStructureError>;

    fn election_event_context_payload(
        &self,
    ) -> Result<Box<ElectionEventContextPayload>, FileStructureError>;
    fn election_event_configuration(
        &self,
    ) -> Result<Box<ElectionEventConfiguration>, FileStructureError>;

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> Self::ControlComponentPublicKeysPayloadAsResultIterType;

    /// Collect the names of the vcs directories
    fn vcs_directory_names(&self) -> Vec<String> {
        self.vcs_directories().iter().map(|d| d.name()).collect()
    }

    fn location(&self) -> &Path;
}

/// Trait to set the necessary functions for the struct [VCSDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait ContextVCSDirectoryTrait: CompletnessTestTrait + Send + Sync {
    fn setup_component_tally_data_payload_file(&self) -> &File;
    fn setup_component_tally_data_payload(
        &self,
    ) -> Result<Box<SetupComponentTallyDataPayload>, FileStructureError>;
    fn name(&self) -> String;
    fn location(&self) -> &Path;
}

impl_iterator_over_data_payload!(
    ControlComponentPublicKeysPayload,
    control_component_public_keys_payload,
    ControlComponentPublicKeysPayloadAsResult,
    ControlComponentPublicKeysPayloadAsResultIter
);

impl ContextDirectory {
    /// New [ContextDirectory]
    #[allow(clippy::redundant_clone)]
    pub fn new(data_location: &Path) -> Self {
        let location = data_location.join(Config::context_dir_name());
        let mut res = Self {
            location: location.to_path_buf(),
            setup_component_public_keys_payload_file: create_file!(
                location,
                Context,
                VerifierContextDataType::SetupComponentPublicKeysPayload
            ),
            election_event_context_payload_file: create_file!(
                location,
                Context,
                VerifierContextDataType::ElectionEventContextPayload
            ),
            election_event_configuration_file: create_file!(
                location,
                Context,
                VerifierContextDataType::ElectionEventConfiguration
            ),
            control_component_public_keys_payload_group: FileGroup::new(
                &location,
                create_verifier_context_data_type!(Context, ControlComponentPublicKeysPayload),
            ),
            vcs_directories: vec![],
        };
        let vcs_path = location.join(Config::vcs_dir_name());
        if vcs_path.is_dir() {
            for re in fs::read_dir(&vcs_path).unwrap() {
                let e = re.unwrap().path();
                if e.is_dir() {
                    res.vcs_directories.push(ContextVCSDirectory::new(&e))
                }
            }
        }
        res
    }
}

macro_rules! impl_completness_test_trait_for_context {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                if !self.location().is_dir() {
                    return Err(FileStructureError::PathIsNotDir(self.location().to_path_buf()))
                }
                let mut missings = vec![];
                if !self.election_event_context_payload_file().exists() {
                    missings.push("election_event_context_payload does not exist".to_string());
                }
                if !self.setup_component_public_keys_payload_file().exists() {
                    missings.push("setup_component_public_keys_payload_file does not exist".to_string());
                }
                if !self.election_event_configuration_file().exists() {
                    missings.push("setup_component_public_keys_payload_file does not exist".to_string());
                }
                if self
                    .control_component_public_keys_payload_group()
                    .get_numbers()
                    != &vec![1, 2, 3, 4]
                {
                    missings.push(format!(
                        "control_component_public_keys_payload_group missing. only these parts are present: {:?}",
                        self
                            .control_component_public_keys_payload_group()
                            .get_numbers()
                    ))
                }
                if self.vcs_directories().is_empty() {
                    missings.push("No vcs directory found".to_string());
                }
                for d in self.vcs_directories().iter() {
                    missings.extend(d.test_completness()?);
                }
                Ok(missings)
            }
        }
    }
}
pub(crate) use impl_completness_test_trait_for_context;

impl_completness_test_trait_for_context!(ContextDirectory);

impl ContextDirectoryTrait for ContextDirectory {
    type VCSDirType = ContextVCSDirectory;
    type ControlComponentPublicKeysPayloadAsResultIterType =
        ControlComponentPublicKeysPayloadAsResultIter;

    fn setup_component_public_keys_payload_file(&self) -> &File {
        &self.setup_component_public_keys_payload_file
    }
    fn election_event_context_payload_file(&self) -> &File {
        &self.election_event_context_payload_file
    }
    fn election_event_configuration_file(&self) -> &File {
        &self.election_event_configuration_file
    }
    fn control_component_public_keys_payload_group(&self) -> &FileGroup {
        &self.control_component_public_keys_payload_group
    }
    fn vcs_directories(&self) -> &[ContextVCSDirectory] {
        &self.vcs_directories
    }

    fn setup_component_public_keys_payload(
        &self,
    ) -> Result<Box<SetupComponentPublicKeysPayload>, FileStructureError> {
        self.setup_component_public_keys_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.setup_component_public_keys_payload().unwrap().clone()))
    }

    fn election_event_context_payload(
        &self,
    ) -> Result<Box<ElectionEventContextPayload>, FileStructureError> {
        self.election_event_context_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.election_event_context_payload().unwrap().clone()))
    }

    fn election_event_configuration(
        &self,
    ) -> Result<Box<ElectionEventConfiguration>, FileStructureError> {
        self.election_event_configuration_file
            .get_verifier_data()
            .map(|d| Box::new(d.election_event_configuration().unwrap().clone()))
    }

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> Self::ControlComponentPublicKeysPayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_public_keys_payload_group)
    }

    fn location(&self) -> &Path {
        self.location.as_path()
    }
}

macro_rules! impl_completness_test_trait_for_context_vcs {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                if !self.location().is_dir() {
                    return Err(FileStructureError::PathIsNotDir(
                        self.location().to_path_buf(),
                    ));
                }
                let mut missings = vec![];
                if !self.setup_component_tally_data_payload_file().exists() {
                    missings.push(format!(
                        "setup_component_tally_data_payload does not exist in {:?}",
                        self.location().file_name().unwrap()
                    ))
                }
                Ok(missings)
            }
        }
    };
}
pub(crate) use impl_completness_test_trait_for_context_vcs;

impl_completness_test_trait_for_context_vcs!(ContextVCSDirectory);

impl ContextVCSDirectory {
    /// New [VCSDirectory]
    pub fn new(location: &Path) -> Self {
        Self {
            location: location.to_path_buf(),
            setup_component_tally_data_payload_file: create_file!(
                location,
                Context,
                VerifierContextDataType::SetupComponentTallyDataPayload
            ),
        }
    }
}

impl ContextVCSDirectoryTrait for ContextVCSDirectory {
    fn setup_component_tally_data_payload_file(&self) -> &File {
        &self.setup_component_tally_data_payload_file
    }
    fn setup_component_tally_data_payload(
        &self,
    ) -> Result<Box<SetupComponentTallyDataPayload>, FileStructureError> {
        self.setup_component_tally_data_payload_file
            .get_verifier_data()
            .map(|d| Box::new(d.setup_component_tally_data_payload().unwrap().clone()))
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::{
        test_all_context_vcs_paths, test_context_verification_card_set_path,
        test_datasets_context_path, test_datasets_path,
    };

    #[test]
    fn test_context_dir() {
        let context_location = test_datasets_context_path();
        let dir = ContextDirectory::new(context_location.parent().unwrap());
        assert_eq!(dir.location(), context_location);
        assert!(dir.setup_component_public_keys_payload().is_ok());
        assert!(dir.election_event_context_payload().is_ok());
        for (i, p) in dir.control_component_public_keys_payload_iter() {
            assert!(p.is_ok());
            assert_eq!(p.unwrap().control_component_public_keys.node_id, i)
        }
        let expected = test_all_context_vcs_paths();
        for d in dir.vcs_directories().iter() {
            let j = expected.iter().position(|l| d.location() == l).unwrap();
            assert_eq!(d.location(), expected[j])
        }
    }

    #[test]
    fn test_context_vcs_dir() {
        let location = test_context_verification_card_set_path();
        let dir = ContextVCSDirectory::new(&location);
        assert_eq!(dir.location(), location);
        assert!(dir.setup_component_tally_data_payload().is_ok());
    }

    #[test]
    fn test_completness() {
        let dir = ContextDirectory::new(&test_datasets_path());
        let c = dir.test_completness();
        assert!(c.is_ok());
        assert!(c.unwrap().is_empty());
    }
}
