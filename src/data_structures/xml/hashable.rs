use super::{
    hashable_no_value,
    schema::{Schema, SchemaKind},
    schema_tree::{ComplexTypeChildKind, ElementNode},
};
use anyhow::{anyhow, Context};
use quick_xml::{
    events::Event,
    name::{Namespace, QName, ResolveResult::*},
    reader::NsReader,
};
use rust_ev_crypto_primitives::{ByteArray, Decode, HashableMessage, RecursiveHashTrait};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    str,
};

/// An struct to hash the xml file according to the specification of Swiss Post
/// TODO: the options (xs:choice) are missing
pub struct XMLFileHashable {
    file: PathBuf,
    schema: &'static Schema<'static>,
    exclusion: String,
}

/// An struct to hash a node in an xml file according to the specification of Swiss Post
struct NodeHashable<'a> {
    reader: &'a mut NsReader<BufReader<File>>,
    tag_name: &'a str,
    schema_node: &'a ElementNode,
    exclusion: String,
}

impl XMLFileHashable {
    /// Create a new [XMLFileHashable]
    ///
    /// `exclusion` contains the name of the tag to be excluded. The tag should be exactly
    /// the same than in xml file (with or without namespaces)
    pub fn new(xml: &Path, schema_kind: &SchemaKind, exclusion: &str) -> Self {
        Self::new_with_schema(xml, schema_kind.get_schema(), exclusion)
    }

    pub fn new_with_schema(xml: &Path, schema: &'static Schema<'static>, exclusion: &str) -> Self {
        Self {
            file: xml.to_path_buf(),
            schema,
            exclusion: exclusion.to_string(),
        }
    }
}

impl RecursiveHashTrait for XMLFileHashable {
    type Error = anyhow::Error;

    fn try_hash(&self) -> Result<ByteArray, Self::Error> {
        let mut reader = NsReader::from_file(&self.file).map_err(|e| {
            anyhow!(e).context(format!(
                "Error creating xml reader for file {}",
                self.file.to_str().unwrap()
            ))
        })?;
        let mut buf = Vec::new();
        let schema_node = ElementNode::try_from(self.schema)?;
        let _ns = self.schema.target_namespace_name().as_bytes();
        loop {
            //match reader.read_resolved_event_into(/*&mut buf*/).unwrap() {
            match reader.read_resolved_event_into(&mut buf) {
                Ok((Bound(Namespace(_ns)), Event::Start(e))) => {
                    let tag_local_name = e.local_name();
                    let tag_name = str::from_utf8(tag_local_name.as_ref()).unwrap();
                    if tag_name == schema_node.name() {
                        return NodeHashable::new(
                            &schema_node,
                            tag_name,
                            &mut reader,
                            &self.exclusion,
                        )
                        .try_hash();
                    }
                }
                Ok((_, Event::Eof)) => panic!("tag {} not found", schema_node.name()),
                Ok(_) => (),
                Err(e) => return Err(anyhow!(e).context("Error reader in try_hash")),
            }
            buf.clear();
        }
    }
}

impl<'a> NodeHashable<'a> {
    /// Create a new [NodeHashable]
    ///
    /// `exclusion` contains the name of the tag to be excluded. The tag should be exactly
    /// the same than in xml file (with or without namespaces)
    fn new(
        schema_node: &'a ElementNode,
        tag_name: &'a str,
        reader: &'a mut NsReader<BufReader<File>>,
        exclusion: &str,
    ) -> Self {
        Self {
            reader,
            tag_name,
            schema_node,
            exclusion: exclusion.to_string(),
        }
    }

    /// Hash a native type
    fn hash_native_type(&mut self, native_type: &str) -> anyhow::Result<ByteArray> {
        let mut buf = Vec::new();
        match self.reader.read_event_into(&mut buf) {
            Ok(Event::Text(b)) => Ok(NativeTypeConverter::new(
                b.unescape().unwrap().into_owned().as_str(),
                native_type,
            )?
            .to_hashable()?
            .hash()),
            Ok(e) => Err(anyhow!("Text expected. {:?} found", e)),
            Err(e) => Err(anyhow!(e).context("Error in hash_native_type getting the type")),
        }
    }

    /// Calculate the hash value of the child of the node given by `tag_name`
    ///
    /// # Error
    /// If the child is not found or an error during the calulation
    fn get_hash_from_child(&mut self, tag_name: &str) -> anyhow::Result<ByteArray> {
        let schema_node = match self
            .schema_node
            .node_kind()
            .try_find_child_with_tag_name(tag_name)?
        {
            Some(e) => e,
            None => return Err(anyhow!("tag {} not found in xsd", tag_name)),
        };
        NodeHashable::new(
            schema_node,
            schema_node.name(),
            self.reader,
            &self.exclusion,
        )
        .try_hash()
    }

    fn push_hashed_from_element_node(
        &self,
        hashables: &mut Vec<HashableMessage>,
        element_node: &ElementNode,
        hashed_children: &HashMap<String, Vec<ByteArray>>,
    ) {
        if hashed_children.contains_key(element_node.name()) {
            let values = hashed_children.get(element_node.name()).unwrap();
            if values.len() == 1 {
                hashables.push(HashableMessage::Hashed(values[0].clone()));
            } else {
                let l: Vec<HashableMessage> = values
                    .iter()
                    .map(|e| HashableMessage::Hashed(e.clone()))
                    .collect();
                hashables.push(HashableMessage::Hashed(HashableMessage::from(l).hash()));
            }
        } else {
            let exclusion_local_name = QName(self.exclusion.as_bytes()).local_name();
            let exclusion_name = str::from_utf8(exclusion_local_name.as_ref()).unwrap();
            if exclusion_name != element_node.name() {
                hashables.push(HashableMessage::Hashed(
                    hashable_no_value(element_node.name()).hash(),
                ));
            }
        }
    }

    fn push_hashed_from_sequence(
        &self,
        hashables: &mut Vec<HashableMessage>,
        element_nodes: &[ElementNode],
        hashed_children: &HashMap<String, Vec<ByteArray>>,
    ) {
        for e in element_nodes {
            self.push_hashed_from_element_node(hashables, e, hashed_children)
        }
    }

    fn push_hashed_from_choices(
        &self,
        hashables: &mut Vec<HashableMessage>,
        element_nodes: &[ElementNode],
        hashed_children: &HashMap<String, Vec<ByteArray>>,
    ) {
        match element_nodes
            .iter()
            .find(|e| hashed_children.contains_key(e.name()))
        {
            Some(e) => self.push_hashed_from_element_node(hashables, e, hashed_children),
            None => {
                self.push_hashed_from_element_node(hashables, &element_nodes[0], hashed_children)
            }
        }
    }

    /// Calculate the hash value from all the hashed children, according to the specification of Swiss Post
    ///
    /// It reads the schema in parallel in order to get the correct order of elements, and to add the value for
    /// missing entries.
    ///
    /// # Error
    /// If an error occured during the calculation
    fn hash_hashed_children(
        &self,
        hashed_children: &HashMap<String, Vec<ByteArray>>,
    ) -> anyhow::Result<ByteArray> {
        let mut hashables: Vec<HashableMessage> = vec![];
        for c in self.schema_node.node_kind().try_unwrap_complex_type()? {
            match c {
                ComplexTypeChildKind::Element(e) => {
                    self.push_hashed_from_element_node(&mut hashables, e, hashed_children)
                }
                ComplexTypeChildKind::Sequence(seq) => {
                    self.push_hashed_from_sequence(&mut hashables, seq, hashed_children)
                }
                ComplexTypeChildKind::Choice(choices) => {
                    self.push_hashed_from_choices(&mut hashables, choices, hashed_children)
                }
            }
        }
        Ok(HashableMessage::from(hashables).hash())
    }

    /// Hash a complex type
    fn hash_complex_type(&mut self) -> anyhow::Result<ByteArray> {
        let mut buf = Vec::new();
        let mut hm: HashMap<String, Vec<ByteArray>> = HashMap::new();
        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let tag_local_name = e.local_name();
                    let tag_name = str::from_utf8(tag_local_name.as_ref())?;
                    if e.name() != QName(self.exclusion.as_bytes()) {
                        let hash = self.get_hash_from_child(tag_name)?;
                        if !hm.contains_key(tag_name) {
                            hm.insert(tag_name.to_string(), vec![]);
                        }
                        hm.get_mut(tag_name).unwrap().push(hash);
                    }
                }
                Ok(Event::End(e)) => {
                    if e.local_name().as_ref() == self.tag_name.as_bytes() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow!(e).context("Error in hash_complex_type getting the type"));
                }
            }
            buf.clear();
        }
        self.hash_hashed_children(&hm)
    }

    /// Try to hash the node
    fn try_hash(&mut self) -> anyhow::Result<ByteArray> {
        if self.schema_node.node_kind().is_complex_type() {
            self.hash_complex_type()
        } else {
            self.hash_native_type(self.schema_node.node_kind().try_unwrap_native()?)
        }
    }
}

#[derive(Debug, PartialEq)]
enum NativeTypeConverter {
    Boolean(String),
    Numeric(String),
    Binary(String),
    String(String),
}

impl NativeTypeConverter {
    pub fn new(value: &str, native_type: &str) -> anyhow::Result<Self> {
        let res = value.to_string();
        Ok(match native_type {
            "boolean" => Self::Boolean(res),
            "integer" => Self::Numeric(res),
            "long" => Self::Numeric(res),
            "int" => Self::Numeric(res),
            "short" => Self::Numeric(res),
            "byte" => Self::Numeric(res),
            "nonNegativeInteger" => Self::Numeric(res),
            "positiveInteger" => Self::Numeric(res),
            "unsignedLong" => Self::Numeric(res),
            "unsignedInt" => Self::Numeric(res),
            "unsignedShort" => Self::Numeric(res),
            "unsignedByte" => Self::Numeric(res),
            "negativeInteger" => Self::Numeric(res),
            "nonPositiveInteger" => Self::Numeric(res),
            "base64Binary" => Self::Binary(res),
            "string" => Self::String(res),
            "normalizedString" => Self::String(res),
            "dateTime" => Self::String(res),
            "date" => Self::String(res),
            "token" => Self::String(res),
            _ => {
                return Err(anyhow!(
                    "Error creating NativeTypeConverter: type {} unknowm",
                    native_type
                ))
            }
        })
    }

    fn to_hashable(&self) -> anyhow::Result<HashableMessage<'_>> {
        match self {
            Self::Boolean(b) => {
                let res = match b.as_str() {
                    "true" => "true",
                    "false" => "false",
                    "0" => "true",
                    "1" => "false",
                    _ => return Err(anyhow!("Value {} is not a correct boolean", b)),
                };
                Ok(HashableMessage::from(res))
            }
            Self::Numeric(b) => Ok(HashableMessage::from(
                b.parse::<usize>()
                    .context(format!("{b} is not a valid numeric"))?,
            )),
            Self::Binary(b) => Ok(HashableMessage::from(
                ByteArray::base64_decode(b.as_str())
                    .context(format!("{b} is not a valid byte array"))?,
            )),
            Self::String(b) => Ok(HashableMessage::from(b)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::schema::{
        test_schemas::{get_schema_test_1, get_schema_test_2, get_schema_test_3},
        SchemaKind,
    };
    use super::*;
    use crate::config::test::{test_dataset_setup_path, test_xml_path};

    #[test]
    fn test_1_schema_1() {
        let xml = test_xml_path().join("test_1_schema_1.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_1(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from("true"),
            HashableMessage::from(10usize),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_1_schema_1_with_exclusion() {
        let xml = test_xml_path().join("test_1_schema_1.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_1(), "valueInt");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from("true"),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_1_schema_1_qualified() {
        let xml = test_xml_path().join("test_1_schema_1_qualified.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_1(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from("true"),
            HashableMessage::from(10usize),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_1_schema_1_qualified_with_exclusion() {
        let xml = test_xml_path().join("test_1_schema_1_qualified.xml");
        let xml_hashable =
            XMLFileHashable::new_with_schema(&xml, get_schema_test_1(), "toto:valueBoolean");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from(10usize),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_2_schema_1() {
        let xml = test_xml_path().join("test_2_schema_1.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_1(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from("no valueBoolean value"),
            HashableMessage::from(10usize),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_3_schema_1() {
        let xml = test_xml_path().join("test_3_schema_1.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_1(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from("true"),
            HashableMessage::from(10usize),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_1_schema_2() {
        let xml = test_xml_path().join("test_1_schema_2.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_2(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from("true"),
            HashableMessage::from(vec![
                HashableMessage::from("12345"),
                HashableMessage::from("toto"),
            ]),
            HashableMessage::from(5usize),
            HashableMessage::from(10usize),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_2_schema_2() {
        let xml = test_xml_path().join("test_2_schema_2.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_2(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from("true"),
            HashableMessage::from(vec![
                HashableMessage::from("12345"),
                HashableMessage::from("toto"),
            ]),
            HashableMessage::from(vec![
                HashableMessage::from(1usize),
                HashableMessage::from(2usize),
                HashableMessage::from(3usize),
                HashableMessage::from(4usize),
                HashableMessage::from(5usize),
            ]),
            HashableMessage::from(10usize),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_1_schema_3() {
        let xml = test_xml_path().join("test_1_schema_3.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_3(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from(vec![
                HashableMessage::from("12345"),
                HashableMessage::from("seq1"),
                HashableMessage::from("seq2"),
                HashableMessage::from("choice1"),
                HashableMessage::from("toto"),
            ]),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_2_schema_3() {
        let xml = test_xml_path().join("test_2_schema_3.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_3(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from(vec![
                HashableMessage::from("12345"),
                HashableMessage::from("seq1"),
                HashableMessage::from("seq2"),
                HashableMessage::from("choice"),
                HashableMessage::from("toto"),
            ]),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_3_schema_3() {
        let xml = test_xml_path().join("test_3_schema_3.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_3(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from(vec![
                HashableMessage::from("12345"),
                HashableMessage::from("seq1"),
                HashableMessage::from("seq2"),
                HashableMessage::from("no choiceString1 value"),
                HashableMessage::from("toto"),
            ]),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_4_schema_3() {
        let xml = test_xml_path().join("test_4_schema_3.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_3(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from(vec![
                HashableMessage::from("12345"),
                HashableMessage::from("no seqString1 value"),
                HashableMessage::from("no seqString2 value"),
                HashableMessage::from("no choiceString1 value"),
                HashableMessage::from("toto"),
            ]),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_5_schema_3() {
        let xml = test_xml_path().join("test_5_schema_3.xml");
        let xml_hashable = XMLFileHashable::new_with_schema(&xml, get_schema_test_3(), "");
        let expected = HashableMessage::from(vec![
            HashableMessage::from("test"),
            HashableMessage::from(vec![
                HashableMessage::from("12345"),
                HashableMessage::from("seq1"),
                HashableMessage::from("no seqString2 value"),
                HashableMessage::from("no choiceString1 value"),
                HashableMessage::from("toto"),
            ]),
        ])
        .hash();
        assert_eq!(xml_hashable.try_hash().unwrap(), expected)
    }

    #[test]
    fn test_config() {
        let xml = test_dataset_setup_path()
            .join("setup")
            .join("configuration-anonymized.xml");
        let xml_hashable =
            XMLFileHashable::new_with_schema(&xml, SchemaKind::Config.get_schema(), "");
        let hash = xml_hashable.try_hash();
        println!("{:?}", hash.err());
        assert!(xml_hashable.try_hash().is_ok())
    }
}

#[cfg(test)]
mod test_converter {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(
            NativeTypeConverter::new("toto", "boolean").unwrap(),
            NativeTypeConverter::Boolean("toto".to_string())
        );
        assert_eq!(
            NativeTypeConverter::new("toto", "integer").unwrap(),
            NativeTypeConverter::Numeric("toto".to_string())
        );
        assert_eq!(
            NativeTypeConverter::new("toto", "nonNegativeInteger").unwrap(),
            NativeTypeConverter::Numeric("toto".to_string())
        );
        assert_eq!(
            NativeTypeConverter::new("toto", "positiveInteger").unwrap(),
            NativeTypeConverter::Numeric("toto".to_string())
        );
        assert_eq!(
            NativeTypeConverter::new("toto", "string").unwrap(),
            NativeTypeConverter::String("toto".to_string())
        );
        assert_eq!(
            NativeTypeConverter::new("toto", "base64Binary").unwrap(),
            NativeTypeConverter::Binary("toto".to_string())
        );
        assert!(NativeTypeConverter::new("toto", "toto").is_err());
    }

    #[test]
    fn test_to_hashable_boolean() {
        assert_eq!(
            NativeTypeConverter::new("true", "boolean")
                .unwrap()
                .to_hashable()
                .unwrap(),
            HashableMessage::from("true")
        );
        assert_eq!(
            NativeTypeConverter::new("false", "boolean")
                .unwrap()
                .to_hashable()
                .unwrap(),
            HashableMessage::from("false")
        );
        assert_eq!(
            NativeTypeConverter::new("0", "boolean")
                .unwrap()
                .to_hashable()
                .unwrap(),
            HashableMessage::from("true")
        );
        assert_eq!(
            NativeTypeConverter::new("1", "boolean")
                .unwrap()
                .to_hashable()
                .unwrap(),
            HashableMessage::from("false")
        );
        assert!(NativeTypeConverter::new("2", "boolean")
            .unwrap()
            .to_hashable()
            .is_err());
        assert!(NativeTypeConverter::new("toto", "boolean")
            .unwrap()
            .to_hashable()
            .is_err());
    }
}
