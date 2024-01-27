use super::super::{
    xml::{hashable::XMLFileHashable, xml_read_to_end_into_buffer, SchemaKind},
    VerifierDataDecode,
};
use crate::direct_trust::{CertificateAuthority, VerifiySignatureTrait};
use anyhow::anyhow;
use quick_xml::{
    de::from_str as xml_de_from_str,
    events::{BytesStart, Event},
    Reader,
};
use rust_ev_crypto_primitives::{ByteArray, HashableMessage, RecursiveHashTrait};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ElectionEventConfiguration {
    pub path: PathBuf,
    pub header: ConfigHeader,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigHeader {
    pub file_date: String,
    pub voter_total: usize,
    pub partial_delivery: Option<PartialDelivery>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialDelivery {
    pub voter_from: usize,
    pub voter_to: usize,
}

impl VerifierDataDecode for ElectionEventConfiguration {
    fn from_xml_file(p: &Path) -> anyhow::Result<Self> {
        let mut reader = Reader::from_file(p).map_err(|e| {
            anyhow!(e).context(format!(
                "Error creating xml reader for file {}",
                p.to_str().unwrap()
            ))
        })?;
        reader.trim_text(true);

        let start_header = BytesStart::new("header");

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    return Err(anyhow!(e)
                        .context(format!("Error at position {}", reader.buffer_position())))
                }
                Ok(Event::Eof) => return Err(anyhow!(format!("cannot find tag voterTotal"))),
                Ok(Event::Start(e)) => {
                    if e == start_header {
                        let header_bytes =
                            xml_read_to_end_into_buffer(&mut reader, &start_header, &mut buf)
                                .map_err(|e| {
                                    anyhow!(e).context("Error reading header bytes".to_string())
                                })?;
                        let config_header: ConfigHeader =
                            xml_de_from_str(&String::from_utf8_lossy(&header_bytes)).map_err(
                                |e| anyhow!(e).context("Error deserializing header".to_string()),
                            )?;
                        return Ok(Self {
                            path: p.to_path_buf(),
                            header: config_header,
                        });
                    }
                }
                // There are several other `Event`s we do not consider here
                _ => (),
            }
            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }
    }
}

impl<'a> VerifiySignatureTrait<'a> for ElectionEventConfiguration {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Config);
        let hash = hashable.try_hash()?;
        Ok(HashableMessage::Hashed(hash))
    }

    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("configuration")]
    }

    fn get_certificate_authority(&self) -> anyhow::Result<String> {
        Ok(String::from(CertificateAuthority::Canton))
    }

    fn get_signature(&self) -> ByteArray {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_dataset_tally_path;

    #[test]
    fn read_data_set() {
        let path = test_dataset_tally_path()
            .join("setup")
            .join("configuration-anonymized.xml");
        let config = ElectionEventConfiguration::from_xml_file(&path);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().header.voter_total, 43);
    }
}
