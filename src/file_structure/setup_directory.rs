//! Module to implement the setup directory

use super::{
    file_group::{
        add_type_for_file_group_iter_trait, impl_iterator_over_data_payload, FileGroup,
        FileGroupIter, FileGroupIterTrait,
    },
    CompletnessTestTrait, FileStructureError,
};
use crate::{
    config::Config,
    data_structures::{
        create_verifier_setup_data_type,
        setup::{
            control_component_code_shares_payload::ControlComponentCodeSharesPayload,
            setup_component_verification_data_payload::SetupComponentVerificationDataPayload,
            VerifierSetupDataType,
        },
        VerifierDataType, VerifierSetupDataTrait,
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The setup directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct SetupDirectory {
    location: PathBuf,
    vcs_directories: Vec<SetupVCSDirectory>,
}

/// The vcs directoy, containing the files, file groues and subdirectories
#[derive(Clone)]
pub struct SetupVCSDirectory {
    location: PathBuf,
    setup_component_verification_data_payload_group: FileGroup,
    control_component_code_shares_payload_group: FileGroup,
}

/// Trait to set the necessary functions for the struct [SetupDirectory] that
/// are used during the verifications
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait SetupDirectoryTrait: CompletnessTestTrait {
    type VCSDirType: SetupVCSDirectoryTrait;
    fn vcs_directories(&self) -> &[Self::VCSDirType];

    /// Collect the names of the vcs directories
    fn vcs_directory_names(&self) -> Vec<String> {
        self.vcs_directories().iter().map(|d| d.name()).collect()
    }

    fn location(&self) -> &Path;
}

/// Trait to set the necessary functions for the struct [SetupVCSDirectory] that
/// are used during the tests
///
/// The trait is used as parameter of the verification functions to allow mock of
/// test (negative tests)
pub trait SetupVCSDirectoryTrait: CompletnessTestTrait {
    add_type_for_file_group_iter_trait!(
        SetupComponentVerificationDataPayloadAsResultIterType,
        SetupComponentVerificationDataPayloadAsResult
    );
    add_type_for_file_group_iter_trait!(
        ControlComponentCodeSharesPayloadAsResultIterType,
        ControlComponentCodeSharesPayloadAsResult
    );

    fn setup_component_verification_data_payload_group(&self) -> &FileGroup;
    fn control_component_code_shares_payload_group(&self) -> &FileGroup;
    fn setup_component_verification_data_payload_iter(
        &self,
    ) -> Self::SetupComponentVerificationDataPayloadAsResultIterType;

    fn control_component_code_shares_payload_iter(
        &self,
    ) -> Self::ControlComponentCodeSharesPayloadAsResultIterType;

    fn name(&self) -> String;

    fn location(&self) -> &Path;
}

impl_iterator_over_data_payload!(
    SetupComponentVerificationDataPayload,
    setup_component_verification_data_payload,
    SetupComponentVerificationDataPayloadAsResult,
    SetupComponentVerificationDataPayloadAsResultIter
);

impl_iterator_over_data_payload!(
    ControlComponentCodeSharesPayload,
    control_component_code_shares_payload,
    ControlComponentCodeSharesPayloadAsResult,
    ControlComponentCodeSharesPayloadAsResultIter
);

impl SetupDirectory {
    /// New [SetupDirectory]
    #[allow(clippy::redundant_clone)]
    pub fn new(data_location: &Path) -> Self {
        let location = data_location.join(Config::setup_dir_name());
        let mut res = Self {
            location: location.to_path_buf(),
            vcs_directories: vec![],
        };
        let vcs_path = location.join(Config::vcs_dir_name());
        if vcs_path.is_dir() {
            for re in fs::read_dir(&vcs_path).unwrap() {
                let e = re.unwrap().path();
                if e.is_dir() {
                    res.vcs_directories.push(SetupVCSDirectory::new(&e))
                }
            }
        }
        res
    }
}

impl SetupDirectoryTrait for SetupDirectory {
    type VCSDirType = SetupVCSDirectory;
    fn vcs_directories(&self) -> &[Self::VCSDirType] {
        &self.vcs_directories
    }

    fn location(&self) -> &Path {
        self.location.as_path()
    }
}

macro_rules! impl_completness_test_trait_for_setup {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                let mut missings = vec![];
                if self.vcs_directories().is_empty() {
                    missings.push("No vcs directory found".to_string());
                }
                for d in self.vcs_directories().iter() {
                    missings.extend(d.test_completness()?)
                }
                Ok(missings)
            }
        }
    };
}
pub(crate) use impl_completness_test_trait_for_setup;

impl_completness_test_trait_for_setup!(SetupDirectory);

impl SetupVCSDirectory {
    /// New [VCSDirectory]
    pub fn new(location: &Path) -> Self {
        Self {
            location: location.to_path_buf(),
            setup_component_verification_data_payload_group: FileGroup::new(
                location,
                create_verifier_setup_data_type!(Setup, SetupComponentVerificationDataPayload),
            ),
            control_component_code_shares_payload_group: FileGroup::new(
                location,
                create_verifier_setup_data_type!(Setup, ControlComponentCodeSharesPayload),
            ),
        }
    }
}

macro_rules! impl_completness_test_trait_for_setup_vcs {
    ($t: ident) => {
        impl CompletnessTestTrait for $t {
            fn test_completness(&self) -> Result<Vec<String>, FileStructureError> {
                let mut missings = vec![];
                if !self
                    .setup_component_verification_data_payload_group()
                    .has_elements()
                {
                    missings.push(format!(
                        "{:?}/setup_component_verification_data_payload does not exist",
                        self.location().file_name().unwrap()
                    ))
                }
                if !self
                    .control_component_code_shares_payload_group()
                    .has_elements()
                {
                    missings.push(format!(
                        "{:?}/control_component_code_shares_payload does not exist",
                        self.location().file_name().unwrap()
                    ))
                }
                Ok(missings)
            }
        }
    };
}
pub(crate) use impl_completness_test_trait_for_setup_vcs;

impl_completness_test_trait_for_setup_vcs!(SetupVCSDirectory);

impl SetupVCSDirectoryTrait for SetupVCSDirectory {
    type SetupComponentVerificationDataPayloadAsResultIterType =
        SetupComponentVerificationDataPayloadAsResultIter;
    type ControlComponentCodeSharesPayloadAsResultIterType =
        ControlComponentCodeSharesPayloadAsResultIter;

    fn setup_component_verification_data_payload_group(&self) -> &FileGroup {
        &self.setup_component_verification_data_payload_group
    }
    fn control_component_code_shares_payload_group(&self) -> &FileGroup {
        &self.control_component_code_shares_payload_group
    }

    fn setup_component_verification_data_payload_iter(
        &self,
    ) -> Self::SetupComponentVerificationDataPayloadAsResultIterType {
        FileGroupIter::new(&self.setup_component_verification_data_payload_group)
    }

    fn control_component_code_shares_payload_iter(
        &self,
    ) -> Self::ControlComponentCodeSharesPayloadAsResultIterType {
        FileGroupIter::new(&self.control_component_code_shares_payload_group)
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
        test_all_setup_vcs_paths, test_datasets_path, test_datasets_setup_path,
        test_setup_verification_card_set_path,
    };

    #[test]
    fn test_setup_dir() {
        let setup_location = test_datasets_setup_path().join("setup");
        let dir = SetupDirectory::new(setup_location.parent().unwrap());
        assert_eq!(dir.location(), setup_location);
        let expected = test_all_setup_vcs_paths();
        for d in dir.vcs_directories().iter() {
            let j = expected.iter().position(|l| d.location() == l).unwrap();
            assert_eq!(d.location(), expected[j])
        }
    }

    #[test]
    fn test_vcs_dir() {
        let location = test_setup_verification_card_set_path();
        let dir = SetupVCSDirectory::new(&location);
        assert_eq!(dir.location(), location);
        for (i, p) in dir.control_component_code_shares_payload_iter() {
            assert!(p.is_ok());
            for k in p.unwrap().0.iter() {
                assert_eq!(k.chunk_id, i)
            }
        }
        for (i, p) in dir.setup_component_verification_data_payload_iter() {
            assert!(p.is_ok());
            assert_eq!(p.unwrap().chunk_id, i)
        }
    }

    #[test]
    fn test_completness() {
        let dir = SetupDirectory::new(&test_datasets_path());
        let c = dir.test_completness();
        assert!(c.is_ok());
        assert!(c.unwrap().is_empty());
    }
}
