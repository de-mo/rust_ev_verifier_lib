// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

//! Module to manage the schemas used for the verifier
pub mod hashable;
mod schema;
mod schema_tree;
mod shared_types;

use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
    Error as QuickXmlError, Writer,
};
use roxmltree::Error as RoXmlTreeError;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    ByteArrayError, HashError, HashableMessage,
};
pub use schema::SchemaKind;
pub(super) use shared_types::{impl_iterator_for_tag_many_iter, TagManyIter, TagManyWithIterator};
use std::{io::BufRead, num::ParseIntError, str::Utf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    RoXML { msg: String, source: RoXmlTreeError },
    #[error("No schema found for namespace {0}")]
    NoSchemaFound(String),
    #[error("targeetNamespace missing")]
    NoTargetNamespace,
    #[error("The name of the {0} is not defined in the list of namespaces")]
    NotInNamespaces(String),
    #[error("Attribute namespace is missing")]
    NoNamespaceAttribute,
    #[error("Namespace {0} is missing in import")]
    NoNamespaceImport(String),
    #[error("Namespace name {0} not found")]
    NoNamespaceName(String),
    #[error("The node is not an element")]
    NotElement,
    #[error("The node is not a sequence")]
    NotSequence,
    #[error("The node is not a choice")]
    NotChoice,
    #[error("The node is not a complex type")]
    NotComplexType,
    #[error("{0}")]
    NotCorrectNodeType(String),
    #[error("The attribute is missing {0}")]
    AttributeMissing(String),
    #[error("First element not found")]
    FirstElementNotFound,
    #[error("No type found for node {0} with attributes: {1}")]
    TypeNotFound(String, String),
    #[error("{0}")]
    PrefixError(String),
}

#[derive(Error, Debug)]
pub enum XMLError {
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Error in schema {msg} -> caused by: {source}")]
    Schema { msg: String, source: SchemaError },
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    QuickXML { msg: String, source: QuickXmlError },
    #[error("Type unknown {0}")]
    TypeUnknown(String),
    #[error("Not boolean representation {0}")]
    NotBoolean(String),
    #[error("Not integer representation {0}: {1}")]
    NotInt(String, ParseIntError),
    #[error("Not byte array representation {0}: {1}")]
    NotByteArray(String, ByteArrayError),
    #[error("Error hashing {msg} -> caused by: {source}")]
    HashError { msg: String, source: HashError },
    #[error("Text expected. {0} found")]
    TextExpected(String),
    #[error("Tag {tag} not found in node {node}")]
    TagNotFound { tag: String, node: String },
    #[error(transparent)]
    Uft8(Utf8Error),
}

// reads from a start tag all the way to the corresponding end tag,
// returns the bytes of the whole tag
pub fn xml_read_to_end_into_buffer<R: BufRead>(
    reader: &mut Reader<R>,
    tag_name: &str,
    buffer: &mut Vec<u8>,
) -> Result<(), QuickXmlError> {
    let start_tag = BytesStart::new(tag_name);
    let mut depth = 0;
    let mut temp_buffer = vec![];
    let mut w = Writer::new(buffer);
    let tag_name = start_tag.name();
    w.write_event(Event::Start(start_tag.clone()))?;
    loop {
        temp_buffer.clear();
        let event = reader.read_event_into(&mut temp_buffer)?;
        w.write_event(event.clone())?;

        match event {
            Event::Start(e) if e.name() == tag_name => depth += 1,
            Event::End(e) if e.name() == tag_name => {
                if depth == 0 {
                    return Ok(());
                }
                depth -= 1;
            }
            Event::Eof => {
                panic!("oh no")
            }
            _ => {}
        }
    }
}

/// Return the [HashableMessage] no value with the argument `t`
pub fn hashable_no_value(t: &str) -> HashableMessage {
    let s: String = format!("no {} value", t).to_string();
    HashableMessage::from(s)
}

/// Return the [HashableMessage] from an option using [hashable_no_value] for `None`
pub fn hashable_from_option<'a>(
    opt: Option<HashableMessage<'a>>,
    t: &'a str,
) -> HashableMessage<'a> {
    match opt {
        Some(m) => m,
        None => hashable_no_value(t),
    }
}
