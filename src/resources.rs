pub static VERIFICATION_LIST: &str = include_str!("../resources/verification_list.json");
pub static XSD_ECH_0006: &str = include_str!("../resources/schemas/eCH-0006-2-0.xsd");
pub static XSD_ECH_0007: &str = include_str!("../resources/schemas/eCH-0007-6-0.xsd");
pub static XSD_ECH_0008: &str = include_str!("../resources/schemas/eCH-0008-3-0.xsd");
pub static XSD_ECH_0010: &str = include_str!("../resources/schemas/eCH-0010-6-0.xsd");
pub static XSD_ECH_0044: &str = include_str!("../resources/schemas/eCH-0044-4-1.xsd");
pub static XSD_ECH_0058: &str = include_str!("../resources/schemas/eCH-0058-5-0.xsd");
pub static XSD_ECH_0110: &str = include_str!("../resources/schemas/eCH-0110-4-0.xsd");
pub static XSD_ECH_0155: &str = include_str!("../resources/schemas/eCH-0155-4-0.xsd");
pub static XSD_ECH_0222: &str = include_str!("../resources/schemas/eCH-0222-1-0.xsd");
pub static XSD_CONFIG: &str = include_str!("../resources/schemas/evoting-config-6-0.xsd");
pub static XSD_DECRYPT: &str = include_str!("../resources/schemas/evoting-decrypt-1-3.xsd");

#[cfg(test)]
pub(crate) mod test_resources {
    pub static SCHEMA_TEST_1: &str = include_str!("../resources/schemas/test/test_1.xsd");
    pub static SCHEMA_TEST_2: &str = include_str!("../resources/schemas/test/test_2.xsd");
    pub static SCHEMA_TEST_3: &str = include_str!("../resources/schemas/test/test_3.xsd");
}
