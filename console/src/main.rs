//! Shell command implementation
//!
//! For help:
//! ```shell
//! rust_verifier --help
//! ```
use anyhow::bail;
use log::{error, info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use rust_verifier_lib::{
    constants::{log_path, verification_list_path},
    verification::{meta_data::VerificationMetaDataList, VerificationPeriod},
};
use rust_verifier_runner::Runner;
use rust_verifier_runner::{check_verification_dir, start_check};
use std::path::PathBuf;
use structopt::StructOpt;

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

/// Init the logger with or without stdout
fn init_logger(level: LevelFilter, with_stdout: bool) {
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}{n}")))
        .build(log_path(None))
        .unwrap();

    let mut root_builder = Root::builder().appender("file");
    let mut config_builder =
        Config::builder().appender(Appender::builder().build("file", Box::new(file)));

    if with_stdout {
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{h({l})} - {m}{n}")))
            .build();
        root_builder = root_builder.appender("stdout");
        config_builder =
            config_builder.appender(Appender::builder().build("stdout", Box::new(stdout)));
    }

    let config = config_builder.build(root_builder.build(level)).unwrap();
    let _handle = log4rs::init_config(config).unwrap();
}

fn execute_runner(period: &VerificationPeriod, cmd: &VerifierSubCommand) {
    let metadata = VerificationMetaDataList::load(&verification_list_path(None)).unwrap();
    let mut runner = Runner::new(&cmd.dir, period, &metadata, &cmd.exclude);
    runner.run_all_sequential(&metadata);
}

fn execute_verifier() -> anyhow::Result<()> {
    if let Err(e) = start_check() {
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
    init_logger(LevelFilter::Debug, true);
    if let Err(e) = execute_verifier() {
        error!("{}", e)
    }
}
