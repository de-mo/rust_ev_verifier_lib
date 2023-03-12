#[derive(Clone)]
pub enum SetupOrTally<S: Clone, T: Clone> {
    Setup(S),
    Tally(T),
}

impl<S: Clone, T: Clone> SetupOrTally<S, T> {
    pub fn is_left(&self) -> bool {
        match self {
            SetupOrTally::Setup(_) => true,
            SetupOrTally::Tally(_) => false,
        }
    }

    pub fn is_right(&self) -> bool {
        !self.is_left()
    }

    pub fn unwrap_setup(self) -> S
    where
        T: core::fmt::Debug,
    {
        match self {
            SetupOrTally::Setup(s) => s,
            SetupOrTally::Tally(t) => {
                panic!(
                    "called `SetupOrTally::unwrap_setup()` on a `Tally` value: {:?}",
                    t
                )
            }
        }
    }

    pub fn unwrap_tally(self) -> T
    where
        S: core::fmt::Debug,
    {
        match self {
            SetupOrTally::Setup(s) => {
                panic!(
                    "called `SetupOrTally::unwrap_tally()` on a `Setup` value: {:?}",
                    s
                )
            }
            SetupOrTally::Tally(t) => t,
        }
    }
}
