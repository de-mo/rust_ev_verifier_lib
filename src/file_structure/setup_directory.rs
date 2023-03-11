use super::{
    structure::{create_file, Directory, File, FileGroup},
    FileStructureError, FileStructureErrorType, GetFileName,
};
use crate::{
    data_structures::{
        create_verifier_data_type,
        setup::{
            election_event_context_payload::ElectionEventContextPayload,
            encryption_parameters_payload::EncryptionParametersPayload,
            setup_component_public_keys_payload::SetupComponentPublicKeysPayload,
            VerifierSetupDataType,
        },
        VerifierDataTrait, VerifierDataType,
    },
    error::create_result_with_error,
    verification::VerificationPeriod,
};
use std::{
    borrow::Borrow,
    ops::Deref,
    path::{Path, PathBuf},
};

pub struct VCSDirectory {
    location: PathBuf,
    setup_component_tally_data_payload: File,
    setup_component_verification_data_payload_group: FileGroup,
    control_component_code_shares_payload_group: FileGroup,
}

pub struct SetupDirectory {
    location: PathBuf,
    encryption_parameters_payload_file: File,
    setup_component_public_keys_payload_file: File,
    election_event_context_payload_file: File,
    control_component_public_keys_payload_group: FileGroup,
    vcs_directories: Box<Vec<SetupDirectory>>,
}

impl SetupDirectory {
    pub fn new(data_location: &Path) -> SetupDirectory {
        let location = data_location.join("setup");
        SetupDirectory {
            location: location.to_path_buf(),
            encryption_parameters_payload_file: create_file!(
                location,
                Setup,
                EncryptionParametersPayload
            ),
            setup_component_public_keys_payload_file: create_file!(
                location,
                Setup,
                SetupComponentPublicKeysPayload
            ),
            election_event_context_payload_file: create_file!(
                location,
                Setup,
                ElectionEventContextPayload
            ),
            control_component_public_keys_payload_group: FileGroup::new(
                &location,
                create_verifier_data_type!(Setup, ControlComponentPublicKeysPayload),
            ),
            vcs_directories: Box::new(vec![]),
        }
    }

    pub fn encryption_parameters_payload(
        &self,
    ) -> Result<Box<EncryptionParametersPayload>, FileStructureError> {
        self.encryption_parameters_payload_file
            .get_data()
            .map(|d| d.encryption_parameters_payload().unwrap())
    }

    pub fn setup_component_public_keys_payload(
        &self,
    ) -> Result<Box<SetupComponentPublicKeysPayload>, FileStructureError> {
        self.setup_component_public_keys_payload_file
            .get_data()
            .map(|d| d.setup_component_public_keys_payload().unwrap())
    }

    pub fn election_event_context_payload(
        &self,
    ) -> Result<Box<ElectionEventContextPayload>, FileStructureError> {
        self.election_event_context_payload_file
            .get_data()
            .map(|d| d.election_event_context_payload().unwrap())
    }
}
