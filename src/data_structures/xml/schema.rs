//! Module to read the static schema and to prodive some functionalities in the
//! structure
//!
//! Use the object [OnceLock] to create the structure only once from the static string. Action is thread safe

use crate::resources;
use anyhow::{anyhow, Context, Result};
use roxmltree::{Document, Node as RoNode};
use std::collections::HashMap;
use std::sync::OnceLock;

const NS_ECH_0006: &str = "http://www.ech.ch/xmlns/eCH-0006/2";
const NS_ECH_0007: &str = "http://www.ech.ch/xmlns/eCH-0007/6";
const NS_ECH_0008: &str = "http://www.ech.ch/xmlns/eCH-0008/3";
const NS_ECH_0010: &str = "http://www.ech.ch/xmlns/eCH-0010/6";
const NS_ECH_0044: &str = "http://www.ech.ch/xmlns/eCH-0044/4";
const NS_ECH_0058: &str = "http://www.ech.ch/xmlns/eCH-0058/5";
const NS_ECH_0155: &str = "http://www.ech.ch/xmlns/eCH-0155/4";

static SCHEMA_CELL_ECH_0006: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0007: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0008: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0010: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0044: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0058: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0110: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0155: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_0222: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_DECRYPT: OnceLock<Schema> = OnceLock::new();
static SCHEMA_CELL_ECH_CONFIG: OnceLock<Schema> = OnceLock::new();

const XML_SCHEMA_URI: &str = "http://www.w3.org/2001/XMLSchema";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
/// Enumarate for the kind of schemas.
pub enum SchemaKind {
    Ech0006,
    Ech0007,
    Ech0008,
    Ech0010,
    Ech0044,
    Ech0058,
    Ech0110,
    Ech0155,
    Ech0222,
    Decrypt,
    Config,
}

/// Schema containing the structure of the schema
#[allow(dead_code)]
pub struct Schema<'a> {
    document: Document<'a>,
    schema_kind: Option<SchemaKind>,
    target_namespace_name: String,
    target_namespace_uri: String,
    xml_schema_name: String,
    namespaces: HashMap<String, String>,
    sub_schemas: HashMap<String, &'a Schema<'a>>,
}

impl SchemaKind {
    /// Get the schema structure
    pub fn get_schema(&self) -> &Schema {
        match self {
            SchemaKind::Ech0006 => SCHEMA_CELL_ECH_0006
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0006)),
            SchemaKind::Ech0007 => SCHEMA_CELL_ECH_0007
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0007)),
            SchemaKind::Ech0008 => SCHEMA_CELL_ECH_0008
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0008)),
            SchemaKind::Ech0010 => SCHEMA_CELL_ECH_0010
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0010)),
            SchemaKind::Ech0044 => SCHEMA_CELL_ECH_0044
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0044)),
            SchemaKind::Ech0058 => SCHEMA_CELL_ECH_0058
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0058)),
            SchemaKind::Ech0110 => SCHEMA_CELL_ECH_0110
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0110)),
            SchemaKind::Ech0155 => SCHEMA_CELL_ECH_0155
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0155)),
            SchemaKind::Ech0222 => SCHEMA_CELL_ECH_0222
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_ECH_0222)),
            SchemaKind::Decrypt => SCHEMA_CELL_ECH_DECRYPT
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_DECRYPT)),
            SchemaKind::Config => SCHEMA_CELL_ECH_CONFIG
                .get_or_init(|| Schema::new(Some(*self), resources::XSD_CONFIG)),
        }
    }

    pub fn get_schema_from_namespace(ns: &str) -> Result<&'static Schema<'static>> {
        match ns {
            NS_ECH_0006 => Ok(SchemaKind::Ech0006.get_schema()),
            NS_ECH_0007 => Ok(SchemaKind::Ech0007.get_schema()),
            NS_ECH_0008 => Ok(SchemaKind::Ech0008.get_schema()),
            NS_ECH_0010 => Ok(SchemaKind::Ech0010.get_schema()),
            NS_ECH_0044 => Ok(SchemaKind::Ech0044.get_schema()),
            NS_ECH_0058 => Ok(SchemaKind::Ech0058.get_schema()),
            NS_ECH_0155 => Ok(SchemaKind::Ech0155.get_schema()),
            _ => Err(anyhow!("No schema for namespace {} found", ns)),
        }
    }
}

impl<'a> Schema<'a> {
    /// Try to create a new schema of kind [schema_kind] with the static str [xsd_str]
    ///
    /// Return an error in the following cases:
    /// - It is not possible to create it
    /// - Targetnamespace is missing
    pub fn try_new(schema_kind: Option<SchemaKind>, xsd_str: &'static str) -> Result<Self> {
        let doc = Document::parse(xsd_str).with_context(|| "Failed to read the schema")?;
        let root = doc.root_element();
        let target_ns_uri = root
            .attributes()
            .find(|attr| attr.name() == "targetNamespace")
            .map(|a| a.value().to_string())
            .ok_or(anyhow!("targetNamespace is missing"))?;
        let mut hm = HashMap::new();
        for ns in root.namespaces() {
            hm.insert(ns.name().unwrap().to_string(), ns.uri().to_string());
        }
        let target_ns_name = hm
            .iter()
            .find(|(_, uri)| uri == &&target_ns_uri)
            .ok_or(anyhow!(
                "The name of the target namespace is not defined in the list of namespaces"
            ))?
            .0;
        let schema_ns_name = hm
            .iter()
            .find(|(_, uri)| uri.as_str() == XML_SCHEMA_URI)
            .ok_or(anyhow!(
                "The name of the xml schema is not defined in the list of namespaces"
            ))?
            .0;
        let mut res = Self {
            document: doc,
            target_namespace_uri: target_ns_uri,
            target_namespace_name: target_ns_name.clone(),
            xml_schema_name: schema_ns_name.clone(),
            namespaces: hm,
            schema_kind,
            sub_schemas: HashMap::new(),
        };
        let sub_schmeas = res.collect_import()?;
        res.sub_schemas = sub_schmeas;
        Ok(res)
    }

    fn collect_import(&mut self) -> Result<HashMap<String, &'a Schema<'a>>> {
        let tag_iter = self.root_element().children().filter(|e| {
            let tag = e.tag_name();
            tag.name() == "import" && tag.namespace() == Some(XML_SCHEMA_URI)
        });
        let nss_res: Vec<Result<String>> = tag_iter
            .map(|e| {
                e.attributes()
                    .find(|attr| attr.name() == "namespace")
                    .map(|a| a.value().to_string())
                    .ok_or(anyhow!("namespace is missing"))
            })
            .collect();
        let mut res = HashMap::new();
        for ns_res in nss_res {
            let ns = ns_res?;
            res.insert(
                ns.clone(),
                SchemaKind::get_schema_from_namespace(ns.as_str()).context("Collection import")?,
            );
        }
        Ok(res)
    }

    /// Try to create a new schema of kind [schema_kind] with the static str [xsd_str]
    ///
    /// Panic if it is not possible to create it
    pub fn new(schema_kind: Option<SchemaKind>, xsd_str: &'static str) -> Self {
        Self::try_new(schema_kind, xsd_str).unwrap()
    }

    /// Root element of the schema
    pub fn root_element(&'a self) -> RoNode<'a, 'a> {
        self.document().root_element()
    }

    /// The source document of type [Document]
    #[allow(dead_code)]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// The name of the target namespace's name, based on the uri in `targetNamespace` and the list of namespaces
    pub fn target_namespace_name(&'a self) -> &'a str {
        self.target_namespace_name.as_str()
    }

    /// The name of the xml schema namespace's name, based on the standard uri "http://www.w3.org/2001/XMLSchema" and the list of namespaces
    pub fn xmlschema_namespace_name(&'a self) -> &'a str {
        self.xml_schema_name.as_str()
    }

    /// Return the schema given by the namespace
    ///
    /// # Error
    /// Return an error if the namespace is not found
    pub fn sub_schema(&'a self, namespace: &str) -> Result<&'a Schema<'a>> {
        self.sub_schemas.get(namespace).map_or(
            Err(anyhow!("Namespace {} not found in import", namespace)),
            |&s| Ok(s),
        )
    }

    /// Return the schema given by the namespace name
    ///
    /// # Error
    /// Return an error if the namespace is not found
    #[allow(dead_code)]
    pub fn sub_schema_name(&'a self, namespace_name: &str) -> Result<&'a Schema<'a>> {
        self.namespace_uri(namespace_name).map_or(
            Err(anyhow!("Namespace name {} not found", namespace_name)),
            |uri| self.sub_schema(uri),
        )
    }

    /// Return the namespace uri given by the namespace name. `None` if not found
    #[allow(dead_code)]
    pub fn namespace_uri(&self, namespace_name: &str) -> Option<&str> {
        self.namespaces.get(namespace_name).map(|uri| uri.as_str())
    }

    /// Check if the schema with the given namespace exists
    #[allow(dead_code)]
    pub fn contains_schema(&self, namespace: &str) -> bool {
        self.namespaces.contains_key(namespace)
    }

    /// Check if the schema with the given namespace name exists
    #[allow(dead_code)]
    pub fn contains_schema_with_namespace_name(&self, namespace_name: &str) -> bool {
        match self.namespaces.get(namespace_name) {
            Some(uri) => self.contains_schema(uri),
            None => false,
        }
    }

    pub fn sub_schema_nodes_with_name(&'a self) -> HashMap<String, RoNode<'a, 'a>> {
        let mut res = HashMap::new();
        for (n, u) in self
            .namespaces
            .iter()
            .filter(|(n, _)| n != &self.target_namespace_name() && n != &&self.xml_schema_name)
        {
            res.insert(n.to_string(), self.sub_schema(u).unwrap().root_element());
        }
        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_schema_decrypt() {
        let xsd = SchemaKind::Decrypt.get_schema();
        assert_eq!(
            xsd.target_namespace_uri,
            "http://www.evoting.ch/xmlns/decrypt/1"
        );
        assert_eq!(xsd.namespaces.keys().len(), 2);
        assert_eq!(
            xsd.namespaces.get("decrypt").unwrap(),
            "http://www.evoting.ch/xmlns/decrypt/1"
        );
        assert_eq!(
            xsd.namespaces.get("xs").unwrap(),
            "http://www.w3.org/2001/XMLSchema"
        );
    }

    #[test]
    fn test_schema_config() {
        let xsd = SchemaKind::Config.get_schema();
        assert_eq!(
            xsd.target_namespace_uri,
            "http://www.evoting.ch/xmlns/config/5"
        );
        assert_eq!(xsd.namespaces.keys().len(), 2);
        assert_eq!(
            xsd.namespaces.get("config").unwrap(),
            "http://www.evoting.ch/xmlns/config/5"
        );
        assert_eq!(
            xsd.namespaces.get("xs").unwrap(),
            "http://www.w3.org/2001/XMLSchema"
        );
    }

    #[test]
    fn test_target_namespace_name() {
        let xsd = SchemaKind::Config.get_schema();
        assert_eq!(xsd.target_namespace_name(), "config");
    }

    #[test]
    fn test_xmlschema_namespace_name() {
        let xsd = SchemaKind::Config.get_schema();
        assert_eq!(xsd.xmlschema_namespace_name(), "xs");
    }

    #[test]
    fn test_sub_schema() {
        let xsd = SchemaKind::Ech0222.get_schema();
        assert_eq!(xsd.sub_schemas.len(), 2);
        assert_eq!(
            xsd.sub_schemas
                .get(&NS_ECH_0058.to_string())
                .unwrap()
                .schema_kind,
            Some(SchemaKind::Ech0058)
        );
        assert_eq!(
            xsd.sub_schemas
                .get(&NS_ECH_0155.to_string())
                .unwrap()
                .schema_kind,
            Some(SchemaKind::Ech0155)
        );
        assert_eq!(
            xsd.sub_schema(NS_ECH_0058).unwrap().schema_kind,
            Some(SchemaKind::Ech0058)
        );
        assert_eq!(
            xsd.sub_schema(NS_ECH_0155).unwrap().schema_kind,
            Some(SchemaKind::Ech0155)
        );
    }
}

#[cfg(test)]
pub(super) mod test_schemas {
    use super::*;
    use crate::resources::test_resources::{SCHEMA_TEST_1, SCHEMA_TEST_2};

    static SCHEMA_CELL_TEST_1: OnceLock<Schema> = OnceLock::new();
    static SCHEMA_CELL_TEST_2: OnceLock<Schema> = OnceLock::new();

    pub fn get_schema_test_1<'a>() -> &'a Schema<'a> {
        SCHEMA_CELL_TEST_1.get_or_init(|| Schema::new(None, SCHEMA_TEST_1))
    }

    pub fn get_schema_test_2<'a>() -> &'a Schema<'a> {
        SCHEMA_CELL_TEST_2.get_or_init(|| Schema::new(None, SCHEMA_TEST_2))
    }
}
