use super::super::{hashable_no_value, xml_read_to_end_into_buffer, VerifierDataDecode};
use anyhow::anyhow;
use crypto_primitives::{
    byte_array::ByteArray, direct_trust::CertificateAuthority, hashing::HashableMessage,
    signature::VerifiySignatureTrait,
};
use quick_xml::{
    de::from_str as xml_de_from_str,
    events::{BytesStart, Event},
    Reader,
};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub(crate) struct ElectionEventConfiguration {
    //path: PathBuf,
    pub(crate) header: ConfigHeader,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigHeader {
    pub(crate) file_date: String,
    pub(crate) voter_total: usize,
    pub(crate) partial_delivery: Option<PartialDelivery>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PartialDelivery {
    pub(crate) voter_from: usize,
    pub(crate) voter_to: usize,
}

impl VerifierDataDecode for ElectionEventConfiguration {
    fn from_xml_file(p: &std::path::Path) -> anyhow::Result<Self> {
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
                                    anyhow!(e).context(format!("Error reading header bytes"))
                                })?;
                        let config_header: ConfigHeader = xml_de_from_str(
                            &String::from_utf8_lossy(&header_bytes),
                        )
                        .map_err(|e| anyhow!(e).context(format!("Error deserializing header")))?;
                        return Ok(Self {
                            //path: p.to_path_buf(),
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
    fn from(config: &'a ElectionEventConfiguration) -> Self {
        let mut elts = vec![];
        elts.push(HashableMessage::from(&config.header));
        Self::from(elts)
    }
}

impl<'a> From<&'a ConfigHeader> for HashableMessage<'a> {
    fn from(value: &'a ConfigHeader) -> Self {
        let mut elts = vec![];
        elts.push(Self::from(&value.file_date));
        elts.push(Self::from(&value.voter_total));
        elts.push(match &value.partial_delivery {
            Some(v) => Self::from(v),
            None => hashable_no_value("partialDelivery"),
        });
        Self::from(elts)
    }
}

impl<'a> From<&'a PartialDelivery> for HashableMessage<'a> {
    fn from(value: &'a PartialDelivery) -> Self {
        Self::from(vec![
            Self::from(&value.voter_from),
            Self::from(&value.voter_to),
        ])
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
    use crate::constants::test::dataset_tally_path;

    #[test]
    fn read_data_set() {
        let path = dataset_tally_path()
            .join("setup")
            .join("configuration-anonymized.xml");
        let config = ElectionEventConfiguration::from_xml_file(&path);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().header.voter_total, 43);
    }
}
