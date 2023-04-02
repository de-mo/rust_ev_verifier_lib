use clap::{Arg, ArgAction, ArgMatches, Command};
use log::{info, warn, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

fn get_verifier_subcommand(
    name: &'static str,
    long_flag: &'static str,
    about: &'static str,
) -> Command {
    Command::new(name)
        .long_flag(long_flag)
        .about(about)
        .arg(
            Arg::new("dir")
                .short('d')
                .long("dir")
                .help("Directory where the data are stored")
                .action(ArgAction::Set)
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .help("Exclusion")
                .action(ArgAction::Set)
                .num_args(1..),
        )
}

fn get_command() -> ArgMatches {
    Command::new("Verifier")
        .about("Verifier for E-Voting System of Swiss Post")
        .version("0.0.1")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .author("Denis Morel")
        // Query subcommand
        .subcommand(get_verifier_subcommand("setup", "setup", "Verifiy Setup"))
        .subcommand(get_verifier_subcommand("tally", "tally", "Verifiy tally"))
        .get_matches()
}

fn init_logger(level: LevelFilter, with_stdout: bool) {
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}{n}")))
        .build("log/log.txt")
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
    let matches = get_command();
    match matches.subcommand() {
        None => {
            init_logger(LevelFilter::Debug, false);
            info!("Start GUI Verifier");
            warn!("Not Implemented yet");
        }
        Some(("setup", setup_matches)) => {
            init_logger(LevelFilter::Debug, true);
            info!("Start Verifier for setup");
            //println!("Setup: {:?}", setup_matches)
        }
        Some(("tally", tally_matches)) => {
            init_logger(LevelFilter::Debug, true);
            info!("Start Verifier for tally");
            //println!("tally: {:?}", tally_matches)
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
    info!("Verifier finished");
}
