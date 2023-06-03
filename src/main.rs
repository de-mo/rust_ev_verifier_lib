pub mod command;
pub mod constants;
pub mod crypto_primitives;
pub mod data_structures;
pub mod file_structure;
pub mod runner;
pub mod setup_or_tally;
pub mod verification;

use command::execute_command;

fn main() {
    execute_command();
}
