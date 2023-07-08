//! Shell command implementation
//!
//! For help:
//! ```shell
//! rust_verifier --help
//! ```
use anyhow::bail;
use lazy_static::lazy_static;
use log::{error, info, LevelFilter};
use rust_verifier_application::{
    check_verification_dir, init_logger, start_check, RunSequential, Runner,
    VerificationMetaDataList, VerificationPeriod, VerifierConfig,
};
use std::path::PathBuf;
use structopt::StructOpt;

lazy_static! {
    static ref CONFIG: VerifierConfig = VerifierConfig::new(".");
}

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

fn execute_runner(period: &VerificationPeriod, cmd: &VerifierSubCommand) {
    let metadata = VerificationMetaDataList::load(&CONFIG.verification_list_path()).unwrap();
    let mut runner = Runner::new(
        &cmd.dir,
        period,
        &metadata,
        &cmd.exclude,
        RunSequential,
        &CONFIG,
    );
    runner.run_all(&metadata);
}

fn execute_verifier() -> anyhow::Result<()> {
    if let Err(e) = start_check(&CONFIG) {
        bail!("Application cannot start: {}", e);
    };
    let command = VerifiyCommand::from_args();
    let period = VerificationPeriod::from(&command.sub);
    let sub_command = command.sub.verifier_sub_command();
    info!("Start Verifier for {}", period);
    if let Err(e) = check_verification_dir(&period, &sub_command.dir) {
        bail!("Application cannot start: {}", e);
    } else {
        execute_runner(&period, sub_command);
    }
    info!("Verifier finished");
    Ok(())
}

fn main() {
    init_logger(&CONFIG, LevelFilter::Debug, true);
    if let Err(e) = execute_verifier() {
        error!("{}", e)
    }
}
