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

//! Module to implement the context directory

use super::{
    file::{create_file, File},
    file_group::{FileGroup, FileGroupDataIter, FileGroupFileIter},
    CompletnessTestTrait, FileStructureError,
};
use crate::{
    config::VerifierConfig,
    data_structures::context::{
        control_component_public_keys_payload::ControlComponentPublicKeysPayload,
        election_event_configuration::ElectionEventConfiguration,
        election_event_context_payload::ElectionEventContextPayload,
        setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
        setup_component_tally_data_payload::SetupComponentTallyDataPayload,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

/// The context directoy, containing the files, file groues and subdirectories
//#[derive(Clone)]
pub struct ContextDirectory {
    location: PathBuf,
    setup_component_public_keys_payload_file: File<SetupComponentPublicKeysPayload>,
    election_event_context_payload_file: File<ElectionEventContextPayload>,
    election_event_configuration_file: File<ElectionEventConfiguration>,
    control_component_public_keys_payload_group: FileGroup<ControlComponentPublicKeysPayload>,
    vcs_directories: Vec<ContextVCSDirectory>,
}

/// The vcs directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct ContextVCSDirectory {
    location: PathBuf,
    setup_component_tally_data_payload_file: File<SetupComponentTallyDataPayload>,
}

/// Trait to set the necessary functions for the struct [ContextDirectory] that
/// are used during the verifications
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait ContextDirectoryTrait: CompletnessTestTrait + Send + Sync {
    type VCSDirType: ContextVCSDirectoryTrait;

    fn setup_component_public_keys_payload_file(&self) -> &File<SetupComponentPublicKeysPayload>;
    fn election_event_context_payload_file(&self) -> &File<ElectionEventContextPayload>;
    fn election_event_configuration_file(&self) -> &File<ElectionEventConfiguration>;
    fn control_component_public_keys_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentPublicKeysPayload>;
    fn vcs_directories(&self) -> &[Self::VCSDirType];
    fn setup_component_public_keys_payload(
        &self,
    ) -> Result<Arc<SetupComponentPublicKeysPayload>, FileStructureError>;

    fn election_event_context_payload(
        &self,
    ) -> Result<Arc<ElectionEventContextPayload>, FileStructureError>;
    fn election_event_configuration(
        &self,
    ) -> Result<Arc<ElectionEventConfiguration>, FileStructureError>;

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            Result<Arc<ControlComponentPublicKeysPayload>, FileStructureError>,
        ),
    >;

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
    fn setup_component_tally_data_payload_file(&self) -> &File<SetupComponentTallyDataPayload>;
    fn setup_component_tally_data_payload(
        &self,
    ) -> Result<Arc<SetupComponentTallyDataPayload>, FileStructureError>;
    fn name(&self) -> String;
    fn location(&self) -> &Path;
}

impl ContextDirectory {
    /// New [ContextDirectory]
    #[allow(clippy::redundant_clone)]
    pub fn new(data_location: &Path) -> Self {
        let location = data_location.join(VerifierConfig::context_dir_name());
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
            control_component_public_keys_payload_group: FileGroup::new(&location),
            vcs_directories: vec![],
        };
        let vcs_path = location.join(VerifierConfig::vcs_dir_name());
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

    fn setup_component_public_keys_payload_file(&self) -> &File<SetupComponentPublicKeysPayload> {
        &self.setup_component_public_keys_payload_file
    }
    fn election_event_context_payload_file(&self) -> &File<ElectionEventContextPayload> {
        &self.election_event_context_payload_file
    }
    fn election_event_configuration_file(&self) -> &File<ElectionEventConfiguration> {
        &self.election_event_configuration_file
    }
    fn control_component_public_keys_payload_group(
        &self,
    ) -> &FileGroup<ControlComponentPublicKeysPayload> {
        &self.control_component_public_keys_payload_group
    }
    fn vcs_directories(&self) -> &[ContextVCSDirectory] {
        &self.vcs_directories
    }

    fn setup_component_public_keys_payload(
        &self,
    ) -> Result<Arc<SetupComponentPublicKeysPayload>, FileStructureError> {
        self.setup_component_public_keys_payload_file
            .decode_verifier_data()
    }

    fn election_event_context_payload(
        &self,
    ) -> Result<Arc<ElectionEventContextPayload>, FileStructureError> {
        self.election_event_context_payload_file
            .decode_verifier_data()
    }

    fn election_event_configuration(
        &self,
    ) -> Result<Arc<ElectionEventConfiguration>, FileStructureError> {
        self.election_event_configuration_file
            .decode_verifier_data()
    }

    fn control_component_public_keys_payload_iter(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            Result<Arc<ControlComponentPublicKeysPayload>, FileStructureError>,
        ),
    > {
        FileGroupDataIter::from(FileGroupFileIter::new(
            &self.control_component_public_keys_payload_group,
        ))
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
    fn setup_component_tally_data_payload_file(&self) -> &File<SetupComponentTallyDataPayload> {
        &self.setup_component_tally_data_payload_file
    }
    fn setup_component_tally_data_payload(
        &self,
    ) -> Result<Arc<SetupComponentTallyDataPayload>, FileStructureError> {
        self.setup_component_tally_data_payload_file
            .decode_verifier_data()
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
