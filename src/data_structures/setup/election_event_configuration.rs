use super::super::{
    error::{DeserializeError, DeserializeErrorType},
    xml_read_to_end_into_buffer, VerifierDataDecode,
};
use crate::{
    crypto_primitives::{
        byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
        signature::VerifiySignatureTrait,
    },
    error::{create_result_with_error, create_verifier_error, VerifierError},
};
use quick_xml::{
    de::from_str as xml_de_from_str,
    events::{BytesStart, Event},
    Reader,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ElectionEventConfiguration {
    path: PathBuf,
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
    fn from_xml_file(p: &std::path::Path) -> Result<Self, DeserializeError> {
        let mut reader = Reader::from_file(p).map_err(|e| {
            create_verifier_error!(
                DeserializeErrorType::XMLError,
                format!("Error creating xml reader for file {}", p.to_str().unwrap()),
                e
            )
        })?;
        reader.trim_text(true);

        let start_header = BytesStart::new("header");

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    return create_result_with_error!(
                        DeserializeErrorType::XMLError,
                        format!("Error at position {}", reader.buffer_position()),
                        e
                    )
                }
                Ok(Event::Eof) => {
                    return create_result_with_error!(
                        DeserializeErrorType::XMLError,
                        format!("cannot find tag voterTotal")
                    )
                }
                Ok(Event::Start(e)) => {
                    if e == start_header {
                        let header_bytes =
                            xml_read_to_end_into_buffer(&mut reader, &start_header, &mut buf)
                                .map_err(|e| {
                                    create_verifier_error!(
                                        DeserializeErrorType::XMLError,
                                        "Error reading header bytes",
                                        e
                                    )
                                })?;
                        let config_header: ConfigHeader = xml_de_from_str(
                            &String::from_utf8_lossy(&header_bytes),
                        )
                        .map_err(|e| {
                            create_verifier_error!(
                                DeserializeErrorType::XMLError,
                                "Error deserializing header",
                                e
                            )
                        })?;
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

impl<'a> From<&'a ElectionEventConfiguration> for HashableMessage<'a> {
    fn from(_: &ElectionEventConfiguration) -> Self {
        todo!()
    }
}

impl<'a> VerifiySignatureTrait<'a> for ElectionEventConfiguration {
    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("configuration")]
    }

    fn get_certificate_authority(&self) -> CertificateAuthority {
        CertificateAuthority::Canton
    }

    fn get_signature(&self) -> ByteArray {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_data_set() {
        let path = Path::new(".")
            .join("datasets")
            .join("dataset-setup1")
            .join("setup")
            .join("configuration-anonymized.xml");
        let config = ElectionEventConfiguration::from_xml_file(&path);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().header.voter_total, 76);
    }
}
