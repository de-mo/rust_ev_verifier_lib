use super::schema::{self, Schema};
use anyhow::{anyhow, Context, Error, Result};
use quick_xml::name::QName;
use roxmltree::Node as RoNode;
use std::str;

struct Node<'a> {
    schema: &'a Schema<'a>,
    ro_node: RoNode<'a, 'a>,
    parent: Option<&'a Node<'a>>,
}

enum NodeKind<'a> {
    ComplexType(RoNode<'a, 'a>),
    SimpleType(RoNode<'a, 'a>),
    Native(String),
}

impl<'a> NodeKind<'a> {
    fn is_complex_type(&self) -> bool {
        self.unwrap_complex_type().is_some()
    }

    fn is_simple_type(&self) -> bool {
        self.unwrap_simple_type().is_some()
    }

    fn is_native(&self) -> bool {
        self.unwrap_native().is_some()
    }

    fn unwrap_complex_type(&self) -> Option<RoNode<'a, 'a>> {
        if let Self::ComplexType(n) = self {
            return Some(*n);
        }
        None
    }

    fn unwrap_simple_type(&self) -> Option<RoNode<'a, 'a>> {
        if let Self::SimpleType(n) = self {
            return Some(*n);
        }
        None
    }

    fn unwrap_native(&'a self) -> Option<&'a str> {
        if let Self::Native(s) = self {
            return Some(s.as_str());
        }
        None
    }
}

impl<'a, 'input> Node<'a> {
    fn name(&'a self) -> &'a str {
        self.ro_node.find_attribute("name").unwrap()
    }

    fn is_optional(&self) -> bool {
        self.ro_node.min_occurs() == 0
    }

    fn is_list(&self) -> bool {
        self.ro_node.max_occurs() > 1
    }

    fn node_kind(&'a self) -> Result<NodeKind<'a>> {
        let mut res_node = None;
        // Type is defined as attribute
        if let Some(q_name) = self.ro_node.schema_node_type() {
            // The type name is qualified with prefix
            if q_name.prefix().is_some() {
                // The prefix is for xmlschema
                if q_name.prefix().as_ref().map(|e| e.as_ref())
                    == self.schema.xmlschema_namespace_name().map(|e| e.as_bytes())
                {
                    return Ok(NodeKind::Native(
                        str::from_utf8(q_name.local_name().as_ref())
                            .unwrap()
                            .to_string(),
                    ));
                }
                // The prefix is for target namespace (e.g. in the current schema)
                if q_name.prefix().as_ref().map(|e| e.as_ref())
                    == self.schema.target_namespace_name().map(|e| e.as_bytes())
                {
                    let n = self
                        .ro_node
                        .find_node_with_name(str::from_utf8(q_name.local_name().as_ref()).unwrap());
                    if n.is_some() {
                        res_node = Some(n.unwrap());
                    }
                }
            }
            // Not qualified -> the result remains none
        }
        // Type not in the attribute. Take the first child
        else if let Some(fcn) = self.ro_node.first_element_child() {
            res_node = Some(fcn);
        }
        match res_node {
            None => Err(anyhow!("No type found")),
            Some(n) => {
                if n.is_schema_complex_type() {
                    return Ok(NodeKind::ComplexType(n));
                }
                if n.is_schema_simple_type() {
                    return Ok(NodeKind::SimpleType(n));
                }
                Err(anyhow!(
                    "The node {} ist not a simple type or a complex type",
                    n.node_tag_name()
                ))
            }
        }
    }

    fn children(&'a self) -> Result<Vec<Self>> {
        let mut res = vec![];
        let kind = self.node_kind().context("Error getting children")?;
        if let Some(n) = kind.unwrap_complex_type() {
            let seq = n
                .first_element_child()
                .ok_or(anyhow!("Missing first child"))?;
            for e in seq.children().filter(|e| e.is_schema_element()) {
                res.push(Self {
                    schema: self.schema,
                    ro_node: e,
                    parent: Some(self),
                })
            }
        }
        Ok(res)
    }
}

impl<'a> From<&'a schema::Schema<'a>> for Node<'a> {
    fn from(value: &'a schema::Schema) -> Self {
        let root = value.root_element();
        let node = root.children().find(|e| e.is_schema_element()).unwrap();
        Self {
            schema: value,
            ro_node: node,
            parent: None,
        }
    }
}

trait AdditionalMethodsRoxmlNode<'a>: Sized {
    fn node_tag_name(&self) -> &'a str;
    fn find_attribute(&'a self, name: &str) -> Option<&'a str>;
    fn find_node_with_name(&'a self, name: &str) -> Option<Self>;

    fn is_schema_element(&self) -> bool {
        self.node_tag_name() == "element"
    }

    fn is_schema_complex_type(&self) -> bool {
        self.node_tag_name() == "complexType"
    }

    fn is_schema_simple_type(&self) -> bool {
        self.node_tag_name() == "simpleType"
    }

    fn is_schema_type(&self) -> bool {
        self.is_schema_complex_type() || self.is_schema_simple_type()
    }

    fn min_occurs(&'a self) -> usize {
        self.find_attribute("minOccurs")
            .map_or(1, |e| e.parse::<usize>().unwrap())
    }

    fn max_occurs(&'a self) -> usize {
        self.find_attribute("maxOccurs").map_or(1, |e| match e {
            "unbounded" => usize::MAX,
            s => s.parse::<usize>().unwrap(),
        })
    }

    fn schema_node_type(&'a self) -> Option<QName> {
        self.find_attribute("type").map(|e| QName(e.as_bytes()))
    }
}

impl<'a> AdditionalMethodsRoxmlNode<'a> for RoNode<'a, 'a> {
    fn find_attribute(&'a self, name: &str) -> Option<&'a str> {
        self.attributes()
            .find(|e| e.name() == name)
            .map(|e| e.value())
    }

    fn node_tag_name(&self) -> &'a str {
        self.tag_name().name()
    }

    fn find_node_with_name(&'a self, name: &str) -> Option<Self> {
        self.document()
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some(name))
    }
}

#[cfg(test)]
mod test {
    use super::super::schema::SchemaKind;
    use super::*;

    fn schema<'a>() -> &'a Schema<'a> {
        SchemaKind::config.get_schema()
    }

    #[test]
    fn test_from_schema() {
        let xsd = schema();
        let node = Node::from(xsd);
        assert!(node.parent.is_none());
        assert_eq!(node.ro_node.node_tag_name(), "element");
        assert_eq!(node.ro_node.find_attribute("name"), Some("configuration"))
    }

    #[test]
    fn test_node_type_complex() {
        let xsd = schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.is_schema_element())
            .unwrap();
        let node1 = Node {
            schema: xsd,
            ro_node: n1,
            parent: None,
        };
        let r_k1 = node1.node_kind();
        assert!(r_k1.is_ok());
        let k1 = r_k1.unwrap();
        assert!(k1.is_complex_type());
        assert_eq!(k1.unwrap_complex_type().unwrap().attributes().len(), 0);
        let n_parent = n1
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        let n2 = n_parent.first_element_child().unwrap();
        let node2 = Node {
            schema: xsd,
            ro_node: n2,
            parent: None,
        };
        let r_k2 = node2.node_kind();
        assert!(r_k2.is_ok());
        let k2 = r_k2.unwrap();
        assert!(k2.is_complex_type());
        assert_eq!(
            k2.unwrap_complex_type()
                .unwrap()
                .find_attribute("name")
                .unwrap(),
            "headerType"
        );
    }

    #[test]
    fn test_node_type_simple() {
        let xsd = schema();
        let root = xsd.root_element();
        let n1 = root
            .find_node_with_name("adminBoardType")
            .unwrap()
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        let node1 = Node {
            schema: xsd,
            ro_node: n1,
            parent: None,
        };
        let r_k1 = node1.node_kind();
        assert!(r_k1.is_ok());
        let k1 = r_k1.unwrap();
        assert!(k1.is_simple_type());
        assert_eq!(
            k1.unwrap_simple_type()
                .unwrap()
                .find_attribute("name")
                .unwrap(),
            "identifierType"
        );
        let n2 = n1.next_sibling_element().unwrap();
        let node2 = Node {
            schema: xsd,
            ro_node: n2,
            parent: None,
        };
        let r_k2 = node2.node_kind();
        assert!(r_k2.is_ok());
        let k2 = r_k2.unwrap();
        assert!(k2.is_simple_type());
        assert_eq!(k2.unwrap_simple_type().unwrap().attributes().len(), 0);
    }

    #[test]
    fn test_children_1() {
        let xsd = schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.is_schema_element())
            .unwrap();
        let node1 = Node {
            schema: xsd,
            ro_node: n1,
            parent: None,
        };
        let r_cs = node1.children();
        assert!(r_cs.is_ok());
        let cs = r_cs.unwrap();
        assert_eq!(cs.len(), 5);
        assert_eq!(cs[0].name(), "header");
        assert!(cs[0].node_kind().unwrap().is_complex_type());
        assert_eq!(cs[1].name(), "contest");
        assert!(cs[1].node_kind().unwrap().is_complex_type());
        assert_eq!(cs[2].name(), "authorizations");
        assert!(cs[2].node_kind().unwrap().is_complex_type());
        assert_eq!(cs[3].name(), "register");
        assert!(cs[3].node_kind().unwrap().is_complex_type());
        assert_eq!(cs[4].name(), "signature");
        assert!(cs[4].node_kind().unwrap().is_native());
    }

    #[test]
    fn test_children_2() {
        let xsd = schema();
        let root = xsd.root_element();
        let n1 = root
            .find_node_with_name("contestType")
            .unwrap()
            .first_element_child()
            .unwrap()
            .children()
            .find(|e| {
                e.find_attribute("name").is_some()
                    && e.find_attribute("name").unwrap() == "adminBoard"
            })
            .unwrap();
        let node1 = Node {
            schema: xsd,
            ro_node: n1,
            parent: None,
        };
        let r_cs = node1.children();
        assert!(r_cs.is_ok());
        let cs = r_cs.unwrap();
        assert_eq!(cs.len(), 5);
        assert_eq!(cs[0].name(), "adminBoardIdentification");
        assert!(cs[0].node_kind().unwrap().is_simple_type());
        assert_eq!(cs[1].name(), "adminBoardName");
        assert!(cs[1].node_kind().unwrap().is_simple_type());
        assert_eq!(cs[2].name(), "adminBoardDescription");
        assert!(cs[2].node_kind().unwrap().is_simple_type());
        assert_eq!(cs[3].name(), "adminBoardThresholdValue");
        assert!(cs[3].node_kind().unwrap().is_simple_type());
        assert_eq!(cs[4].name(), "adminBoardMembers");
        assert!(cs[4].node_kind().unwrap().is_complex_type());
    }
}

#[cfg(test)]
mod test_additional_method_node {
    use super::super::schema::SchemaKind;
    use super::*;

    fn schema<'a>() -> &'a Schema<'a> {
        SchemaKind::config.get_schema()
    }

    #[test]
    fn test_tag_name() {
        let xsd = schema();
        let root = xsd.root_element();
        assert_eq!(root.node_tag_name(), "schema");
        let n1 = root.children().find(|e| e.node_tag_name() == "element");
        assert!(n1.is_some());
        let n2 = n1
            .unwrap()
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        assert_eq!(n2.node_tag_name(), "sequence");
    }

    #[test]
    fn test_find_attribute() {
        let xsd = schema();
        let node = xsd
            .root_element()
            .children()
            .find(|e| e.node_tag_name() == "element")
            .unwrap()
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        assert_eq!(node.find_attribute("name"), Some("header"));
        assert_eq!(node.find_attribute("type"), Some("config:headerType"));
        assert_eq!(node.find_attribute("toto"), None);
    }

    #[test]
    fn test_is_element() {
        let xsd = schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.node_tag_name() == "element")
            .unwrap();
        assert!(n1.is_schema_element());
        assert!(!n1.first_element_child().unwrap().is_schema_element());
    }

    #[test]
    fn test_is_complex_type() {
        let xsd = schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.node_tag_name() == "element")
            .unwrap();
        assert!(!n1.is_schema_complex_type());
        assert!(n1.first_element_child().unwrap().is_schema_complex_type());
    }

    #[test]
    fn test_is_simple_type() {
        let xsd = schema();
        let n1 = xsd.root_element().first_element_child().unwrap();
        assert!(!xsd.root_element().is_schema_simple_type());
        assert!(n1.is_schema_simple_type());
    }

    #[test]
    fn test_max_occurs() {
        let xsd = schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some("contestDescriptionInformationType"))
            .unwrap();
        assert_eq!(n1.max_occurs(), 1);
        let n2 = n1
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        assert_eq!(n2.max_occurs(), usize::MAX);
    }

    #[test]
    fn test_min_occurs() {
        let xsd = schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some("contestDescriptionInformationType"))
            .unwrap();
        assert_eq!(n1.min_occurs(), 1);
        let n2 = n1
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        assert_eq!(n2.min_occurs(), 4);
    }

    #[test]
    fn test_find_node_with_name() {
        let xsd = schema();
        let n1 = xsd.root_element().first_element_child().unwrap();
        assert!(n1
            .find_node_with_name("electoralBoardType")
            .unwrap()
            .is_schema_complex_type());
        assert!(n1
            .find_node_with_name("configuration")
            .unwrap()
            .is_schema_element());
        assert!(n1
            .find_node_with_name("languageType")
            .unwrap()
            .is_schema_simple_type());
        assert!(n1.find_node_with_name("toto").is_none())
    }

    #[test]
    fn test_schema_node_type() {
        let xsd = schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.node_tag_name() == "element")
            .unwrap();
        assert!(n1.schema_node_type().is_none());
        let n2 = n1
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        let n3 = n2.first_element_child().unwrap();
        let qn3 = n3.schema_node_type().unwrap();
        assert_eq!(qn3.local_name().as_ref(), "headerType".as_bytes());
        assert_eq!(qn3.prefix().unwrap().as_ref(), "config".as_bytes());
        let n4 = n2.last_element_child().unwrap();
        let qn4 = n4.schema_node_type().unwrap();
        assert_eq!(qn4.local_name().as_ref(), "base64Binary".as_bytes());
        assert_eq!(qn4.prefix().unwrap().as_ref(), "xs".as_bytes());
    }
}
