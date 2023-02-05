//! Module to manage the errors within the verifier
//TODO Document the module

#![feature(trace_macros)]

use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

/// Generic Error for the Verifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifierError<E: Error, K: Display + Debug> {
    kind: K,
    error: Option<E>,
    message: String,
}
//TODO Add function and function parameters in struct (or do it in message)

macro_rules! create_error {
    ($k: expr, $m: expr) => {
        VerifierError::new($k, Option::<EmptyError>::None, $m)
    };
    ($k: expr, $m: expr, $e:expr, $et: ty) => {
        VerifierError::new($k, Option::<$et>::Some($e), $m)
    };
}

macro_rules! create_result_error {
    ($k: expr, $m: expr) => {
        Result::Err(create_error!($k, $m))
    };
    ($k: expr, $m: expr, $e: expr, $et: ty) => {
        Result::Err(VerifierError::new($k, Option::<$et>::Some($e), $m))
    };
}
pub(crate) use create_result_error;

#[derive(Debug)]
struct EmptyError {}

impl Display for EmptyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl Error for EmptyError {}

impl<E: Error, K: Display + Debug> VerifierError<E, K> {
    fn __description(&self) -> String {
        let s: String = format!("Error \"{}\": {}", self.kind, self.message);
        match &self.error {
            None => s,
            Some(e) => format!("{}.\nInternal Error: {}", s, e.to_string()),
        }
    }

    pub fn new(kind: K, error: Option<E>, msg: String) -> Self {
        Self {
            kind,
            error,
            message: msg,
        }
    }
}

impl<T: Error, K: Display + Debug> fmt::Display for VerifierError<T, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.__description())
    }
}

impl<T: Error, K: Display + Debug> Error for VerifierError<T, K> {}

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
            Option::<EmptyError>::None,
            "test".to_string(),
        );
        assert_eq!(e1.__description(), "Error \"toto\": test");
        let e2 = VerifierError::new(
            TestErrorType::Fifi,
            Option::Some(TestError {
                details: "test".to_string(),
            }),
            "test".to_string(),
        );
        assert_eq!(
            e2.__description(),
            "Error \"fifi\": test.\nInternal Error: test"
        );
    }

    #[test]
    fn test_macro_error_new() {
        let e1 = create_error!(TestErrorType::Toto, "test".to_string());
        assert_eq!(e1.__description(), "Error \"toto\": test");
        let e2 = create_error!(
            TestErrorType::Fifi,
            "test".to_string(),
            TestError {
                details: "test".to_string(),
            },
            TestError
        );
        assert_eq!(
            e2.__description(),
            "Error \"fifi\": test.\nInternal Error: test"
        );
    }

    #[test]
    fn test_create_result_error() {
        let e1: Result<u64, VerifierError<EmptyError, TestErrorType>> =
            create_result_error!(TestErrorType::Toto, "test".to_string());
        assert!(e1.is_err());
        assert_eq!(e1.unwrap_err().__description(), "Error \"toto\": test");
        let e2: Result<u64, VerifierError<TestError, TestErrorType>> = create_result_error!(
            TestErrorType::Fifi,
            "test".to_string(),
            TestError {
                details: "test".to_string(),
            },
            TestError
        );
        assert!(e2.is_err());
        assert_eq!(
            e2.unwrap_err().__description(),
            "Error \"fifi\": test.\nInternal Error: test"
        );
    }
}
