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

use derive_builder::Builder;
use derive_getters::Getters;
use tracing::Level;

/// Configuration of the report
#[derive(Debug, Clone, PartialEq, Builder, Getters)]
pub struct ReportConfig {
    /// Size of the tabulation in the output
    tab_size: u8,

    /// Print output log generating the report
    ///
    /// Default: `false`
    #[builder(default = false)]
    output_log: bool,

    /// Level of output log
    ///
    /// Default: `[Level::INFO]`
    #[builder(default=Level::INFO)]
    output_log_level: Level,

    /// Format of the printed date
    fromat_date: String,
}
