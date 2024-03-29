//! Module implementing [SetupOrTally]

/// Generic Enum that is a type of setup or tally
#[derive(Clone)]
pub enum DatasetType<C, S, T> {
    Context(C),
    Setup(S),
    Tally(T),
}

#[allow(dead_code)]
impl<C: Clone, S: Clone, T: Clone> DatasetType<C, S, T> {
    /// Is context
    pub fn is_context(&self) -> bool {
        matches!(self, DatasetType::Context(_))
    }

    /// Is setup
    pub fn is_setup(&self) -> bool {
        matches!(self, DatasetType::Setup(_))
    }

    /// Is tally
    pub fn is_tally(&self) -> bool {
        matches!(self, DatasetType::Tally(_))
    }

    /// Unwrap context and give a reference to S
    ///
    /// panic if type is not context
    pub fn unwrap_context(&self) -> &C {
        match self {
            DatasetType::Context(s) => s,
            _ => {
                panic!("called `unwrap_context()` on a wrong variant")
            }
        }
    }

    /// Unwrap setup and give a reference to S
    ///
    /// panic if type is not setup
    pub fn unwrap_setup(&self) -> &S {
        match self {
            DatasetType::Setup(s) => s,
            _ => {
                panic!("called `unwrap_setup()` on a wrong variant")
            }
        }
    }

    /// Unwrap tally and give a reference to T
    ///
    /// panic if type is not tally
    pub fn unwrap_tally(&self) -> &T {
        match self {
            DatasetType::Tally(t) => t,
            _ => {
                panic!("called `unwrap_tally()` on a wrong variant")
            }
        }
    }
}
