use clap::{Arg, ArgAction, ArgMatches, Command};

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

pub fn execute_command() {
    let matches = get_command();
    match matches.subcommand() {
        None => {
            println!("Start GUI Verifier. Not Implemented yet")
        }
        Some(("setup", setup_matches)) => {
            println!("Setup: {:?}", setup_matches)
        }
        Some(("tally", tally_matches)) => {
            println!("tally: {:?}", tally_matches)
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
}
