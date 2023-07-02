use rust_verifier_lib::constants::verification_list_path;
use rust_verifier_lib::verification::meta_data::{VerificationMetaData, VerificationMetaDataList};
use rust_verifier_lib::verification::VerificationPeriod;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

use super::root_dir;

#[derive(Clone, Copy, serde::Serialize)]
enum VerificationStatus {
    #[serde(rename = "Not started")]
    NotStarted,
    Running,
    Successful,
    #[serde(rename = "Finished with errors")]
    Errors,
    #[serde(rename = "Finished with failures")]
    Failures,
    #[serde(rename = "Finished with errors and failures")]
    ErrorsAndFailures,
}

#[derive(Clone, serde::Serialize)]
struct Verification {
    id: String,
    name: String,
    algorithm: String,
    description: String,
    category: String,
    status: VerificationStatus,
    errors: Vec<String>,
    failures: Vec<String>,
}

#[derive(Clone, serde::Serialize)]
struct VerificationListPayload(Vec<Verification>);

impl From<&VerificationMetaData> for Verification {
    fn from(value: &VerificationMetaData) -> Self {
        Self {
            id: value.id().clone(),
            name: value.name().clone(),
            algorithm: value.algorithm().clone(),
            description: value.description().clone(),
            category: value.category().to_string(),
            status: VerificationStatus::NotStarted,
            errors: vec![],
            failures: vec![],
        }
    }
}

impl From<&VerificationMetaDataList> for VerificationListPayload {
    fn from(value: &VerificationMetaDataList) -> Self {
        Self(
            value
                .iter()
                .map(Verification::from)
                .collect::<Vec<Verification>>(),
        )
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn get_verifications(is_tally: bool) -> VerificationListPayload {
    let p = match is_tally {
        true => VerificationPeriod::Tally,
        false => VerificationPeriod::Setup,
    };
    VerificationListPayload::from(
        &VerificationMetaDataList::load_period(&verification_list_path(Some(&root_dir())), &p)
            .unwrap(),
    )
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("verifications")
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![get_verifications])
        .build()
}
