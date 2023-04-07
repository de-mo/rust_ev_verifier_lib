//! Module to manage the errors within the verifier
//!
//! The errors of the verifiier can be chained to have a deep view of all the errors
//!
//! Each module implements its own error using the following construction
//! ```rust
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! pub enum ExampleErrorType {
//!    Element1,
//!    Element2,
//! }
//!
//! impl Display for ExampleErrorType {
//!    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!        let s = match self {
//!            Self::Element1 => "Text Element1",
//!            Self::Element1 => "Text Element2",
//!        };
//!        write!(f, "{s}")
//!    }
//! }
//!
//! type ExampleError = VerifierError<ExampleErrorType>;
//! ```
//!
//! Using the macros [create_verifier_error] and [create_result_with_error] is it possible to
//! create easily errors, e.g.:
//! ```rust
//! create_verifier_error!(ExampleErrorType::Element1, "text of error")
//! create_result_with_error!(ExampleErrorType::Element1, "text of error")
//! ```
//!

use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

/// This type represents all possible errors that can occur within the
/// verifier.
#[derive(Debug)]
pub struct VerifierError<K: Display + Debug> {
    /// This `Box` allows us to keep the size of `Error` as small as possible. A
    /// larger `Error` type was substantially slower due to all the functions
    /// that pass around results.
    err: Box<VerifierErrorImpl<K>>,
}

impl<K: Display + Debug> VerifierError<K> {
    pub fn kind(&self) -> &K {
        &self.err.kind
    }

    pub fn message(&self) -> &String {
        &&self.err.message
    }

    pub fn source(&self) -> &Option<Box<dyn Error + 'static>> {
        &self.err.source
    }
}

impl<K: Display + Debug> VerifierError<K> {
    fn __description(&self) -> String {
        let s: String = format!("Error \"{}\": {}", self.kind(), self.message());
        match &self.source() {
            None => s,
            Some(e) => format!("{}.\nInternal Error: {}", s, e.to_string()),
        }
    }

    pub fn new(kind: K, source: Option<Box<dyn Error + 'static>>, msg: String) -> Self {
        Self {
            err: Box::new(VerifierErrorImpl {
                kind,
                source,
                message: msg,
            }),
        }
    }
}

impl<K: Display + Debug> fmt::Display for VerifierError<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.__description())
    }
}

#[derive(Debug)]
struct VerifierErrorImpl<K: Display + Debug> {
    kind: K,
    source: Option<Box<dyn Error + 'static>>,
    message: String,
}

/// Macro to create an error.
macro_rules! create_verifier_error {
    ($k: expr, $m: expr) => {
        VerifierError::new($k, None, $m.to_string())
    };
    ($k: expr, $m: expr, $e: expr) => {
        VerifierError::new($k, Some(Box::new($e)), $m.to_string())
    };
}
pub(crate) use create_verifier_error;

/// Macro to create an error and encapuslate it in a Result.
macro_rules! create_result_with_error {
    ($k: expr, $m: expr) => {
        Result::Err(create_verifier_error!($k, $m))
    };
    ($k: expr, $m: expr, $e: expr) => {
        Result::Err(create_verifier_error!($k, $m, $e))
    };
}
pub(crate) use create_result_with_error;

#[derive(Debug)]
struct EmptyError {}

impl Display for EmptyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl Error for EmptyError {}

impl<K: Display + Debug> Error for VerifierError<K> {}

#[cfg(test)]
mod test {
    use super::*;
    use std::option::Option;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestErrorType {
        Toto,
        Fifi,
    }

    #[derive(Debug)]
    struct TestError {
        details: String,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.details)
        }
    }

    impl Error for TestError {
        fn description(&self) -> &str {
            &self.details
        }
    }

    impl Display for TestErrorType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                Self::Toto => "toto",
                Self::Fifi => "fifi",
            };
            write!(f, "{s}")
        }
    }
    #[test]
    fn test_error_new() {
        let e1 = VerifierError::new(
            TestErrorType::Toto,
            Option::<Box<dyn Error + 'static>>::None,
            "test".to_string(),
        );
        assert_eq!(e1.__description(), "Error \"toto\": test");
        let e2 = VerifierError::new(
            TestErrorType::Fifi,
            Option::Some(Box::new(TestError {
                details: "test".to_string(),
            })),
            "test".to_string(),
        );
        assert_eq!(
            e2.__description(),
            "Error \"fifi\": test.\nInternal Error: test"
        );
    }

    #[test]
    fn test_create_result_error() {
        let e1: Result<u64, VerifierError<TestErrorType>> =
            create_result_with_error!(TestErrorType::Toto, "test".to_string());
        assert!(e1.is_err());
        assert_eq!(e1.unwrap_err().__description(), "Error \"toto\": test");
        let e2: Result<u64, VerifierError<TestErrorType>> = create_result_with_error!(
            TestErrorType::Fifi,
            "test".to_string(),
            Box::new(TestError {
                details: "test".to_string(),
            })
        );
        assert!(e2.is_err());
        assert_eq!(
            e2.unwrap_err().__description(),
            "Error \"fifi\": test.\nInternal Error: test"
        );
    }
}
