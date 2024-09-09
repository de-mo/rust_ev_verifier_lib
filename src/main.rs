//! Shell command implementation
//!
//! For help:
//! ```shell
//! rust_verifier_console --help
//! ```

mod application_runner;
mod config;
mod consts;
mod data_structures;
mod dataset;
mod direct_trust;
mod file_structure;
mod resources;
mod verification;

use anyhow::ensure;
use anyhow::Context;
use application_runner::{
    init_logger, no_action_after_fn, no_action_before_fn, RunParallel, Runner,
};
use config::Config as VerifierConfig;
use dataset::DatasetType;
use lazy_static::lazy_static;
use log::{error, info, LevelFilter};
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use verification::meta_data;
use verification::{meta_data::VerificationMetaDataList, VerificationPeriod};

lazy_static! {
    static ref CONFIG: VerifierConfig = VerifierConfig::new(".");
}

/// Specification of the sub commands tally and setup
#[derive(Debug, PartialEq, StructOpt)]
#[structopt()]
struct VerifierSubCommand {
    #[structopt(short, long, parse(from_os_str))]
    /// Directory where the data are stored
    /// The directory must contains the subdirectory setup and tally
    dir: PathBuf,

    #[structopt(long)]
    /// Exclusion of verifications.
    /// Use the id of the verification. Many separated by blanks. E.g. --exclude 02.02 05.05
    exclude: Vec<String>,
}

/// Enum with the possible subcommands
#[derive(Debug, PartialEq, StructOpt)]
#[structopt()]
enum SubCommands {
    #[structopt()]
    /// Verify the setup configuration
    Setup(VerifierSubCommand),

    #[structopt()]
    /// Verify the tally configuration
    Tally(VerifierSubCommand),

    #[structopt()]
    /// Extraction of the zip
    Extract {
        #[structopt(short, long, parse(from_os_str))]
        input: PathBuf,
        #[structopt(short, long)]
        password: String,
        dataset_type: String,
    },
}

/// Main command
#[derive(Debug, StructOpt)]
#[structopt(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
/// E-Voting Verifier
/// Verifier for E-Voting System of Swiss Post
struct VerifiyCommand {
    #[structopt(subcommand)]
    sub: SubCommands,
}

/// Execute the verifications, starting the runner
fn execute_verifications(
    period: &VerificationPeriod,
    sub_command: &VerifierSubCommand,
) -> anyhow::Result<()> {
    info!("Start Verifier for {}", period);
    let metadata = VerificationMetaDataList::load(CONFIG.get_verification_list_str()).unwrap();
    let mut runner = Runner::new(
        &sub_command.dir,
        period,
        &metadata,
        &sub_command.exclude,
        RunParallel,
        &CONFIG,
        no_action_before_fn,
        no_action_after_fn,
    )
    .context("Error creating the runner")?;
    runner
        .run_all(&metadata)
        .context("error running the tests")?;
    info!("Verifier finished");
    Ok(())
}

/// Execute the verifications, starting the runner
fn execute_extract(input: &Path, password: &str, dataset_type_str: &str) -> anyhow::Result<()> {
    ensure!(
        ["context", "setup", "tally"].contains(&dataset_type_str),
        "not correct dataset type: only context, setup or tally allowed"
    );
    let target_dir = CONFIG.create_dataset_dir_path();
    info!(
        "Start extracting file {}",
        input.as_os_str().to_str().unwrap(),
    );
    let dataset = DatasetType::get_from_context_str_with_inputs(
        dataset_type_str,
        input,
        password,
        &target_dir,
        &CONFIG.zip_temp_dir_path(),
    )?;
    let meta_data = dataset.metadata();
    info!(
        "Successfully extraction of file {} in directory {}. (Fingerprint: {})",
        input.as_os_str().to_str().unwrap(),
        meta_data.extracted_dir_path.as_os_str().to_str().unwrap(),
        meta_data.fingerprint,
    );
    Ok(())
}

/// Execute the command
/// This is the main method called from the console
///
/// # return
/// * Nothing if the execution runs correctly
/// * [anyhow::Result] with the related error by a problem
fn execute_command() -> anyhow::Result<()> {
    match VerifiyCommand::from_args().sub {
        SubCommands::Setup(c) => execute_verifications(&VerificationPeriod::Setup, &c),
        SubCommands::Tally(c) => execute_verifications(&VerificationPeriod::Tally, &c),
        SubCommands::Extract {
            input,
            password,
            dataset_type,
        } => execute_extract(&input, &password, &dataset_type),
    }
}

fn main() {
    init_logger(&CONFIG, LevelFilter::Debug, true);
    if let Err(e) = execute_command() {
        error!("{}", e)
    }
}
