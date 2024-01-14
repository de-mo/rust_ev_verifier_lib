//! Module to manage the schemas used for the verifier
mod schema;
mod schema_iterator;
pub mod hashable;

use anyhow::anyhow;
use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
    Writer,
};
use rust_ev_crypto_primitives::hashing::HashableMessage;
use std::io::BufRead;

pub use schema::SchemaKind;


// reads from a start tag all the way to the corresponding end tag,
// returns the bytes of the whole tag
pub fn xml_read_to_end_into_buffer<R: BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
    junk_buf: &mut Vec<u8>,
) -> anyhow::Result<Vec<u8>> {
    let mut depth = 0;
    let mut output_buf: Vec<u8> = Vec::new();
    let mut w = Writer::new(&mut output_buf);
    let tag_name = start_tag.name();
    w.write_event(Event::Start(start_tag.clone()))
        .map_err(|e| {
            anyhow!(e).context(format!("Error writing event {:?} in writer", start_tag))
        })?;
    loop {
        junk_buf.clear();
        let event = reader
            .read_event_into(junk_buf)
            .map_err(|e| anyhow!(e).context("format!(Error reading event"))?;
        w.write_event(&event).map_err(|e| {
            anyhow!(e).context(format!("Error writing event {:?} in writer", event))
        })?;

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
    HashableMessage::from(format!("no {} value", t))
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