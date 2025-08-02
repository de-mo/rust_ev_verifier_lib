use std::fmt::Display;

/// An iterator over an Error and its sources.
///
/// If you want to omit the initial error and only process its sources, use `skip(1)`.
#[derive(Debug, Clone)]
pub struct ErrorChain<'a, 'b> {
    inner: Option<&'a (dyn std::error::Error + 'b)>,
}

impl<'a, 'b> ErrorChain<'a, 'b> {
    /// Creates a new error chain iterator.
    pub fn new(error: &'a (dyn std::error::Error + 'b)) -> Self {
        ErrorChain { inner: Some(error) }
    }
}

impl<'a, 'b> Iterator for ErrorChain<'a, 'b> {
    type Item = &'a (dyn std::error::Error + 'b);

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            None => None,
            Some(e) => {
                self.inner = e.source();
                Some(e)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Report<'a> {
    inner: &'a (dyn std::error::Error),
}

impl<'a> Report<'a> {
    pub fn new(error: &'a (dyn std::error::Error)) -> Self {
        Self { inner: error }
    }
}

impl<'a> Display for Report<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_str = ErrorChain::new(self.inner)
            .map(|e| e.to_string())
            .collect::<Vec<_>>();
        let mut res = vec![self.inner.to_string()];
        if error_str.len() > 1 {
            res.push("backtrace:".to_string());
            res.append(
                &mut error_str
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("{i}: {s}"))
                    .collect(),
            );
        }
        write!(f, "{}", res.join("\n"))
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
    fn test_iter() {
        let e = Outer::Inner {
            source: Inner::Inner21 { source: Inner21 {} },
        };
        let res = ErrorChain::new(&e)
            .map(|e| e.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            res,
            vec![
                "Context Inner".to_string(),
                "Context 21".to_string(),
                "inner 21 error".to_string()
            ]
        )
    }

    #[test]
    fn test_iter2() {
        let e = Outer::Inner {
            source: Inner::Inner22 { source: Inner22 {} },
        };
        let res = ErrorChain::new(&e)
            .map(|e| e.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            res,
            vec![
                "Context Inner".to_string(),
                "Context 22".to_string(),
                "inner 22 error".to_string()
            ]
        )
    }
}
