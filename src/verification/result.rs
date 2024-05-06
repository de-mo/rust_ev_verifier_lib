//! Module implementing the errors of the verifications
//!
use std::{error::Error, fmt::Display, sync::Arc};

/// Kind of the event during a verification
#[derive(Debug, Clone)]
pub enum VerificationEventKind {
    Error,
    Failure,
}

/// Enum representing one event (an error or a failure) during the tests
#[derive(Debug, Clone)]
pub struct VerificationEvent {
    kind: VerificationEventKind,
    source: Arc<anyhow::Error>,
    contexts: Vec<String>,
}

/// Struct representing a result of the verification
/// The verification can have many errors and/or many failures
#[derive(Debug)]
pub struct VerificationResult {
    results: Vec<VerificationEvent>,
}

impl VerificationEventKind {
    pub fn is_error(&self) -> bool {
        match self {
            VerificationEventKind::Error => true,
            VerificationEventKind::Failure => false,
        }
    }

    pub fn is_failure(&self) -> bool {
        !self.is_error()
    }
}

impl Display for VerificationEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VerificationEventKind::Error => "Error",
                VerificationEventKind::Failure => "Failure",
            }
        )
    }
}

impl VerificationEvent {
    /// Create a new Event of given type from an error
    pub fn from_error<E>(kind: VerificationEventKind, error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            kind,
            source: Arc::new(anyhow!(error)),
            contexts: vec![],
        }
    }

    /// Create a new Event of given type from [anyhow::Error]
    pub fn from_anyhow_error(kind: VerificationEventKind, error: anyhow::Error) -> Self {
        Self {
            kind,
            source: Arc::new(error),
            contexts: vec![],
        }
    }

    /// Create a new Event of given type from str
    pub fn from_str(kind: VerificationEventKind, str: &str) -> Self {
        Self {
            kind,
            source: Arc::new(anyhow!(str.to_string())),
            contexts: vec![],
        }
    }

    /// Create a new Event of type [VerificationEventKind::Error] from an error
    #[allow(dead_code)]
    pub fn error_from_error<E>(error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::from_error(VerificationEventKind::Error, error)
    }

    /// Create a new Event of type [VerificationEventKind::Error] from [anyhow::Error]
    pub fn error_from_anyhow_error(error: anyhow::Error) -> Self {
        Self::from_anyhow_error(VerificationEventKind::Error, error)
    }

    /// Create a new Event of type [VerificationEventKind::Error] from str
    #[allow(dead_code)]
    pub fn error_from_str(str: &str) -> Self {
        Self::from_str(VerificationEventKind::Error, str)
    }

    /// Create a new Event of type [VerificationEventKind::Failure] from error
    pub fn failure_from_error<E>(error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::from_error(VerificationEventKind::Failure, error)
    }

    /// Create a new Event of type [VerificationEventKind::Failure] from [anyhow::Error]
    pub fn failure_from_anyhow_error(error: anyhow::Error) -> Self {
        Self::from_anyhow_error(VerificationEventKind::Failure, error)
    }

    /// Create a new Event of type [VerificationEventKind::Failure] from str
    #[allow(dead_code)]
    pub fn failure_from_str(str: &str) -> Self {
        Self::from_str(VerificationEventKind::Failure, str)
    }

    /// Add a context to the Verification Event and return a new [VerificationEvent]
    #[allow(dead_code)]
    pub fn add_context<C>(mut self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        self.contexts.push(context.to_string());
        self
    }

    /// Is the event an error (of kind [VerificationEventKind::Error])
    pub fn is_error(&self) -> bool {
        self.kind.is_error()
    }

    /// Is the event a failure (of kind [VerificationEventKind::Failure])
    pub fn is_failure(&self) -> bool {
        self.kind.is_failure()
    }

    /// Source of the event as [anyhow::Error]
    #[allow(dead_code)]
    fn source(&self) -> &anyhow::Error {
        &self.source
    }
}

impl Display for VerificationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = format!("{}: {:#}", self.kind, self.source);
        if !self.contexts.is_empty() {
            res = format!("{} ({})", res, self.contexts.join(" -> "));
        }
        write!(f, "{}", res)
    }
}

impl VerificationResult {
    /// Return `true` if no error and not failure
    pub fn is_ok(&self) -> bool {
        !self.has_errors() && !self.has_failures()
    }

    /// Has the result errors
    pub fn has_errors(&self) -> bool {
        self.results.iter().any(|e| e.is_error())
    }

    /// Has the result failures
    pub fn has_failures(&self) -> bool {
        self.results.iter().any(|e| e.is_failure())
    }

    /// Get the errors
    pub fn errors(&self) -> Vec<&VerificationEvent> {
        self.results.iter().filter(|e| e.is_error()).collect()
    }

    /// Get the failures
    pub fn failures(&self) -> Vec<&VerificationEvent> {
        self.results.iter().filter(|e| e.is_failure()).collect()
    }

    /// Get the errors as string
    pub fn errors_to_string(&self) -> Vec<String> {
        self.errors().iter().map(|e| e.to_string()).collect()
    }

    /// Get the failures as string
    pub fn failures_to_string(&self) -> Vec<String> {
        self.failures().iter().map(|e| e.to_string()).collect()
    }

    /// Get the errors and the failures (all events)
    #[allow(dead_code)]
    pub fn errors_and_failures(&self) -> Vec<&VerificationEvent> {
        self.results.iter().collect()
    }

    /// New VerificationResult
    pub fn new() -> Self {
        VerificationResult { results: vec![] }
    }

    pub fn from_vec(list: Vec<VerificationEvent>) -> Self {
        Self { results: list }
    }

    pub fn from_verification_event(event: VerificationEvent) -> Self {
        Self::from_vec(vec![event])
    }

    pub fn add_context<C>(&self, context: C) -> Self
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        let mut res = Self::new();
        res.append_wtih_context(&self, context);
        res
    }

    /// Push a new error or failure to the VerificationResult
    pub fn push(&mut self, e: VerificationEvent) {
        self.results.push(e)
    }

    /// Push a new error or failure to the VerificationResult add the given context
    #[allow(dead_code)]
    pub fn push_with_context<C>(&mut self, e: VerificationEvent, context: C)
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        self.push(e.add_context(context))
    }

    /// Append the results of ohter to self, emptying the vectors of other
    pub fn append(&mut self, other: &mut Self) {
        self.results.append(&mut other.results);
    }

    /// Append the results of ohter to self with context
    pub fn append_wtih_context<C>(&mut self, other: &Self, context: C)
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        for e in other.errors_and_failures() {
            self.push_with_context(e.clone(), context.clone());
        }
    }

    /// Append the results of ohter to self, emptying the vectors of other
    #[allow(dead_code)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_result() {
        let res = VerificationResult::new();
        assert!(res.is_ok());
        assert!(!res.has_errors());
        assert!(!res.has_failures());
    }

    #[test]
    fn test_verif_event() {
        let event = VerificationEvent::error_from_str("toto")
            .add_context("context")
            .add_context("first");
        assert_eq!(
            event.to_string(),
            "Error: toto (context -> first)".to_string()
        )
    }
}
