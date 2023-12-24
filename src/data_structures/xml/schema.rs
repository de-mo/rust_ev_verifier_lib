use std::collections::HashMap;

use anyhow::anyhow;
use roxmltree::{Document, Node};
use super::{SCHEMA_CONFIG, SCHEMA_DECRYPT};

/// Enum containing the various schemas.
/// 
/// Used to get the [XMLSchema] associated to the enum.
pub enum SchemaEnum {
    CONFIG,
    DECRYPT,
    ECH_0222,
    ECH_0110,
}

struct Namespaces {
    hm: HashMap<String, String>
}

/// Representation of an XML Schema
/// 
/// Can be generated using the enum [SchemaEnum].
pub struct XMLSchema<'a> {
    document: Document<'a>,
    namespaces: Namespaces,
}


impl SchemaEnum {
    /// Get the schema associated to the element of the enum
    pub fn get_schema(&self) -> XMLSchema<'_> {
        match self {
            SchemaEnum::CONFIG => XMLSchema::try_from(SCHEMA_CONFIG).unwrap(),
            SchemaEnum::DECRYPT => XMLSchema::try_from(SCHEMA_DECRYPT).unwrap(),
            SchemaEnum::ECH_0222 => todo!(),
            SchemaEnum::ECH_0110 => todo!(),
        }
    }
}

impl Namespaces {
    pub fn new() -> Self {
        Self { hm: HashMap::new() }
    }

    pub fn set(&mut self, hm: &HashMap<String, String>) {
        self.hm = hm.clone()
    }

    pub fn len(&self) -> usize {
        self.hm.len()
    }
    pub fn uri(&self, key: &str) -> Option<&str> {
        self.hm.get(key).map(|s| s.as_str())
    }

    pub fn name(&self, uri: &str) -> Option<&str> {
        self.hm
            .iter()
            .find(|(_, u)| u.as_str()==uri)
            .map(|(n, _)| n.as_str())
    }
}

impl<'a> XMLSchema<'a> {

    /// Derive the namespace of the schema according to the content of the tag `<xs:schema>`
    pub fn derive_namespaces(&mut self) {
        let schema = self.schema_tag();
        let hm = schema.namespaces().map(|ns| 
            match ns.name() {
                Some(n) => (n.to_string(), ns.uri().to_string()),
                None => (String::new(), ns.uri().to_string()),
            }
        ).collect();
        self.namespaces.set(&hm);
    }

    /// Get the [Document] from the crate [roxmltree].
    pub fn document(&self) -> &'a Document {
        &self.document
    }

    /// Get the root of the [Document]
    fn root(&'a self) -> Node<'a, 'a> {
        self.document.root()
    }

    /// Get the schema tag containing namespace information
    fn schema_tag(&'a self) -> Node<'a, 'a> {
        self.root().children().find(|n| !n.is_comment()).unwrap()
    }

    /// List of the namespaces used in the xml document
    /// 
    /// panic if no namespace is set.
    pub fn namespaces(&self) -> &Namespaces {
        &self.namespaces
    }

    /// Return the value of `targetNamespace`. None if not set
    pub fn target_namespace(&'a self) -> Option<&'a str> {
        self.schema_tag()
            .attributes()
            .find(|attr| attr.name()=="targetNamespace")
            .map(|attr| attr.value())
    }
}

impl<'a> TryFrom<&'static str> for XMLSchema<'a> {
    type Error=anyhow::Error;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        let document = Document::parse(value).map_err(|e| anyhow!(e))?;
        let mut res = Self { 
            document,
            namespaces: Namespaces::new()
        };
        res.derive_namespaces();
        Ok(res)
    }
}
 

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_namespaces() {
        let mut ns = Namespaces::new();
        assert_eq!(ns.len(), 0);
        ns.set(&HashMap::from([("a".to_string(),"1".to_string()),("b".to_string(),"3".to_string()),("c".to_string(),"2".to_string())]));
        assert_eq!(ns.len(), 3);
        assert_eq!(ns.uri("a"), Some("1"));
        assert_eq!(ns.uri("b"), Some("3"));
        assert_eq!(ns.uri("c"), Some("2"));
        assert_eq!(ns.uri("d"), None);
        assert_eq!(ns.name("1"), Some("a"));
        assert_eq!(ns.name("2"), Some("c"));
        assert_eq!(ns.name("4"), None);
    }


    #[test]
    fn test_config() {
        let xsd_config = SchemaEnum::CONFIG.get_schema();
        assert_eq!(xsd_config.target_namespace().unwrap(), "http://www.evoting.ch/xmlns/config/5");
        let ns = xsd_config.namespaces();
        assert_eq!(ns.len(), 2);
        assert_eq!(ns.uri("xs"), Some("http://www.w3.org/2001/XMLSchema"));
        assert_eq!(ns.uri("config"), Some("http://www.evoting.ch/xmlns/config/5"));
    }

    #[test]
    fn test_decrypt() {
        let xsd_config = SchemaEnum::DECRYPT.get_schema();
        assert_eq!(xsd_config.target_namespace().unwrap(), "http://www.evoting.ch/xmlns/decrypt/1");
        let ns = xsd_config.namespaces();
        assert_eq!(ns.len(), 2);
        assert_eq!(ns.uri("xs"), Some("http://www.w3.org/2001/XMLSchema"));
        assert_eq!(ns.uri("decrypt"), Some("http://www.evoting.ch/xmlns/decrypt/1")); 
    }
}
