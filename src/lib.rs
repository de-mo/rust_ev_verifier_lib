//! Library for all the functionalities of the E-Voting Verifier

pub mod application_runner;
mod config;
mod consts;
mod data_structures;
pub mod dataset;
pub mod direct_trust;
pub mod file_structure;
mod resources;
pub mod verification;

pub use config::VerifierConfig;
pub use consts::{ENV_TXT_REPORT_TAB_SIZE, ENV_VERIFIER_DATASET_PASSWORD};
