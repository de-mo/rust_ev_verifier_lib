// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

//! Module implementing the errors of the verifications
//!
use std::{collections::HashMap, fmt::Display};
use strum::AsRefStr;

use crate::ErrorChain;

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
    results: Vec<String>,
}

/// Struct representing a result of the verification
/// The verification can have many errors and/or many failures
#[derive(Clone, Debug)]
pub struct VerificationResult {
    results: Vec<VerificationEvent>,
}

/// Type representing verifications with errors and failures
#[derive(Clone, Debug, Default)]
pub struct VerficationsWithErrorAndFailures(HashMap<String, (Vec<String>, Vec<String>)>);

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
            results: vec![format!("{}", value)],
        }
    }

    pub fn new_from_error<E: std::error::Error + Display>(
        kind: VerificationEventKind,
        error: &E,
    ) -> Self {
        let chain = ErrorChain::new(error);
        let mut values = chain.map(|e| e.to_string()).collect::<Vec<_>>();
        values.reverse();
        Self {
            kind,
            results: values,
        }
    }

    pub fn new_error_from_error<E: std::error::Error + Display>(error: &E) -> Self {
        Self::new_from_error(VerificationEventKind::Error, error)
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
        self.results.push(context.to_string());
        self
    }

    /// Is the event an error)
    pub fn is_error(&self) -> bool {
        self.kind.is_error()
    }

    /// Is the event a failure)
    pub fn is_failure(&self) -> bool {
        self.kind.is_failure()
    }

    /// Source of the event
    pub fn source(&self) -> &str {
        match self.results.first() {
            Some(s) => s.as_str(),
            None => "",
        }
    }

    /// Last comment of the event (the main element of the event)
    pub fn last(&self) -> &str {
        match self.results.last() {
            Some(s) => s.as_str(),
            None => "",
        }
    }

    /// Contexts of the event
    pub fn contexts(&self) -> Vec<&str> {
        self.results.iter().skip(1).map(|s| s.as_str()).collect()
    }
}

impl VerficationsWithErrorAndFailures {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_errors(&self, id: &str) -> Option<bool> {
        self.0.get(id).map(|(errors, _)| !errors.is_empty())
    }

    pub fn has_failures(&self, id: &str) -> Option<bool> {
        self.0.get(id).map(|(_, failures)| !failures.is_empty())
    }

    pub fn number_of_verifications_with_errors(&self) -> usize {
        self.0
            .values()
            .filter(|(errors, _)| !errors.is_empty())
            .count()
    }

    pub fn number_of_verifications_with_failures(&self) -> usize {
        self.0
            .values()
            .filter(|(_, failures)| !failures.is_empty())
            .count()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &(Vec<String>, Vec<String>))> {
        self.0.iter()
    }

    pub fn insert<S: Into<String>>(&mut self, id: S, errors: Vec<String>, failures: Vec<String>) {
        self.0.insert(id.into(), (errors, failures));
    }
}

impl Display for VerificationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = vec![format!("{}: {}", self.kind.as_ref(), self.last())];
        if self.results.len() > 1 {
            res.push("backtrace:".to_string());
            res.append(
                &mut self
                    .results
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("{i}: {s}"))
                    .collect(),
            );
        }
        write!(f, "{}", res.join("\n"))
    }
}

impl Extend<VerificationEvent> for VerificationResult {
    fn extend<T: IntoIterator<Item = VerificationEvent>>(&mut self, iter: T) {
        for elt in iter {
            self.push(elt);
        }
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
    pub fn errors_and_failures(&self) -> Vec<&VerificationEvent> {
        self.results.iter().collect()
    }

    /// New VerificationResult
    pub fn new() -> Self {
        Self { results: vec![] }
    }

    /// Add the context to the contexts of self
    pub fn add_context<C>(&mut self, context: C)
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        self.results
            .iter_mut()
            .for_each(|event| event.results.push(context.to_string()));
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
    pub fn append_vec(&mut self, other: &mut Vec<VerificationEvent>) {
        self.results.append(other);
    }

    /// Append strings to self as errors
    pub fn append_errors_from_string_slice(&mut self, errors: &[String]) {
        let events: Vec<VerificationEvent> = errors
            .iter()
            .map(|e| VerificationEvent::new(VerificationEventKind::Error, &e.as_str()))
            .collect();
        for e in events {
            self.push(e)
        }
    }

    /// Append strings to self as failures
    pub fn append_failures_from_string_slice(&mut self, failures: &[String]) {
        let events: Vec<VerificationEvent> = failures
            .iter()
            .map(|e| VerificationEvent::new(VerificationEventKind::Failure, &e.as_str()))
            .collect();
        for e in events {
            self.push(e)
        }
    }

    /// Create new [Self] with errors from strings
    pub fn new_errors_from_string_slice(errors: &[String]) -> Self {
        let mut res = Self::new();
        res.append_errors_from_string_slice(errors);
        res
    }

    /// Create new [Self] with failures from strings
    pub fn new_failures_from_string_slice(errors: &[String]) -> Self {
        let mut res = Self::new();
        res.append_failures_from_string_slice(errors);
        res
    }

    /// Join a &[Self] with context
    pub fn join_with_context<C>(data: &[Self], context: C) -> Self
    where
        C: Clone + Display + Send + Sync + 'static,
    {
        let mut res = Self::new();
        for d in data.iter() {
            res.append_with_context(d, context.clone());
        }
        res
    }

    /// Join a &[Self]
    pub fn join(data: &[Self]) -> Self {
        let mut res = Self::new();
        for d in data.iter() {
            for e in d.results.iter() {
                res.push(e.clone());
            }
        }
        res
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
            results: value.to_vec(),
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
    use thiserror::Error;

    #[derive(Error, Debug)]
    #[error("inner 21 error")]
    struct Inner21 {}

    #[derive(Error, Debug)]
    #[error("inner 22 error")]
    struct Inner22 {}

    #[derive(Error, Debug)]
    enum Inner {
        #[error("Context 21")]
        Inner21 { source: Inner21 },
        #[error("Context 22")]
        Inner22 { source: Inner22 },
    }

    #[derive(Error, Debug)]
    enum Outer {
        #[error("Context Inner")]
        Inner { source: Inner },
    }

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
            "Error: first\nbacktrace:\n0: toto\n1: context\n2: first".to_string()
        )
    }

    #[test]
    fn test_from_error() {
        let e = Outer::Inner {
            source: Inner::Inner21 { source: Inner21 {} },
        };
        let event = VerificationEvent::new_from_error(VerificationEventKind::Error, &e);
        assert_eq!(
            event.to_string(),
            "Error: Context Inner\nbacktrace:\n0: inner 21 error\n1: Context 21\n2: Context Inner"
                .to_string()
        );
    }

    #[test]
    fn test_from_error_with_context() {
        let e = Outer::Inner {
            source: Inner::Inner22 { source: Inner22 {} },
        };
        let event = VerificationEvent::new_from_error(VerificationEventKind::Error, &e)
            .add_context("Context 1")
            .add_context("Context 2");
        assert_eq!(
            event.to_string(),
            "Error: Context 2\nbacktrace:\n0: inner 22 error\n1: Context 22\n2: Context Inner\n3: Context 1\n4: Context 2"
                .to_string()
        );
    }

    #[test]
    fn number_of_verifications_with_errors_or_failures() {
        let mut verifs = VerficationsWithErrorAndFailures::new();
        verifs
            .0
            .insert("test1".to_string(), (vec!["error1".to_string()], vec![]));
        verifs
            .0
            .insert("test2".to_string(), (vec![], vec!["failure1".to_string()]));
        verifs.0.insert(
            "test3".to_string(),
            (vec!["error2".to_string()], vec!["failure2".to_string()]),
        );

        assert_eq!(verifs.number_of_verifications_with_errors(), 2);
        assert_eq!(verifs.number_of_verifications_with_failures(), 2);
    }

    #[test]
    fn has_errors_or_failures() {
        let mut verifs = VerficationsWithErrorAndFailures::new();
        verifs
            .0
            .insert("test1".to_string(), (vec!["error1".to_string()], vec![]));
        verifs
            .0
            .insert("test2".to_string(), (vec![], vec!["failure1".to_string()]));
        verifs.0.insert(
            "test3".to_string(),
            (vec!["error2".to_string()], vec!["failure2".to_string()]),
        );

        assert_eq!(verifs.has_errors("test1"), Some(true));
        assert_eq!(verifs.has_errors("test2"), Some(false));
        assert_eq!(verifs.has_errors("test3"), Some(true));
        assert_eq!(verifs.has_errors("test4"), None);
        assert_eq!(verifs.has_failures("test1"), Some(false));
        assert_eq!(verifs.has_failures("test2"), Some(true));
        assert_eq!(verifs.has_failures("test3"), Some(true));
        assert_eq!(verifs.has_failures("test4"), None);
    }
}
