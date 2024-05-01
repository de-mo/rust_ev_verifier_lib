//! Module implementing the errors of the verifications
//!
use std::{any, error::Error, fmt::Display, sync::Arc};

pub enum VerificationEventKind {
    Error,
    Failure,
}

/// Enum representing one event (an error or a failure) during the tests
#[derive(Debug)]
enum VerificationEventImpl {
    Error(Arc<anyhow::Error>),
    Failure(Arc<anyhow::Error>),
}

#[derive(Debug)]
pub struct VerificationEvent {
    value: VerificationEventImpl,
}

/// Struct representing a result of the verification
/// The verification can have many errors and/or many failures
#[derive(Debug)]
pub struct VerificationResult {
    results: Vec<VerificationEvent>,
}

/// Trait defining functions to access the verficiation result
pub trait VerificationResultTrait {
    /// Check if the verification results are valid, and can be used
    ///
    /// If not, it can mean, that the verifications have not run
    fn is_valid(&self) -> bool;

    /// Is the verification ok ?
    ///
    /// If not run, the output is None
    fn is_ok(&self) -> bool;

    /// Has the verification errors ?
    ///
    /// If not run, the output is None
    fn has_errors(&self) -> bool;

    /// Has the verification failures ?
    ///
    /// If not run, the output is None
    fn has_failures(&self) -> bool;

    /// All errors and failures
    fn errors_and_failures(&self) -> Vec<&VerificationEvent>;

    /// All the errors
    fn errors(&self) -> Vec<&VerificationEvent>;

    fn errors_to_string(&self) -> Vec<String>;

    /// All the failures
    fn failures(&self) -> Vec<&VerificationEvent>;

    fn failures_to_string(&self) -> Vec<String>;

    // Mutable reference to the errors
    //fn errors_mut(&mut self) -> &mut Vec<VerificationEvent>;

    // Mutable reference to the failures
    //fn failures_mut(&mut self) -> &mut Vec<VerificationEvent>;
}

impl VerificationEvent {
    /// Create a new Error Event from an error
    pub fn from_error<E>(kind: VerificationEventKind, error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        match kind {
            VerificationEventKind::Error => Self {
                value: VerificationEventImpl::Error(Arc::new(anyhow::anyhow!(error))),
            },
            VerificationEventKind::Failure => Self {
                value: VerificationEventImpl::Failure(Arc::new(anyhow::anyhow!(error))),
            },
        }
    }

    /// Create a new Error Event from an error
    pub fn from_anyhow_error(kind: VerificationEventKind, error: anyhow::Error) -> Self {
        match kind {
            VerificationEventKind::Error => Self {
                value: VerificationEventImpl::Error(Arc::new(error)),
            },
            VerificationEventKind::Failure => Self {
                value: VerificationEventImpl::Failure(Arc::new(error)),
            },
        }
    }

    /// Create a new Error Event from a string
    pub fn from_str(kind: VerificationEventKind, str: &str) -> Self {
        match kind {
            VerificationEventKind::Error => Self {
                value: VerificationEventImpl::Error(Arc::new(anyhow::anyhow!(str.to_string()))),
            },
            VerificationEventKind::Failure => Self {
                value: VerificationEventImpl::Failure(Arc::new(anyhow::anyhow!(str.to_string()))),
            },
        }
    }

    pub fn error_from_error<E>(error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::from_error(VerificationEventKind::Error, error)
    }

    pub fn error_from_anyhow_error(error: anyhow::Error) -> Self {
        Self::from_anyhow_error(VerificationEventKind::Error, error)
    }

    pub fn error_from_str(str: &str) -> Self {
        Self::from_str(VerificationEventKind::Error, str)
    }

    pub fn failure_from_error<E>(error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::from_error(VerificationEventKind::Failure, error)
    }

    pub fn failure_from_anyhow_error(error: anyhow::Error) -> Self {
        Self::from_anyhow_error(VerificationEventKind::Failure, error)
    }

    pub fn failure_from_str(str: &str) -> Self {
        Self::from_str(VerificationEventKind::Failure, str)
    }

    /// Add a context to the Verification Event and return a new [VerificationEvent]
    pub fn add_context<C>(&mut self, context: C)
    where
        C: Display + Send + Sync + 'static,
    {
        match &self.value {
            VerificationEventImpl::Error(source) => {
                self.value =
                    VerificationEventImpl::Error(Arc::new(anyhow!(source.clone()).context(context)))
            }
            VerificationEventImpl::Failure(source) => {
                self.value =
                    VerificationEventImpl::Error(Arc::new(anyhow!(source.clone()).context(context)))
            }
        }
    }

    pub fn new_with_context<C>(&self, context: C) -> Self
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        match &self.value {
            VerificationEventImpl::Error(source) => Self::from_anyhow_error(
                VerificationEventKind::Error,
                anyhow!(source.clone()).context(context),
            ),
            VerificationEventImpl::Failure(source) => Self::from_anyhow_error(
                VerificationEventKind::Failure,
                anyhow!(source.clone()).context(context),
            ),
        }
    }

    pub fn is_error(&self) -> bool {
        match self.value {
            VerificationEventImpl::Error(_) => true,
            VerificationEventImpl::Failure(_) => false,
        }
    }

    pub fn is_failure(&self) -> bool {
        !self.is_error()
    }

    fn source(&self) -> &anyhow::Error {
        match &self.value {
            VerificationEventImpl::Error(source) => source,
            VerificationEventImpl::Failure(source) => source,
        }
    }
}

impl ToString for VerificationEvent {
    fn to_string(&self) -> String {
        match &self.value {
            VerificationEventImpl::Error(e) => e.to_string(),
            VerificationEventImpl::Failure(e) => e.to_string(),
        }
    }
}

impl VerificationResult {
    /// New VerificationResult
    pub fn new() -> Self {
        VerificationResult { results: vec![] }
    }

    pub fn from_vec(list: Vec<VerificationEvent>) -> Self {
        Self { results: list }
    }

    /// Push a new error or failure to the VerificationResult
    pub fn push(&mut self, e: VerificationEvent) {
        self.results.push(e)
    }

    /// Push a new error or failure to the VerificationResult add the given context
    pub fn push_with_context<C>(&mut self, e: VerificationEvent, context: C)
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        self.push(e.new_with_context(context))
    }

    /// Append the results of ohter to self, emptying the vectors of other
    pub fn append(&mut self, other: &mut Self) {
        self.results.append(&mut other.results);
    }

    /// Append the results of ohter to self, emptying the vectors of other
    pub fn append_vec(&mut self, other: &mut Vec<VerificationEvent>) {
        self.results.append(other);
    }

    /// Append strings to self as errors
    #[allow(dead_code)]
    pub fn append_errors_from_string(&mut self, errors: &[String]) {
        let events: Vec<VerificationEvent> = errors
            .iter()
            .map(|e| VerificationEvent::from_str(VerificationEventKind::Error, e.as_str()))
            .collect();
        for e in events {
            self.push(e)
        }
    }

    /// Append strings to self as failures
    #[allow(dead_code)]
    pub fn append_failures_from_string(&mut self, failures: &[String]) {
        let events: Vec<VerificationEvent> = failures
            .iter()
            .map(|e| VerificationEvent::from_str(VerificationEventKind::Failure, e.as_str()))
            .collect();
        for e in events {
            self.push(e)
        }
    }
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationResultTrait for VerificationResult {
    fn is_valid(&self) -> bool {
        true
    }

    fn is_ok(&self) -> bool {
        !self.has_errors() && !self.has_failures()
    }

    fn has_errors(&self) -> bool {
        self.results.iter().any(|e| e.is_error())
    }

    fn has_failures(&self) -> bool {
        self.results.iter().any(|e| e.is_failure())
    }

    fn errors(&self) -> Vec<&VerificationEvent> {
        self.results.iter().filter(|e| e.is_error()).collect()
    }

    fn failures(&self) -> Vec<&VerificationEvent> {
        self.results.iter().filter(|e| e.is_failure()).collect()
    }

    /*
       fn errors_mut(&mut self) -> &mut Vec<VerificationEvent> {
           &mut self.errors
       }

       fn failures_mut(&mut self) -> &mut Vec<VerificationEvent> {
           &mut self.failures
       }
    */
    fn errors_to_string(&self) -> Vec<String> {
        self.errors().iter().map(|e| e.to_string()).collect()
    }

    fn failures_to_string(&self) -> Vec<String> {
        self.failures().iter().map(|e| e.to_string()).collect()
    }

    fn errors_and_failures(&self) -> Vec<&VerificationEvent> {
        self.results.iter().collect()
    }
}

/// Macro to create a verification error (with or without embedded error)
macro_rules! create_verification_error {
    ($m: expr) => {{
        let e = anyhow!($m);
        debug!("{}", format!("Error: {}", e));
        VerificationEvent::error_from_anyhow_error(e)
    }};
    ($m: expr, $e: expr) => {{
        let e = anyhow!($e).context($m);
        debug!("{}", format!("Error: {}", e));
        VerificationEvent::error_from_anyhow_error(e)
    }};
}
use anyhow::anyhow;
pub(crate) use create_verification_error;

/// Macro to create a verification failure (with or without embedded error)
macro_rules! create_verification_failure {
    ($m: expr) => {{
        let e = anyhow!($m);
        debug!("{}", format!("Failure: {}", e));
        VerificationEvent::failure_from_anyhow_error(e)
    }};
    ($m: expr, $e: expr) => {{
        let e = anyhow!($e).context($m);
        debug!("{}", format!("Failure: {}", e));
        VerificationEvent::failure_from_anyhow_error(e)
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
