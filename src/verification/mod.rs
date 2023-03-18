pub mod error;
pub mod setup;
pub mod tally;
pub mod verification;
use std::collections::{hash_map::Iter, HashMap};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum VerificationCategory {
    Authenticity,
    Consistency,
    Completness,
    Integrity,
    Evidence,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    Stopped,
    Started,
    Success,
    Error,
    Failed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VerificationPeriod {
    Setup,
    Tally,
}

pub type VerificationList = HashMap<String, verification::Verification>;

pub type VerificationListCategory = HashMap<VerificationCategory, VerificationList>;

pub enum VerificationsForPeriod {
    Setup(Box<VerificationListCategory>),
    Tally(Box<VerificationListCategory>),
}

pub struct VerificationsForPeriodIter<'a> {
    iter_category: Iter<'a, VerificationCategory, VerificationList>,
    iter_list: Option<Iter<'a, String, verification::Verification>>,
}

impl VerificationsForPeriod {
    pub fn new(period: VerificationPeriod) -> VerificationsForPeriod {
        match period {
            VerificationPeriod::Setup => {
                VerificationsForPeriod::Setup(Box::new(setup::get_verifications()))
            }
            VerificationPeriod::Tally => {
                VerificationsForPeriod::Tally(Box::new(tally::get_verifications()))
            }
        }
    }

    pub fn value(&self) -> &VerificationListCategory {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_ref(),
            VerificationsForPeriod::Tally(vs) => vs.as_ref(),
        }
    }

    pub fn value_mut(&mut self) -> &mut VerificationListCategory {
        match self {
            VerificationsForPeriod::Setup(vs) => vs.as_mut(),
            VerificationsForPeriod::Tally(vs) => vs.as_mut(),
        }
    }
}

impl<'a> VerificationsForPeriodIter<'a> {
    fn new(list: &'a Box<VerificationListCategory>) -> Self {
        Self {
            iter_category: list.iter(),
            iter_list: None,
        }
    }
}

impl<'a> Iterator for VerificationsForPeriodIter<'a> {
    type Item = &'a verification::Verification;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_list.is_some() {
            let next = self.iter_list.as_mut().unwrap().next();
            if next.is_some() {
                return Some(next.unwrap().1);
            }
        }
        match self.iter_category.next() {
            Some(n) => self.next(),
            None => None,
        }
    }
}
