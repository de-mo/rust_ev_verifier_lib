use super::super::{
    xml::{hashable::XMLFileHashable, xml_read_to_end_into_buffer, SchemaKind},
    DataStructureError, VerifierDataDecode,
};
use crate::{
    data_structures::common_types::Signature,
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};
use quick_xml::{
    de::from_str as xml_de_from_str,
    events::{BytesEnd, BytesStart, Event},
    Reader,
};
use rust_ev_crypto_primitives::{
    ByteArray, HashableMessage, RecursiveHashTrait, VerifyDomainTrait,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ElectionEventConfiguration {
    pub path: PathBuf,
    pub header: ConfigHeader,
    pub signature: Signature,
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

impl VerifyDomainTrait<anyhow::Error> for ElectionEventConfiguration {}

impl VerifierDataDecode for ElectionEventConfiguration {
    fn stream_xml(p: &Path) -> Result<Self, DataStructureError> {
        let mut reader = Reader::from_file(p).map_err(|e| DataStructureError::ParseQuickXML {
            msg: format!("Error creating xml reader for file {}", p.to_str().unwrap()),
            source: e,
        })?;
        let reader_config_mut = reader.config_mut();
        reader_config_mut.trim_text(true);

        let header_tag = "header";
        let signature_tag = "signature";
        let mut signature_started = false;

        let mut signature: Option<Signature> = None;
        let mut config_header: Option<ConfigHeader> = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    return Err(DataStructureError::ParseQuickXML {
                        msg: format!("Error at position {}", reader.buffer_position()),
                        source: e,
                    })
                }
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    if e == BytesStart::new(signature_tag) {
                        signature_started = true;
                    }
                    if e == BytesStart::new(header_tag) {
                        let header_bytes = xml_read_to_end_into_buffer(
                            &mut reader,
                            &BytesStart::new(header_tag),
                            &mut buf,
                        )
                        .map_err(|e| {
                            DataStructureError::ParseQuickXML {
                                msg: "Error reading header bytes".to_string(),
                                source: e,
                            }
                        })?;
                        config_header = Some(
                            xml_de_from_str(&String::from_utf8_lossy(&header_bytes)).map_err(
                                |e| DataStructureError::ParseQuickXMLDE {
                                    msg: "Error deserializing header".to_string(),
                                    source: e,
                                },
                            )?,
                        );
                    }
                }
                Ok(Event::End(e)) => {
                    if e == BytesEnd::new(signature_tag) {
                        signature_started = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    if signature_started {
                        signature = Some(Signature {
                            signature_contents: e.unescape().unwrap().into_owned(),
                        })
                    }
                }
                // There are several other `Event`s we do not consider here
                _ => (),
            }
            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }
        if config_header.is_none() {
            return Err(DataStructureError::DataError(
                "Header not found".to_string(),
            ));
        }
        if signature.is_none() {
            return Err(DataStructureError::DataError(
                "Signature not found".to_string(),
            ));
        }
        Ok(Self {
            path: p.to_path_buf(),
            header: config_header.unwrap(),
            signature: signature.unwrap(),
        })
    }
}

impl<'a> VerifiySignatureTrait<'a> for ElectionEventConfiguration {
    fn get_hashable(&'a self) -> Result<HashableMessage<'a>, VerifySignatureError> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Config, "signature");
        let hash = hashable
            .recursive_hash()
            .map_err(|e| VerifySignatureError::XMLError {
                msg: String::default(),
                source: e,
            })?;
        Ok(HashableMessage::Hashed(hash))
    }

    fn get_context_data(&self) -> Vec<HashableMessage<'a>> {
        vec![HashableMessage::from("configuration")]
    }

    fn get_certificate_authority(&self) -> Option<CertificateAuthority> {
        Some(CertificateAuthority::Canton)
    }

    fn get_signature(&self) -> ByteArray {
        self.signature.get_signature()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::test::test_datasets_context_path;

    fn get_data_res() -> Result<ElectionEventConfiguration, DataStructureError> {
        ElectionEventConfiguration::stream_xml(
            &test_datasets_context_path().join("configuration-anonymized.xml"),
        )
    }

    #[test]
    fn read_data_set() {
        let data_res = get_data_res();
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        assert_eq!(data.header.voter_total, 43);
        assert_eq!(data.signature.signature_contents, "ctgAK+cygZ89QJ/4XGccw4fl4Yc1MCdsywfJuR5AIs/+SnozxB8USh7UJvl64fxuZ6ks+86tRGazABP+Az/0hEmSxihadlYpGe5b2goqo/TSQzC+Z683sHV1O4B8RGjFYt93xIVpsvs4mYiyktz7ma8IanZk0nNhihhgF0Da07Tv4PmhUqAuzd7IQEYTaTz7RXebOHkH4pG4fA2HbSHeUMlBw0Ni51zx5LOO0riX/bHf4ffnmaqibbOdt88VZegQoNp1gy/R29L6mNrSi01WQnDZ3xxzeFJCG1eSb0MoLoDwNSizC63pqKmQbjhsQbqxDpmkhvSqW5EnvY4VH4rYIaONyjZeivJwUwnLEPbE9k/PnZTAlESCFFR3bHnawEKsRCtwynH0u6IRuTW2iMuupl+UE3tfx8WOsqbWBWNCL9/0WSrvdiJLTcScRmU3ZqW+1La0FG/BhZiI0egBA4KIOYAb9McWlIE7QS8hWJjpQP5xYa+s4SHP63YNr0LvQ1dh");
    }

    #[test]
    fn verify_signature() {}
}
