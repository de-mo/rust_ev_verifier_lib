use std::{path::Path, sync::Mutex};

use super::CONFIG;
use lazy_static::lazy_static;
use log::debug;
use rust_verifier_application::{
    get_not_implemented_verifications_id, RunParallel, Runner, VerificationMetaData,
    VerificationMetaDataList, VerificationPeriod,
};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[derive(Clone, Copy, Debug, serde::Serialize, PartialEq, Eq)]
enum VerificationStatus {
    #[serde(rename = "Not started")]
    NotStarted,
    Waiting,
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

impl VerificationListPayload {
    pub fn as_slice(&self) -> &[Verification] {
        &self.0
    }

    pub fn push(&mut self, v: Verification) {
        self.0.push(v)
    }

    pub fn update(&mut self, v: &Verification) {
        let v_mut = match self.0.iter_mut().find(|e| v.id == e.id) {
            Some(e) => e,
            None => return,
        };
        v_mut.clone_from(v);
    }

    pub fn empty(&mut self) {
        self.0 = vec![]
    }
}

lazy_static! {
    static ref VERIFICATIONS: Mutex<VerificationListPayload> =
        Mutex::new(VerificationListPayload(vec![]));
}

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
    debug!("Loading verifications for period {}", p);
    let list = VerificationListPayload::from_medata_list(
        p,
        &VerificationMetaDataList::load_period(&CONFIG.verification_list_path(), &p).unwrap(),
    );
    let mut mlist = VERIFICATIONS.lock().unwrap();
    mlist.empty();
    for v in list.as_slice() {
        mlist.push(v.clone())
    }
    println!(
        "Number of verifications in get_verifications: {}",
        list.0.len()
    );
    list
}

#[tauri::command(rename_all = "snake_case")]
async fn update_verifications() -> VerificationListPayload {
    let res = VERIFICATIONS.lock().unwrap().clone();
    println!(
        "Number after clone in update_verifications: {}",
        res.0.len()
    );
    res
}

fn verification_start(id: &str) {
    let mut v = match VerificationMetaData::from_id(id, &CONFIG.verification_list_path()) {
        Some(v) => Verification::from(&v),
        None => return,
    };
    v.status = VerificationStatus::Running;
    VERIFICATIONS.lock().unwrap().update(&v);
}

fn verification_finished(id: &str, errors: Vec<String>, failures: Vec<String>) {
    let mut v = match VerificationMetaData::from_id(id, &CONFIG.verification_list_path()) {
        Some(v) => Verification::from(&v),
        None => return,
    };
    let mut status = VerificationStatus::Successful;
    if !errors.is_empty() {
        status = VerificationStatus::Errors
    };
    if !failures.is_empty() {
        status = match status {
            VerificationStatus::Errors => VerificationStatus::ErrorsAndFailures,
            _ => VerificationStatus::Failures,
        }
    };
    v.status = status;
    v.errors = errors;
    v.failures = failures;
    VERIFICATIONS.lock().unwrap().update(&v)
}

#[tauri::command(rename_all = "snake_case")]
async fn run_all(dir: String, is_tally: bool, exclusions: Vec<String>) {
    VERIFICATIONS
        .lock()
        .unwrap()
        .0
        .iter_mut()
        .filter(|v| !exclusions.contains(&v.id))
        .filter(|v| v.status != VerificationStatus::NotImplemented)
        .for_each(|v| v.status = VerificationStatus::Waiting);
    let metadata = VerificationMetaDataList::load(&CONFIG.verification_list_path()).unwrap();
    let mut runner = Runner::new(
        Path::new(&dir),
        &match is_tally {
            true => VerificationPeriod::Tally,
            false => VerificationPeriod::Setup,
        },
        &metadata,
        &exclusions,
        RunParallel,
        &CONFIG,
        verification_start,
        verification_finished,
    );
    runner.run_all(&metadata);
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("verifications")
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            get_verifications,
            run_all,
            update_verifications
        ])
        .build()
}
