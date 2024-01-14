use super::{
    schema::{Schema, SchemaKind},
    schema_iterator::ElementNode,
};
use anyhow::{anyhow, Context, Result};
use quick_xml::{
    events::Event,
    name::{Namespace, ResolveResult::*},
    reader::NsReader,
};
use rust_ev_crypto_primitives::{
    byte_array::{ByteArray, Decode},
    hashing::{HashTrait, HashableMessage},
};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

pub struct XMLFileHashable<'a> {
    file: PathBuf,
    schema: &'a Schema<'a>,
}

struct NodeHashable<'a> {
    reader: &'a NsReader<BufReader<File>>,
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

    fn try_hash(&self) -> std::prelude::v1::Result<ByteArray, Self::Error> {
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
            match reader.read_resolved_event_into(&mut buf).unwrap() {
                (Bound(Namespace(ns)), Event::Start(e)) => {
                    if e.local_name().as_ref() == schema_node.name().as_bytes() {
                        return NodeHashable::new(&schema_node, &reader).try_hash();
                    }
                }
                (_, Event::Eof) => panic!("tag {} not found", schema_node.name()),
                _ => (),
            }
            buf.clear();
        }
    }
}

impl<'a> NodeHashable<'a> {
    fn new(schema_node: &'a ElementNode<'a>, reader: &'a NsReader<BufReader<File>>) -> Self {
        Self {
            reader,
            schema_node,
        }
    }

    fn hash_native_type(value: &str, native_type: &str) -> Result<ByteArray> {
        Ok(NativeTypeConverter::new(value, native_type)?.to_hashable()?.hash())
    }

    fn try_hash(&mut self) -> Result<ByteArray> {
        if self.schema_node.is_complex_type() {
            todo!()
        } else {
            todo!()
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
    pub fn new(value: &str, native_type: &str) -> Result<Self> {
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

    fn to_hashable<'a>(&'a self) -> Result<HashableMessage<'a>> {
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
