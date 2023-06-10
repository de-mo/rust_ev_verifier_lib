//! Module implementing the errors of the verifications
//!
//use crate::error::VerifierError;
use thiserror::Error;

/// Enum representing one event (an error or a failure) during the tests
#[derive(Error, Debug)]
pub enum VerificationEvent {
    #[error(transparent)]
    Error { source: anyhow::Error },
    #[error(transparent)]
    Failure { source: anyhow::Error },
}

/// Struct representing a result of the verification
/// The verification can have many errors and/or many failures
#[derive(Debug)]
pub struct VerificationResult {
    errors: Vec<VerificationEvent>,
    failures: Vec<VerificationEvent>,
}

/// Trait defining functions to access the verficiation result
pub trait VerificationResultTrait {
    /// Is the verification ok ?
    ///
    /// If not run, the output is None
    fn is_ok(&self) -> Option<bool>;

    /// Has the verification errors ?
    ///
    /// If not run, the output is None
    fn has_errors(&self) -> Option<bool>;

    /// Has the verification failures ?
    ///
    /// If not run, the output is None
    fn has_failures(&self) -> Option<bool>;

    /// All the errors
    fn errors(&self) -> &Vec<VerificationEvent>;

    /// All the failures
    fn failures(&self) -> &Vec<VerificationEvent>;

    /// Mutable reference to the errors
    fn errors_mut(&mut self) -> &mut Vec<VerificationEvent>;

    /// Mutable reference to the failures
    fn failures_mut(&mut self) -> &mut Vec<VerificationEvent>;
}

impl VerificationResult {
    /// New VerificationResult
    pub fn new() -> Self {
        VerificationResult {
            errors: vec![],
            failures: vec![],
        }
    }

    /// Push a new error or failure to the VerificationResult
    pub fn push(&mut self, e: VerificationEvent) {
        match &e {
            VerificationEvent::Error { source: _ } => self.errors.push(e),
            VerificationEvent::Failure { source: _ } => self.failures.push(e),
        }
    }

    /// Append the results of ohter to self, emptying the vectors of other
    pub fn append(&mut self, other: &mut Self) {
        self.errors.append(other.errors_mut());
        self.failures.append(other.failures_mut());
    }
}

impl VerificationResultTrait for VerificationResult {
    fn is_ok(&self) -> Option<bool> {
        Some(!self.has_errors().unwrap() && !self.has_failures().unwrap())
    }

    fn has_errors(&self) -> Option<bool> {
        Some(!self.errors.is_empty())
    }

    fn has_failures(&self) -> Option<bool> {
        Some(!self.failures.is_empty())
    }

    fn errors(&self) -> &Vec<VerificationEvent> {
        &self.errors
    }

    fn failures(&self) -> &Vec<VerificationEvent> {
        &self.failures
    }

    fn errors_mut(&mut self) -> &mut Vec<VerificationEvent> {
        &mut self.errors
    }

    fn failures_mut(&mut self) -> &mut Vec<VerificationEvent> {
        &mut self.failures
    }
}

/// Macro to create a verification error (with or without embedded error)
macro_rules! create_verification_error {
    ($m: expr) => {{
        let e = anyhow!($m);
        debug!("{}", format!("Error: {}", e));
        VerificationEvent::Error { source: e }
    }};
    ($m: expr, $e: expr) => {{
        let e = anyhow!($e).context($m);
        debug!("{}", format!("Error: {}", e));
        VerificationEvent::Error { source: e }
    }};
}
pub(crate) use create_verification_error;

/// Macro to create a verification failure (with or without embedded error)
macro_rules! create_verification_failure {
    ($m: expr) => {{
        let e = anyhow!($m);
        debug!("{}", format!("Failure: {}", e));
        VerificationEvent::Failure { source: e }
    }};
    ($m: expr, $e: expr) => {{
        let e = anyhow!($e).context($m);
        debug!("{}", format!("Failure: {}", e));
        VerificationEvent::Failure { source: e }
    }};
}
pub(crate) use create_verification_failure;

/*
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationErrorType {
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailureType {
    Failure,
}

impl Display for VerificationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Error => "Error on test",
        };
        write!(f, "{s}")
    }
}

impl Display for VerificationFailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Failure => "Failure on test",
        };
        write!(f, "{s}")
    }
}


pub type VerificationError = VerifierError<VerificationErrorType>;
pub type VerificationFailure = VerifierError<VerificationFailureType>;
 */
