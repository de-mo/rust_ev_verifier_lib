//! Shell command implementation
//!
//! For help:
//! ```shell
//! rust_verifier --help
//! ```
use super::runner::Runner;
use crate::{
    constants::LOG_PATH,
    verification::{
        meta_data::{VerificationMetaDataList, VerificationMetaDataListTrait},
        VerificationPeriod,
    },
};
use log::{info, warn, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
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
    setup(VerifierSubCommand),

    #[structopt()]
    /// Tally Verification
    /// Verify the tally configuration
    tally(VerifierSubCommand),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Verifier", version = "0.0.1", author = "Denis Morel")]
/// E-Voting Verifier
/// Verifier for E-Voting System of Swiss Post
struct VerifiyCommand {
    #[structopt(subcommand)]
    sub: Option<SubCommands>,
}

/// Init the logger with or without stdout
fn init_logger(level: LevelFilter, with_stdout: bool) {
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}{n}")))
        .build(LOG_PATH)
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

pub fn execute_command() {
    let command = VerifiyCommand::from_args();
    println!("command: {:?}", command);
    match command.sub {
        Some(cmd) => {
            init_logger(LevelFilter::Debug, true);
            match cmd {
                SubCommands::setup(c) => {
                    info!("Start Verifier for setup");
                    execute_runner(&VerificationPeriod::Setup, &c);
                }
                SubCommands::tally(c) => {
                    info!("Start Verifier for tally");
                    execute_runner(&VerificationPeriod::Setup, &c);
                }
            };
        }
        None => {
            init_logger(LevelFilter::Debug, false);
            info!("Start GUI Verifier");
            warn!("Not Implemented yet");
        }
    }
    info!("Verifier finished");
}

fn execute_runner(period: &VerificationPeriod, cmd: &VerifierSubCommand) {
    let metadata = VerificationMetaDataList::load().unwrap();
    let mut runner = Runner::new(&cmd.dir, period, &metadata, &cmd.exclude);
    runner.run_all_sequential(&metadata);
}
