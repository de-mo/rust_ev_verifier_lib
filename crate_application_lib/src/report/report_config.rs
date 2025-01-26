use derive_getters::Getters;
use tracing::Level;
use typed_builder::TypedBuilder;

/// Configuration of the report
#[derive(Debug, Clone, PartialEq, TypedBuilder, Getters)]
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
