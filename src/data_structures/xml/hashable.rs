use crate::data_structures::xml::{hashable, schema};

use super::{
    hashable_no_value,
    schema::{Schema, SchemaKind},
    schema_iterator::ElementNode,
};
use anyhow::{anyhow, Context};
use quick_xml::{
    events::Event,
    name::{self, Namespace, ResolveResult::*},
    reader::NsReader,
};
use rust_ev_crypto_primitives::{
    byte_array::{ByteArray, Decode},
    hashing::{HashTrait, HashableMessage},
};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    str,
};

pub struct XMLFileHashable<'a> {
    file: PathBuf,
    schema: &'a Schema<'a>,
}

struct NodeHashable<'a> {
    reader: &'a mut NsReader<BufReader<File>>,
    tag_name: &'a str,
    schema_node: &'a ElementNode<'a>,
}

impl<'a> XMLFileHashable<'a> {
    pub fn new(xml: &Path, schema_kind: &'a SchemaKind) -> Self {
        Self {
            file: xml.to_path_buf(),
            schema: schema_kind.get_schema(),
        }
    }
}

impl<'a> HashTrait for XMLFileHashable<'a> {
    type Error = anyhow::Error;

    fn try_hash(&self) -> Result<ByteArray, Self::Error> {
        let mut reader = NsReader::from_file(&self.file).map_err(|e| {
            anyhow!(e).context(format!(
                "Error creating xml reader for file {}",
                self.file.to_str().unwrap()
            ))
        })?;
        let mut buf = Vec::new();
        let schema_node = ElementNode::from(self.schema);
        let ns = self.schema.target_namespace_name().as_bytes();
        loop {
            //match reader.read_resolved_event_into(/*&mut buf*/).unwrap() {
            match reader.read_resolved_event_into(&mut buf) {
                Ok((Bound(Namespace(ns)), Event::Start(e))) => {
                    let tag_local_name = e.local_name();
                    let tag_name = str::from_utf8(tag_local_name.as_ref()).unwrap();
                    if tag_name == schema_node.name() {
                        return NodeHashable::new(&schema_node, tag_name, &mut reader).try_hash();
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
    fn new(
        schema_node: &'a ElementNode<'a>,
        tag_name: &'a str,
        reader: &'a mut NsReader<BufReader<File>>,
    ) -> Self {
        Self {
            reader,
            tag_name,
            schema_node,
        }
    }

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

    fn get_hash_from_child(&mut self, tag_name: &str) -> anyhow::Result<ByteArray> {
        let children = self.schema_node.children()?;
        let schema_node = children
            .iter()
            .find(|e| e.name() == tag_name)
            .ok_or(anyhow!("tag {} not found in xsd", tag_name))?;
        NodeHashable::new(schema_node, schema_node.name(), self.reader).try_hash()
    }

    fn hash_hashed_childern(
        &self,
        hashed_children: &HashMap<String, Vec<ByteArray>>,
    ) -> anyhow::Result<ByteArray> {
        let mut hashable: Vec<HashableMessage> = vec![];
        for c in self.schema_node.children()? {
            if hashed_children.contains_key(c.name()) {
                let values = hashed_children.get(c.name()).unwrap();
                if values.len() == 1 {
                    hashable.push(HashableMessage::Hashed(values[0].clone()));
                }
                else {
                    let l: Vec<HashableMessage> = values.iter().map(|e| HashableMessage::Hashed(e.clone())).collect();
                    hashable.push(HashableMessage::Hashed(HashableMessage::from(l).hash()));
                }
            } else {
                hashable.push(HashableMessage::Hashed(hashable_no_value(c.name()).hash()));
            }
        }
        Ok(HashableMessage::from(hashable).hash())
    }

    fn hash_complex_type(&mut self) -> anyhow::Result<ByteArray> {
        let mut buf = Vec::new();
        let mut hm: HashMap<String, Vec<ByteArray>> = HashMap::new();
        loop {
            match self.reader.read_resolved_event_into(&mut buf) {
                Ok((Bound(Namespace(ns)), Event::Start(e))) => {
                    let tag_local_name = e.local_name();
                    let tag_name = str::from_utf8(tag_local_name.as_ref())?;
                    let hash = self.get_hash_from_child(tag_name)?;
                    if !hm.contains_key(tag_name) {
                        hm.insert(tag_name.to_string(), vec![]);
                    }
                    hm.get_mut(tag_name).unwrap().push(hash);
                }
                Ok((Bound(Namespace(ns)), Event::End(e))) => {
                    if e.local_name().as_ref() == self.tag_name.as_bytes() {
                        break;
                    } else {
                        return Err(anyhow!(
                            "something goes wrong. End tag not found. Found {}",
                            str::from_utf8(e.local_name().as_ref()).unwrap()
                        ));
                    }
                }
                Ok(e) => {
                    return Err(anyhow!("Text expected. {:?} found", e));
                }
                Err(e) => {
                    return Err(anyhow!(e).context("Error in hash_native_type getting the type"));
                }
            }
            buf.clear();
        }
        self.hash_hashed_childern(&hm)
    }

    fn try_hash(&mut self) -> anyhow::Result<ByteArray> {
        if self.schema_node.is_complex_type() {
            self.hash_complex_type()
        } else {
            self.hash_native_type(self.schema_node.native_type()?.unwrap().as_str())
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
            "token" => Self::String(res),
            _ => return Err(anyhow!("type {} unknowm", native_type)),
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
mod test {}

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
