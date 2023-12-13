//! Module implementing [SetupOrTally]

/// Generic Enum that is a type of setup or tally
#[derive(Clone)]
pub enum SetupOrTally<S, T> {
    Setup(S),
    Tally(T),
}

#[allow(dead_code)]
impl<S: Clone, T: Clone> SetupOrTally<S, T> {
    /// Is setup
    pub fn is_setup(&self) -> bool {
        match self {
            SetupOrTally::Setup(_) => true,
            SetupOrTally::Tally(_) => false,
        }
    }

    /// Is tally
    pub fn is_tally(&self) -> bool {
        !self.is_setup()
    }

    /// Unwrap setup and give a reference to S
    ///
    /// panic if type is tally
    pub fn unwrap_setup(&self) -> &S {
        match self {
            SetupOrTally::Setup(s) => s,
            SetupOrTally::Tally(_) => {
                panic!("called `SetupOrTally::unwrap_setup()` on a `Tally` value")
            }
        }
    }

    /// Unwrap tally and give a reference to S
    ///
    /// panic if type is seup
    pub fn unwrap_tally(&self) -> &T {
        match self {
            SetupOrTally::Setup(_) => {
                panic!("called `SetupOrTally::unwrap_tally()` on a `Setup` value")
            }
            SetupOrTally::Tally(t) => t,
        }
    }
}
