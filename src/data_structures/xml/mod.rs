//! Module to manage the schemas used for the verifier
pub mod schema;

pub use schema::*;

macro_rules! schema_dir { () => { "../../../resources/schemas" } }
macro_rules! xsd_configuration_path { () => {concat!(schema_dir!(), "/evoting-config-5-0.xsd")} }
macro_rules! xsd_decryption_path { () => {concat!(schema_dir!(), "/evoting-decrypt-1-3.xsd")} }

static SCHEMA_CONFIG: &str = include_str!(xsd_configuration_path!());
static SCHEMA_DECRYPT: &str = include_str!(xsd_decryption_path!());