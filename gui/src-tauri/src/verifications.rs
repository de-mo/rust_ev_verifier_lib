use super::CONFIG;
use rust_verifier_lib::verification::{
    meta_data::{VerificationMetaData, VerificationMetaDataList},
    suite::get_not_implemented_verifications_id,
    VerificationPeriod,
};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[derive(Clone, Copy, Debug, serde::Serialize)]
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
    #[serde(rename = "Not Implemented")]
    NotImplemented,
}

#[derive(Clone, Debug, serde::Serialize)]
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

impl VerificationListPayload {
    fn from_medata_list(period: VerificationPeriod, list: &VerificationMetaDataList) -> Self {
        let mut res = list
            .iter()
            .map(Verification::from)
            .collect::<Vec<Verification>>();
        let not_implemented = get_not_implemented_verifications_id(period, &CONFIG);
        for v in res.iter_mut().filter(|v| not_implemented.contains(&v.id)) {
            v.status = VerificationStatus::NotImplemented
        }
        Self(res)
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn get_verifications(is_tally: bool) -> VerificationListPayload {
    let p = match is_tally {
        true => VerificationPeriod::Tally,
        false => VerificationPeriod::Setup,
    };
    VerificationListPayload::from_medata_list(
        p,
        &VerificationMetaDataList::load_period(&CONFIG.verification_list_path(), &p).unwrap(),
    )
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("verifications")
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![get_verifications])
        .build()
}
