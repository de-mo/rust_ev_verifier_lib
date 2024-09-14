//! Module to manage the schemas used for the verifier
pub mod hashable;
mod schema;
mod schema_tree;

use anyhow::anyhow;
use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
    Error as QuickXmlError, Writer,
};
use rust_ev_crypto_primitives::HashableMessage;
pub use schema::SchemaKind;
use std::io::BufRead;
use thiserror::Error;

/*
// Enum representing the direct trust errors
#[derive(Error, Debug)]
pub enum ParseXMLError {
    #[error("IO error for {path} -> caused by: {source}")]
    IO {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    ParseQuickXML { msg: String, source: QuickXmlError },

    #[error("Path is not a file {0}")]
    PathNotFile(PathBuf),
    #[error("Error parsing xml {msg} -> caused by: {source}")]
    ParseRoXML { msg: String, source: RoXmlTreeError },
    #[error("Any error {msg} -> caused by: {source}")]
    Any { msg: String, source: anyhow::Error },
    #[error("Mock error: {0}")]
    Mock(String),
    #[error("Error reading data structure {path} -> caudes by: {source}")]
    ReadDataStructure {
        path: PathBuf,
        source: DataStructureError,
    },
}*/

// reads from a start tag all the way to the corresponding end tag,
// returns the bytes of the whole tag
pub fn xml_read_to_end_into_buffer<R: BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
    junk_buf: &mut Vec<u8>,
) -> Result<Vec<u8>, QuickXmlError> {
    let mut depth = 0;
    let mut output_buf: Vec<u8> = Vec::new();
    let mut w = Writer::new(&mut output_buf);
    let tag_name = start_tag.name();
    w.write_event(Event::Start(start_tag.clone()))?;
    loop {
        junk_buf.clear();
        let event = reader.read_event_into(junk_buf)?;
        w.write_event(event.clone())?;

        match event {
            Event::Start(e) if e.name() == tag_name => depth += 1,
            Event::End(e) if e.name() == tag_name => {
                if depth == 0 {
                    return Ok(output_buf);
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

#[allow(dead_code)]
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
