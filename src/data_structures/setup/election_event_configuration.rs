use super::super::{
    xml::{hashable::XMLFileHashable, xml_read_to_end_into_buffer, SchemaKind},
    VerifierDataDecode,
};
use crate::{
    data_structures::common_types::Signature,
    direct_trust::{CertificateAuthority, VerifiySignatureTrait},
};
use anyhow::anyhow;
use quick_xml::{
    de::from_str as xml_de_from_str,
    events::{BytesEnd, BytesStart, Event},
    Reader,
};
use rust_ev_crypto_primitives::{ByteArray, HashableMessage, RecursiveHashTrait};
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

impl VerifierDataDecode for ElectionEventConfiguration {
    fn from_xml_file(p: &Path) -> anyhow::Result<Self> {
        let mut reader = Reader::from_file(p).map_err(|e| {
            anyhow!(e).context(format!(
                "Error creating xml reader for file {}",
                p.to_str().unwrap()
            ))
        })?;
        reader.trim_text(true);

        let header_tag = "header";
        let signature_tag = "signature";
        let mut signature_started = false;

        let mut signature: Option<Signature> = None;
        let mut config_header: Option<ConfigHeader> = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    return Err(anyhow!(e)
                        .context(format!("Error at position {}", reader.buffer_position())))
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
                            anyhow!(e).context("Error reading header bytes".to_string())
                        })?;
                        config_header = Some(
                            xml_de_from_str(&String::from_utf8_lossy(&header_bytes)).map_err(
                                |e| anyhow!(e).context("Error deserializing header".to_string()),
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
            return Err(anyhow!("Header not found"));
        }
        if signature.is_none() {
            return Err(anyhow!("Signature not found"));
        }
        Ok(Self {
            path: p.to_path_buf(),
            header: config_header.unwrap(),
            signature: signature.unwrap(),
        })
    }
}

impl<'a> VerifiySignatureTrait<'a> for ElectionEventConfiguration {
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Config, "signature");
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
        self.signature.get_signature()
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
        let config_res = ElectionEventConfiguration::from_xml_file(&path);
        assert!(config_res.is_ok());
        let config = config_res.unwrap();
        assert_eq!(config.header.voter_total, 43);
        assert_eq!(config.signature.signature_contents, "uIW6bxAAXcwkntKGQ4c1otj7ZwXvqz3t4yP5QDHLHblKGP/b0/eyI0FPwD0397aOdwi7IEU5h7+yhGO4OwhXsAcqqSmwPqK855BA4Mc6gSURVFc+JMwf0iDUFgw4dxTKRTTiuI9uS4P5fyT5kqQfhwbBMjh2IOYzYadaNqQ+jGCv/kmhSlw59yG03OUqObDwcvhjaTS3CqhBk/DhEHfnfcYnCri+KEXArV4PCCS8cGO8cqzi+oQzX66o+NSGmihbj0T/FZTRbvWE+VoXa0XTNSdwOVnjVkv9YuQRPBw0jy/CEuO+mwigVZckX5FjINENcRd9KTEvNf1Erxlp1o0l1p2Dqo2WZsx40O31SQmyvHYRu6xAVTDndtT/FCd3yqsJo8u3wFM6jd8oI20/kGInk8B/+bAjYDG/syAj742dLpstrkG77LEa01PVcGlhny4k8y+76gsMFrCEAEIqKtUODX8b15QARmmACQVuZ57VBGllBAWkWpJa44GAMTjCEiPe");
    }
}
