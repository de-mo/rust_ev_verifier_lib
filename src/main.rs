pub mod command;
pub mod crypto_primitives;
pub mod data_structures;
pub mod error;
pub mod file_structure;
pub mod runner;
pub mod verification;

use command::execute_command;

fn main() {
    execute_command();
}
