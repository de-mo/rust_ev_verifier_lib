//! Module implementing the errors of the verifications
//!
use std::fmt::Display;
use strum::AsRefStr;

/// Kind of the event during a verification
#[derive(Debug, Clone, AsRefStr)]
pub enum VerificationEventKind {
    Error,
    Failure,
}

/// Enum representing one event (an error or a failure) during the tests
#[derive(Debug, Clone)]
pub struct VerificationEvent {
    kind: VerificationEventKind,
    source: String,
    contexts: Vec<String>,
}

/// Struct representing a result of the verification
/// The verification can have many errors and/or many failures
#[derive(Clone, Debug)]
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

impl VerificationEvent {
    pub fn new<T: Display + ?Sized>(kind: VerificationEventKind, value: &T) -> Self {
        Self {
            kind,
            source: format!("{}", value),
            contexts: vec![],
        }
    }

    pub fn new_error<T: Display + ?Sized>(value: &T) -> Self {
        Self::new(VerificationEventKind::Error, value)
    }

    pub fn new_failure<T: Display + ?Sized>(value: &T) -> Self {
        Self::new(VerificationEventKind::Failure, value)
    }

    /// Add a context to the Verification Event
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
    fn source(&self) -> &str {
        self.source.as_str()
    }
}

impl Display for VerificationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = format!("{}: {}", self.kind.as_ref(), self.source);
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

    /// Add the context to the contexts of self
    pub fn add_context<C>(&mut self, context: C)
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        self.results
            .iter_mut()
            .for_each(|event| event.contexts.push(context.to_string()));
    }

    /// Clone self and add the context
    pub fn clone_add_context<C>(self, context: C) -> Self
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        let mut res = self.clone();
        res.add_context(context);
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
    pub fn append_with_context<C>(&mut self, other: &Self, context: C)
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
            .map(|e| VerificationEvent::new(VerificationEventKind::Error, &e.as_str()))
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
            .map(|e| VerificationEvent::new(VerificationEventKind::Failure, &e.as_str()))
            .collect();
        for e in events {
            self.push(e)
        }
    }
}

impl From<&VerificationEvent> for VerificationResult {
    fn from(value: &VerificationEvent) -> Self {
        VerificationResult::from(vec![value.clone()].as_slice())
    }
}

impl From<&[VerificationEvent]> for VerificationResult {
    fn from(value: &[VerificationEvent]) -> Self {
        Self {
            results: value.iter().cloned().collect::<Vec<_>>(),
        }
    }
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self::new()
    }
}

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
        let event = VerificationEvent::new_error("toto")
            .add_context("context")
            .add_context("first");
        assert_eq!(
            event.to_string(),
            "Error: toto (context -> first)".to_string()
        )
    }
}
