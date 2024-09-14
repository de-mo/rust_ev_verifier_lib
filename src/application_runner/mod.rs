//! Module implementing common functionalities for all Verifier applications (console and GUI)

mod checks;
mod runner;
pub use runner::{no_action_after_fn, no_action_before_fn, RunParallel, Runner};
