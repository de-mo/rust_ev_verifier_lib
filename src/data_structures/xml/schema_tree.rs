//! Module to define an iterator over the definition of the types in the schema, as a tree structure.

use super::schema::Schema;
use anyhow::anyhow;
use core::fmt;
use quick_xml::name::QName;
use roxmltree::{Document as RoDocument, Node as RoNode};
use std::str;

/// Node in the schema tree
///
/// The tree contains the nodes that reperesent a tag in the xml file
pub struct ElementNode {
    schema: &'static Schema<'static>,
    name: String,
    node_kind: ElementNodeKind,
}

impl fmt::Debug for ElementNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementNode")
            .field("name", &self.name)
            .field("node_kind", &self.node_kind)
            .finish()
    }
}

/// Kind of the element node (complex type or native type)
#[derive(Debug)]
pub enum ElementNodeKind {
    /// Node complex type. It stores the [RoNode] of the location of the information
    ComplexType(Vec<ComplexTypeChildKind>),
    /// Native type. It store the name of the type, without prefix
    Native(String),
}

/// Kind of the children of a complex type.
///
/// The enum is necessary to manage the possibility to have a sequence or a choice a child. Other children
/// are [ElementNode]
#[derive(Debug)]
pub enum ComplexTypeChildKind {
    Element(ElementNode),
    /// Node sequnece. It stores the [RoNode] of the location of the information
    Sequence(Vec<ElementNode>),
    Choice(Vec<ComplexTypeChildKind>),
}

impl ElementNodeKind {
    /// Is the node a complex type
    pub fn is_complex_type(&self) -> bool {
        self.try_unwrap_complex_type().is_ok()
    }

    /// Is the node a native type
    #[allow(dead_code)]
    pub fn is_native(&self) -> bool {
        self.try_unwrap_native().is_ok()
    }

    /// Unwrap the [ElementNode] to the list of the children under the complex type
    ///
    /// Return error if the node is not complex
    pub fn try_unwrap_complex_type(&self) -> anyhow::Result<&Vec<ComplexTypeChildKind>> {
        if let Self::ComplexType(n) = self {
            return Ok(n);
        }
        Err(anyhow!("The node is not a complex type"))
    }

    /// Unwrap the [ElementNode] to the native type
    ///
    /// Return `None` if the node is not native
    pub fn try_unwrap_native(&self) -> anyhow::Result<&str> {
        if let Self::Native(s) = self {
            return Ok(s.as_str());
        }
        Err(anyhow!("The node is not a complex type"))
    }

    /// Unwrap the [ElementNode] to the list of the children under the complex type
    ///
    /// Panic if the node is not complex
    pub fn unwrap_complex_type(&self) -> &Vec<ComplexTypeChildKind> {
        let res = self.try_unwrap_complex_type();
        if let Err(e) = res {
            panic!("{}", e);
        }
        res.unwrap()
    }

    /// Unwrap the [ElementNode] to the native type
    ///
    /// Panic if the node is not native
    pub fn unwrap_native(&self) -> &str {
        let res = self.try_unwrap_native();
        if let Err(e) = res {
            panic!("{}", e);
        }
        res.unwrap()
    }

    pub fn try_find_child_with_tag_name(
        &self,
        tag_name: &str,
    ) -> anyhow::Result<Option<&ElementNode>> {
        let children = self.try_unwrap_complex_type()?;
        for c in children {
            match c {
                ComplexTypeChildKind::Element(e) => {
                    if e.has_name(tag_name) {
                        return Ok(Some(e));
                    }
                }
                ComplexTypeChildKind::Sequence(seq) => {
                    if let Some(e) = seq.iter().find(|e| e.has_name(tag_name)) {
                        return Ok(Some(e));
                    }
                }
                ComplexTypeChildKind::Choice(choices) => {
                    if let Some(e) = choices
                        .iter()
                        .find(|e| e.is_element() && e.unwrap_element().has_name(tag_name))
                    {
                        return Ok(Some(e.unwrap_element()));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Transform a [RoNode] to an [ElementNode]
    ///
    /// The function take care of the namespaces and also provide the elements from imported schemas
    fn try_from_roxml_node(
        node: &RoNode<'_, '_>,
        schema: &'static Schema<'static>,
    ) -> anyhow::Result<Self> {
        let mut res_node = None;
        let mut res_schema = schema;
        //println!("Schema node type {:?}", node.schema_node_type());
        if let Some(q_name) = node.schema_node_type() {
            // The type name is qualified with prefix
            if let Some(prefix) = q_name.prefix() {
                //println!("Prefix {:?}", prefix);
                //if prefix.as_ref() == "eCH-0010".as_bytes() {
                    //println!("eCH-0010");
                    //println!("Schema: {}", schema.target_namespace_name());
                    //print!("Subschema: {:?}", schema.sub_schema_nodes_with_name());
                //}
                match prefix.as_ref() {
                    // The prefix is for xmlschema
                    s if s == schema.xmlschema_namespace_name().as_bytes() => {
                        return Ok(ElementNodeKind::Native(
                            str::from_utf8(q_name.local_name().as_ref())
                                .unwrap()
                                .to_string(),
                        ));
                    }
                    // The prefix is for target namespace (e.g. in the current schema)
                    s if s == schema.target_namespace_name().as_bytes() => {
                        let n = schema.document().find_node_with_name(
                            str::from_utf8(q_name.local_name().as_ref()).unwrap(),
                        );
                        if n.is_some() {
                            res_node = Some(n.unwrap());
                        }
                    }
                    // The prefix is another namespace in the import
                    s if schema
                        .sub_schema_nodes_with_name()
                        .contains_key(str::from_utf8(s).unwrap()) =>
                    {
                        let ns_name = str::from_utf8(s).unwrap();
                        let sub_schema = schema.sub_schema_name(ns_name)?;
                        let doc = sub_schema.document();
                        let n = doc.find_node_with_name(
                            str::from_utf8(q_name.local_name().as_ref()).unwrap(),
                        );
                        if n.is_some() {
                            res_node = Some(n.unwrap());
                            res_schema = sub_schema;
                        }
                    }
                    // Not qualified -> the result remains none
                    _ => (),
                }
            }
        }
        // Type not in the attribute. Take the first relevant child
        else if let Some(fcn) = node.children().find(|n| n.is_used_schema_tag()) {
            res_node = Some(fcn);
        }
        match res_node {
            None => Err(anyhow!(
                "No type found for node {:?} with attributes: {:?}",
                node.node_tag_name(),
                node.attributes()
            )),
            Some(n) => {
                if n.is_schema_complex_type() {
                    let mut res = vec![];
                    // First children is the sequence containing the children
                    let seq_node = n.first_element_child().unwrap();
                    for c in seq_node.children().filter(|e| e.is_child_of_complex_type()) {
                        res.push(ComplexTypeChildKind::try_from_roxml_node(&c, res_schema)?)
                    }
                    return Ok(Self::ComplexType(res));
                }
                if n.is_schema_simple_type() {
                    return Ok(ElementNodeKind::Native(
                        n.native_type_from_simple_type(res_schema)?,
                    ));
                }
                Err(anyhow!(
                    "The node {} ist not a simple type or a complex type",
                    n.node_tag_name()
                ))
            }
        }
    }
}

impl ComplexTypeChildKind {
    /// Is the child an element
    pub fn is_element(&self) -> bool {
        self.try_unwrap_element().is_ok()
    }

    /// Is the child a sequence
    pub fn is_sequence(&self) -> bool {
        self.try_unwrap_sequence().is_ok()
    }

    /// Is the child a choice
    pub fn is_choice(&self) -> bool {
        self.try_unwrap_choice().is_ok()
    }

    /// Unwrap the [ComplexTypeChildKind] to an [ElementNode]
    ///
    /// Return error if the child is not an element
    fn try_unwrap_element(&self) -> anyhow::Result<&ElementNode> {
        if let Self::Element(n) = self {
            return Ok(n);
        }
        Err(anyhow!("The node is not a an element"))
    }

    /// Unwrap the [ComplexTypeChildKind] to a list of [ElementNode]
    ///
    /// Return error if the child is not an sequence
    fn try_unwrap_sequence(&self) -> anyhow::Result<&Vec<ElementNode>> {
        if let Self::Sequence(n) = self {
            return Ok(n);
        }
        Err(anyhow!("The node is not a sequence"))
    }

    /// Unwrap the [ComplexTypeChildKind] to a list of [ElementNode]
    ///
    /// Return error if the child is not an choice
    fn try_unwrap_choice(&self) -> anyhow::Result<&Vec<ComplexTypeChildKind>> {
        if let Self::Choice(n) = self {
            return Ok(n);
        }
        Err(anyhow!("The node is not a choice"))
    }

    /// Unwrap the [ComplexTypeChildKind] to an [ElementNode]
    ///
    /// Panic if the child is not an element
    pub fn unwrap_element(&self) -> &ElementNode {
        let res = self.try_unwrap_element();
        if let Err(e) = res {
            panic!("{}", e);
        }
        res.unwrap()
    }

    /// Unwrap the [ComplexTypeChildKind] to a list of [ElementNode]
    ///
    /// Panic if the child is not an sequence
    pub fn unwrap_sequence(&self) -> &Vec<ElementNode> {
        let res = self.try_unwrap_sequence();
        if let Err(e) = res {
            panic!("{}", e);
        }
        res.unwrap()
    }

    /// Unwrap the [ComplexTypeChildKind] to a list of [ElementNode]
    ///
    /// Panic if the child is not an choice
    fn unwrap_choice(&self) -> &Vec<ComplexTypeChildKind> {
        let res = self.try_unwrap_choice();
        if let Err(e) = res {
            panic!("{}", e);
        }
        res.unwrap()
    }

    /// Transform a [RoNode] to a [ComplexTypeChildKind]
    ///
    /// The entry should be the child under the sequence under the complex type
    fn try_from_roxml_node(
        node: &RoNode<'_, '_>,
        schema: &'static Schema<'static>,
    ) -> anyhow::Result<Self> {
        // Manage the case if it is an element
        if node.is_schema_element() {
            return Ok(Self::Element(ElementNode::try_from_roxml_node(
                node, schema,
            )?));
        }

        // Manage the case if it is choice
        if node.is_schema_choice() {
            let mut res = vec![];
            for c in node.children().filter(|e| e.is_child_of_complex_type()) {
                res.push(ComplexTypeChildKind::try_from_roxml_node(&c, schema)?);
            }
            return Ok(Self::Choice(res));
        }

        // Manage the case if it is a sequence
        if node.is_schema_sequence() {
            let mut res = vec![];
            for c in node.children().filter(|e| e.is_child_of_complex_type()) {
                res.push(ElementNode::try_from_roxml_node(&c, schema)?);
            }
            return Ok(Self::Sequence(res));
        }
        anyhow::bail!(
            "The node should be an element, a sequence or a choice, not {:?}",
            node.tag_name()
        )
    }
}

impl TryFrom<&'static Schema<'static>> for ElementNode {
    type Error = anyhow::Error;

    fn try_from(schema: &'static Schema<'static>) -> Result<Self, Self::Error> {
        let root = schema.root_element();
        let node = root
            .children()
            .find(|e| e.is_schema_element())
            .ok_or(anyhow!("First element not found"))?;
        Self::try_from_roxml_node(&node, schema)
    }
}

impl ElementNode {
    /// Get kind of node
    pub fn node_kind(&self) -> &ElementNodeKind {
        &self.node_kind
    }

    /// get name of node
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Check if the node has the name given
    pub fn has_name(&self, name: &str) -> bool {
        self.name() == name
    }

    /// Transform a [RoNode] to an [ElementNode]
    ///
    /// The entry should be a complexType, a native Element or a simple Type
    pub fn try_from_roxml_node(
        node: &RoNode<'_, '_>,
        schema: &'static Schema<'static>,
    ) -> anyhow::Result<Self> {
        let name = node.attr_name().ok_or(anyhow!(
            "Attribute name not found for {}",
            node.node_tag_name()
        ))?;
        Ok(Self {
            schema,
            name: name.to_string(),
            node_kind: ElementNodeKind::try_from_roxml_node(node, schema)?,
        })
    }
}

/// Trait to extend the functionalities of [RoDocument]
trait AdditionalMethodsRoxmlDocument<'a>: Sized {
    /// find a node in the document under the root with a given name
    fn find_node_with_name(&'a self, name: &str) -> Option<RoNode<'a, 'a>>;
}

/// Trait to extend the functionalities of [RoNode]
trait AdditionalMethodsRoxmlNode<'a>: Sized {
    /// Tagname of the node
    fn node_tag_name(&self) -> &'a str;
    /// Find an attribute in the node
    fn find_attribute(&'a self, name: &str) -> Option<&'a str>;

    fn attr_name(&'a self) -> Option<&'a str> {
        self.find_attribute("name")
    }

    /// Is a node with tag `element`
    fn is_schema_element(&self) -> bool {
        self.node_tag_name() == "element"
    }

    /// Is a node with tag `choice`
    fn is_schema_choice(&self) -> bool {
        self.node_tag_name() == "choice"
    }

    /// Is a node with tag `complexType`
    fn is_schema_complex_type(&self) -> bool {
        self.node_tag_name() == "complexType"
    }

    /// Is a node with tag `simpleType`
    fn is_schema_simple_type(&self) -> bool {
        self.node_tag_name() == "simpleType"
    }

    /// Is a node with tag `complexType`
    fn is_schema_sequence(&self) -> bool {
        self.node_tag_name() == "sequence"
    }

    fn is_used_schema_tag(&self) -> bool {
        self.is_schema_element()
            || self.is_schema_choice()
            || self.is_schema_complex_type()
            || self.is_schema_simple_type()
            || self.is_schema_sequence()
    }

    /// Is a node with tag `simpleType`, `complexType`, e.g. not a native type
    fn is_schema_type(&self) -> bool {
        self.is_schema_complex_type() || self.is_schema_simple_type()
    }

    /// Is a node is a relevant child of a complex type
    fn is_child_of_complex_type(&self) -> bool {
        self.is_schema_element() || self.is_schema_choice() || self.is_schema_sequence()
    }

    /// Get the value `minOccurs` of the node. Default is 1
    fn min_occurs(&'a self) -> usize {
        self.find_attribute("minOccurs")
            .map_or(1, |e| e.parse::<usize>().unwrap())
    }

    /// Get the value `maxOccurs` of the node. Default is 1. `"unbounded"` is set to `usize:MAX`
    fn max_occurs(&'a self) -> usize {
        self.find_attribute("maxOccurs").map_or(1, |e| match e {
            "unbounded" => usize::MAX,
            s => s.parse::<usize>().unwrap(),
        })
    }

    /// Get the attribute "type" of the node, as qualified name
    fn schema_node_type(&'a self) -> Option<QName> {
        self.find_attribute("type").map(|e| QName(e.as_bytes()))
    }

    /// Return a vector with the relevant children of a sequence or choice
    ///
    /// # Error
    /// If the children cannot be found
    fn children_of_sequence_or_choice(&'a self) -> anyhow::Result<Vec<Self>>;

    /// Return a vector with the relevant children of the complex type
    ///
    /// Take the list of tags under the sequence
    ///
    /// # Error
    /// If the children cannot be found
    fn children_complex_type(&'a self) -> anyhow::Result<Vec<Self>>;

    /// Return the native type behind a simple type
    ///
    /// # Error
    /// If the children cannot be found
    fn native_type_from_simple_type(
        &self,
        schema: &'static Schema<'static>,
    ) -> anyhow::Result<String>;
}

impl<'a> AdditionalMethodsRoxmlDocument<'a> for RoDocument<'a> {
    fn find_node_with_name(&'a self, name: &str) -> Option<RoNode<'a, 'a>> {
        self.root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some(name))
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

    fn children_of_sequence_or_choice(&'a self) -> anyhow::Result<Vec<Self>> {
        if !self.is_schema_sequence() && !self.is_schema_choice() {
            return Err(anyhow!(
                "The tag {} must be a a sequence or a choice",
                self.node_tag_name()
            ));
        }
        Ok(self
            .children()
            .filter(|e| e.is_child_of_complex_type())
            .collect())
    }

    fn children_complex_type(&'a self) -> anyhow::Result<Vec<Self>> {
        if !self.is_schema_complex_type() {
            return Err(anyhow!(
                "The tag {} must be a complex type",
                self.node_tag_name()
            ));
        }
        let seq = self
            .first_element_child()
            .ok_or(anyhow!("Missing first child"))?;
        if !seq.is_schema_sequence() {
            return Err(anyhow!("The first child of the tag must be a sequence"));
        }
        Ok(seq
            .children()
            .filter(|e| e.is_child_of_complex_type())
            .collect())
    }

    fn native_type_from_simple_type(
        &self,
        schema: &'static Schema<'static>,
    ) -> anyhow::Result<String> {
        if !self.is_schema_simple_type() {
            return Err(anyhow!(
                "The tag {} must be a simple type",
                self.node_tag_name()
            ));
        }
        let restriction = self
        .children()
        .find(|e| e.node_tag_name() == "restriction")
        .ok_or(anyhow!("Simple type tag {} must have a child with tag restriction. Other constructs are not implemented", self.node_tag_name()))?;
        let base = QName(
            restriction
                .find_attribute("base")
                .ok_or(anyhow!("The atribute base is missing for restriction."))?
                .as_bytes(),
        );
        if let Some(prefix) = base.prefix() {
            match prefix.as_ref() {
                // The prefix of base is for xmlschema
                s if s == schema.xmlschema_namespace_name().as_bytes() => {
                    return Ok(str::from_utf8(base.local_name().as_ref())
                        .unwrap()
                        .to_string());
                }
                // The prefix is for target namespace (e.g. in the current schema)
                s if s == schema.target_namespace_name().as_bytes() => {
                    match schema
                        .document()
                        .find_node_with_name(str::from_utf8(base.local_name().as_ref()).unwrap())
                    {
                        Some(n) => return n.native_type_from_simple_type(schema),
                        None => {
                            return Err(anyhow!(
                                "Simple Type {:?} not found in schema {}",
                                base,
                                schema.xmlschema_namespace_name()
                            ))
                        }
                    }
                }
                // The prefix is another namespace in the import
                s if schema
                    .sub_schema_nodes_with_name()
                    .contains_key(str::from_utf8(s).unwrap()) =>
                {
                    let ns_name = str::from_utf8(s).unwrap();
                    let sub_schema = schema.sub_schema_name(ns_name)?;
                    let doc = sub_schema.document();
                    match doc
                        .find_node_with_name(str::from_utf8(base.local_name().as_ref()).unwrap())
                    {
                        Some(n) => return n.native_type_from_simple_type(sub_schema),
                        None => {
                            return Err(anyhow!(
                                "Simple Type {:?} not found in schema {}",
                                base,
                                schema.xmlschema_namespace_name()
                            ))
                        }
                    }
                }
                // Not qualified -> the result remains none
                _ => return Err(anyhow!("Simple Type {:?} has no valid prefix", base)),
            }
        }
        Err(anyhow!(
            "Simple Type {:?} has no prefix. Prefix expected",
            base
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::schema::{
        test_schemas::{get_schema_test_1, get_schema_test_2, get_schema_test_3},
        SchemaKind,
    };
    use super::*;

    #[test]
    fn test_schema_1() {
        let tree_res = ElementNode::try_from(get_schema_test_1());
        assert!(tree_res.is_ok());
        let tree = tree_res.unwrap();
        assert_eq!(tree.name, "tests");
        assert!(tree.node_kind.is_complex_type());
        let cs = tree.node_kind.try_unwrap_complex_type().unwrap();
        assert_eq!(cs.len(), 3);
        assert!(cs[0].is_element());
        assert_eq!(cs[0].try_unwrap_element().unwrap().name, "valueString");
        assert!(cs[1].is_element());
        assert_eq!(cs[1].try_unwrap_element().unwrap().name, "valueBoolean");
        assert!(cs[2].is_element());
        assert_eq!(cs[2].try_unwrap_element().unwrap().name, "valueInt");
    }

    #[test]
    fn test_sub_schema_2() {
        let tree_res = ElementNode::try_from(get_schema_test_2());
        assert!(tree_res.is_ok());
        let tree = tree_res.unwrap();
        assert_eq!(tree.name, "tests");
        assert!(tree.node_kind.is_complex_type());
        let cs = tree.node_kind.try_unwrap_complex_type().unwrap();
        assert_eq!(cs.len(), 5);
        assert!(cs[0].is_element());
        assert_eq!(cs[0].try_unwrap_element().unwrap().name, "valueString");
        assert!(cs[1].is_element());
        assert_eq!(cs[1].try_unwrap_element().unwrap().name, "valueBoolean");
        assert!(cs[2].is_element());
        assert_eq!(cs[2].try_unwrap_element().unwrap().name, "complexType");
        assert!(cs[3].is_element());
        assert_eq!(cs[3].try_unwrap_element().unwrap().name, "valueList");
        assert!(cs[4].is_element());
        assert_eq!(cs[4].try_unwrap_element().unwrap().name, "valueInt");
        let ct = cs[2].try_unwrap_element().unwrap();
        let cs2 = ct.node_kind.try_unwrap_complex_type().unwrap();
        assert_eq!(cs2.len(), 2);
        assert!(cs2[0].is_element());
        assert_eq!(cs2[0].try_unwrap_element().unwrap().name, "ctString");
        assert!(cs2[1].is_element());
        assert_eq!(cs2[1].try_unwrap_element().unwrap().name, "csToto");
    }

    #[test]
    fn test_sub_schema_3() {
        let tree_res = ElementNode::try_from(get_schema_test_3());
        assert!(tree_res.is_ok());
        let tree = tree_res.unwrap();
        assert_eq!(tree.name, "tests");
        assert!(tree.node_kind.is_complex_type());
        let cs = tree.node_kind.try_unwrap_complex_type().unwrap();
        assert_eq!(cs.len(), 2);
        assert!(cs[0].is_element());
        assert_eq!(cs[0].try_unwrap_element().unwrap().name, "valueString");
        assert!(cs[1].is_element());
        assert_eq!(cs[1].try_unwrap_element().unwrap().name, "complexType");
        let ct = cs[1].try_unwrap_element().unwrap();
        let cs2 = ct.node_kind.try_unwrap_complex_type().unwrap();
        assert_eq!(cs2.len(), 4);
        assert!(cs2[0].is_element());
        assert_eq!(cs2[0].try_unwrap_element().unwrap().name, "ctString");
        assert!(cs2[1].is_sequence());
        let seq = cs2[1].try_unwrap_sequence().unwrap();
        assert_eq!(seq.len(), 2);
        assert!(seq[0].node_kind.is_native());
        assert_eq!(seq[0].name, "seqString1");
        assert!(seq[1].node_kind.is_native());
        assert_eq!(seq[1].name, "seqString2");
        assert!(cs2[2].is_choice());
        let choice = cs2[2].try_unwrap_choice().unwrap();
        assert_eq!(choice.len(), 2);
        assert!(choice[0].is_element());
        assert!(choice[0].unwrap_element().node_kind.is_native());
        assert_eq!(choice[0].unwrap_element().name, "choiceString1");
        assert!(choice[0].is_element());
        assert!(choice[0].unwrap_element().node_kind.is_native());
        assert_eq!(choice[0].unwrap_element().name, "choiceString2");
        assert!(cs2[3].is_element());
        assert_eq!(cs2[3].try_unwrap_element().unwrap().name, "ctToto");
    }

    #[test]
    fn test_has_name() {
        let tree_res = ElementNode::try_from(get_schema_test_3());
        let tree = tree_res.unwrap();
        assert!(tree.has_name("tests"));
        assert!(!tree.has_name("toto"))
    }

    #[test]
    fn test_try_find_child_with_tag_name() {
        let tree_res = ElementNode::try_from(get_schema_test_3());
        let tree = tree_res.unwrap();
        assert!(tree
            .node_kind
            .try_find_child_with_tag_name("valueString")
            .unwrap()
            .unwrap()
            .has_name("valueString"));
        assert!(tree
            .node_kind
            .try_find_child_with_tag_name("complexType")
            .unwrap()
            .unwrap()
            .has_name("complexType"));
        assert!(tree
            .node_kind
            .try_find_child_with_tag_name("toto")
            .unwrap()
            .is_none());
        let ct = tree.node_kind.try_unwrap_complex_type().unwrap()[1]
            .try_unwrap_element()
            .unwrap();
        assert!(ct
            .node_kind
            .try_find_child_with_tag_name("ctString")
            .unwrap()
            .unwrap()
            .has_name("ctString"));
        assert!(ct
            .node_kind
            .try_find_child_with_tag_name("seqString1")
            .unwrap()
            .unwrap()
            .has_name("seqString1"));
        assert!(ct
            .node_kind
            .try_find_child_with_tag_name("seqString2")
            .unwrap()
            .unwrap()
            .has_name("seqString2"));
        assert!(ct
            .node_kind
            .try_find_child_with_tag_name("choiceString1")
            .unwrap()
            .unwrap()
            .has_name("choiceString1"));
        assert!(ct
            .node_kind
            .try_find_child_with_tag_name("choiceString2")
            .unwrap()
            .unwrap()
            .has_name("choiceString2"));
        assert!(ct
            .node_kind
            .try_find_child_with_tag_name("ctToto")
            .unwrap()
            .unwrap()
            .has_name("ctToto"));
    }

    #[test]
    fn test_config() {
        let node_res = ElementNode::try_from(SchemaKind::Config.schema());
        if let Err(e) = &node_res {
            println!("{}", e)
        }
        assert!(node_res.is_ok());
    }

    #[test]
    fn test_decrypt() {
        let node_res = ElementNode::try_from(SchemaKind::Decrypt.schema());
        if let Err(e) = &node_res {
            println!("{}", e)
        }
        assert!(node_res.is_ok());
    }

    #[test]
    fn test_0222() {
        let node_res = ElementNode::try_from(SchemaKind::Ech0222.schema());
        if let Err(e) = &node_res {
            println!("{}", e)
        }
        assert!(node_res.is_ok());
    }

    #[test]
    fn test_0110() {
        let node_res = ElementNode::try_from(SchemaKind::Ech0110.schema());
        if let Err(e) = &node_res {
            println!("{}", e)
        }
        assert!(node_res.is_ok());
    }

    #[test]
    fn test_0010() {
        let node_res = ElementNode::try_from(SchemaKind::Ech0010.schema());
        if let Err(e) = &node_res {
            println!("{}", e)
        }
        assert!(node_res.is_ok());
    }
}

#[cfg(test)]
mod test_additional_method_node {
    use super::super::schema::SchemaKind;
    use super::*;

    fn schema_config<'a>() -> &'a Schema<'a> {
        SchemaKind::Config.schema()
    }

    #[test]
    fn test_tag_name() {
        let xsd = schema_config();
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
        let xsd = schema_config();
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
        let xsd = schema_config();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.node_tag_name() == "element")
            .unwrap();
        assert!(n1.is_schema_element());
        assert!(!n1.first_element_child().unwrap().is_schema_element());
    }

    #[test]
    fn test_is_sequence() {
        let xsd = schema_config();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some("dwellingAddressType"))
            .unwrap()
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap();
        assert!(n1.is_schema_sequence());
        assert!(n1.is_child_of_complex_type());
    }

    #[test]
    fn test_is_choice() {
        let xsd = schema_config();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some("dwellingAddressType"))
            .unwrap()
            .first_element_child()
            .unwrap()
            .first_element_child()
            .unwrap()
            .next_sibling_element()
            .unwrap();
        assert!(n1.is_schema_choice());
        assert!(n1.is_child_of_complex_type());
    }

    #[test]
    fn test_is_complex_type() {
        let xsd = schema_config();
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
        let xsd = schema_config();
        let n1 = xsd.root_element().first_element_child().unwrap();
        assert!(!xsd.root_element().is_schema_simple_type());
        assert!(n1.is_schema_simple_type());
    }

    #[test]
    fn test_max_occurs() {
        let xsd = schema_config();
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
        let xsd = schema_config();
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
        let xsd = schema_config();
        let doc = xsd.document();
        assert!(doc
            .find_node_with_name("electoralBoardType")
            .unwrap()
            .is_schema_complex_type());
        assert!(doc
            .find_node_with_name("configuration")
            .unwrap()
            .is_schema_element());
        assert!(doc
            .find_node_with_name("languageType")
            .unwrap()
            .is_schema_simple_type());
        assert!(doc.find_node_with_name("toto").is_none())
    }

    #[test]
    fn test_schema_node_type() {
        let xsd = schema_config();
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

    #[test]
    fn test_children_complex_type_sequence_choice() {
        let xsd = schema_config();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some("dwellingAddressType"))
            .unwrap();
        let cs1_res = n1.children_complex_type();
        assert!(cs1_res.is_ok());
        let cs1 = cs1_res.unwrap();
        assert_eq!(cs1.len(), 3);
        assert!(cs1[0].is_schema_sequence());
        assert!(cs1[1].is_schema_choice());
        assert!(cs1[2].is_schema_element());
        let cs1_0_res = cs1[0].children_of_sequence_or_choice();
        assert!(cs1_0_res.is_ok());
        let cs1_0 = cs1_0_res.unwrap();
        assert_eq!(cs1_0.len(), 2);
        assert!(cs1_0[0].is_schema_element());
        assert_eq!(cs1_0[0].attr_name(), Some("street"));
        assert!(cs1_0[1].is_schema_element());
        assert_eq!(cs1_0[1].attr_name(), Some("houseNumber"));
        let cs1_1_res = cs1[1].children_of_sequence_or_choice();
        assert!(cs1_1_res.is_ok());
        let cs1_1 = cs1_1_res.unwrap();
        assert_eq!(cs1_1.len(), 2);
        assert!(cs1_1[0].is_schema_element());
        assert_eq!(cs1_1[0].attr_name(), Some("swissZipCode"));
        assert!(cs1_1[1].is_schema_element());
        assert_eq!(cs1_1[1].attr_name(), Some("foreignZipCode"));
    }

    #[test]
    fn test_native_type_from_simple_type() {
        let xsd = schema_config();
        let n1 = xsd.root_element().first_element_child().unwrap();
        assert_eq!(
            n1.native_type_from_simple_type(xsd).unwrap(),
            "string".to_string()
        );
        let n2 = n1.next_sibling_element().unwrap();
        assert_eq!(
            n2.native_type_from_simple_type(xsd).unwrap(),
            "token".to_string()
        );
    }

    #[test]
    fn test_native_type_from_simple_type_2() {
        let xsd = SchemaKind::Ech0155.schema();
        let n1 = xsd
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some("identifierType"))
            .unwrap();
        assert_eq!(
            n1.native_type_from_simple_type(xsd).unwrap(),
            "token".to_string()
        );
        let n2 = xsd
            .root_element()
            .children()
            .find(|e| e.find_attribute("name") == Some("domainOfInfluenceIdentificationType"))
            .unwrap();
        assert_eq!(
            n2.native_type_from_simple_type(xsd).unwrap(),
            "token".to_string()
        );
    }
}
