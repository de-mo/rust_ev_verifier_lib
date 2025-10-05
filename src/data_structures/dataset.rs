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

//! Module implementing [SetupOrTally]

use enum_kinds::EnumKind;
use strum::{AsRefStr, EnumString};

/// Generic Enum that is a type of context, setup or tally
#[derive(Clone, AsRefStr, EnumKind)]
#[enum_kind(
    DatasetTypeKind,
    derive(Hash, AsRefStr, EnumString),
    strum(serialize_all = "lowercase")
)]
pub enum DatasetType<C, T> {
    Context(C),
    Tally(T),
}

impl<C: Clone, T: Clone> DatasetType<C, T> {
    /// Is context
    pub fn is_context(&self) -> bool {
        matches!(self, DatasetType::Context(_))
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
