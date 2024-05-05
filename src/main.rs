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
mod direct_trust;
mod file_structure;
mod resources;
mod verification;
use application_runner::{
    init_logger, no_action_after_fn, no_action_before_fn, RunParallel, Runner,
};
use config::Config as VerifierConfig;
use lazy_static::lazy_static;
use log::{error, info, LevelFilter};
use std::path::PathBuf;
use structopt::StructOpt;
use verification::{meta_data::VerificationMetaDataList, VerificationPeriod};

lazy_static! {
    static ref CONFIG: VerifierConfig = VerifierConfig::new(".");
}

/// Specification of the sub commands (tally or setup)
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
    /// Setup Verification
    /// Verify the setup configuration
    Setup(VerifierSubCommand),

    #[structopt()]
    /// Tally Verification
    /// Verify the tally configuration
    Tally(VerifierSubCommand),
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

impl From<&SubCommands> for VerificationPeriod {
    fn from(value: &SubCommands) -> Self {
        match value {
            SubCommands::Setup(_) => VerificationPeriod::Setup,
            SubCommands::Tally(_) => VerificationPeriod::Tally,
        }
    }
}

impl SubCommands {
    fn verifier_sub_command(&self) -> &VerifierSubCommand {
        match self {
            SubCommands::Setup(c) => c,
            SubCommands::Tally(c) => c,
        }
    }
}

/// Execute the verifier
/// This is the main method called from the console
///
/// # return
/// * Nothing if the execution runs correctly
/// * [anyhow::Result] with the related error by a problem
fn execute_verifier() -> anyhow::Result<()> {
    let command = VerifiyCommand::from_args();
    let period = VerificationPeriod::from(&command.sub);
    let sub_command = command.sub.verifier_sub_command();
    info!("Start Verifier for {}", period);
    let metadata = VerificationMetaDataList::load(CONFIG.get_verification_list_str()).unwrap();
    let mut runner = Runner::new(
        &sub_command.dir,
        &period,
        &metadata,
        &sub_command.exclude,
        RunParallel,
        &CONFIG,
        no_action_before_fn,
        no_action_after_fn,
    )
    .map_err(|e| e.context("Error creating the runner"))?;
    runner
        .run_all(&metadata)
        .map_err(|e| e.context("error running the tests"))?;
    info!("Verifier finished");
    Ok(())
}

fn main() {
    init_logger(&CONFIG, LevelFilter::Debug, true);
    if let Err(e) = execute_verifier() {
        error!("{}", e)
    }
}
