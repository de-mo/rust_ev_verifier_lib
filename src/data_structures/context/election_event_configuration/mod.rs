mod election;
mod vote;

use super::super::{
    xml::{hashable::XMLFileHashable, xml_read_to_end_into_buffer, SchemaKind},
    DataStructureError, VerifierDataDecode,
};
use crate::{
    data_structures::{
        common_types::Signature,
        xml::{impl_iterator_for_tag_many_iter, TagManyIter, TagManyWithIterator},
    },
    direct_trust::{CertificateAuthority, VerifiySignatureTrait, VerifySignatureError},
};
pub use election::{Candidate, Election, ElectionInformation, List, WriteInCandidate};
use quick_xml::{
    de::from_str as xml_de_from_str,
    events::{BytesEnd, BytesStart, Event},
    Reader,
};
use rust_ev_crypto_primitives::{
    ByteArray, HashableMessage, RecursiveHashTrait, VerifyDomainTrait,
};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
use tracing::error;
pub use vote::{Answer, StandardQuestion, Vote};

const HEADER_TAG: &str = "header";
const SIGNATURE_TAG: &str = "signature";
const CONTEST_TAG: &str = "contest";
const AUTHORIZATIONS_TAG: &str = "authorizations";
const AUTHORIZATION_TAG: &str = "authorization";
const REGISTER_TAG: &str = "register";
const VOTER_TAG: &str = "voter";
const VOTE_INFORMATION_TAG: &str = "voteInformation";
const ELECTION_GROUP_BALLOT_TAG: &str = "electionGroupBallot";

#[derive(Clone, Debug)]
pub struct ElectionEventConfiguration {
    pub path: PathBuf,
    pub header: ConfigHeader,
    pub contest: Contest,
    pub authorizations: TagManyWithIterator<Authorization>,
    pub register: TagManyWithIterator<Voter>,
    pub signature: Signature,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigHeader {
    pub file_date: String,
    pub voter_total: usize,
}

#[derive(Clone, Debug)]
pub struct Contest {
    file_path: PathBuf,
    position_in_buffer: usize,
    pub votes: TagManyWithIterator<VoteInformation>,
    pub election_groups: TagManyWithIterator<ElectionGroupBallot>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Voter {
    pub voter_identification: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Authorization {
    pub authorization_identification: String,
    pub authorization_alias: String,
    pub authorization_object: Vec<AuthorizationObject>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationObject {
    pub domain_of_influence: DomainOfInfluence,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomainOfInfluence {
    pub domain_of_influence_identification: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VoteInformation {
    pub vote: Vote,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionGroupBallot {
    pub election_group_identification: String,
    pub domain_of_influence: String,
    pub election_group_position: usize,
    pub election_information: Vec<ElectionInformation>,
}

impl_iterator_for_tag_many_iter!(Authorization);
impl_iterator_for_tag_many_iter!(Voter);
impl_iterator_for_tag_many_iter!(VoteInformation);
impl_iterator_for_tag_many_iter!(ElectionGroupBallot);

impl VerifyDomainTrait<String> for ElectionEventConfiguration {}

impl VerifierDataDecode for ElectionEventConfiguration {
    fn stream_xml(p: &Path) -> Result<Self, DataStructureError> {
        let mut reader = Reader::from_file(p).map_err(|e| DataStructureError::ParseQuickXML {
            msg: format!("Error creating xml reader for file {}", p.to_str().unwrap()),
            source: e,
        })?;
        let reader_config_mut = reader.config_mut();
        reader_config_mut.trim_text(true);

        let mut signature_started = false;

        let mut signature: Option<Signature> = None;
        let mut config_header: Option<ConfigHeader> = None;
        let mut contest: Option<Contest> = None;
        let mut authorizations: Option<TagManyWithIterator<Authorization>> = None;
        let mut register: Option<TagManyWithIterator<Voter>> = None;

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
                    if e == BytesStart::new(SIGNATURE_TAG) {
                        signature_started = true;
                    }
                    if e == BytesStart::new(HEADER_TAG) {
                        config_header = Some(ConfigHeader::read_header_from_reader(&mut reader)?);
                    }
                    if e == BytesStart::new(CONTEST_TAG) {
                        contest = Some(Contest::new(p, reader.buffer_position() as usize));
                        let mut buffer = vec![];
                        xml_read_to_end_into_buffer(&mut reader, CONTEST_TAG, &mut buffer)
                            .map_err(|e| DataStructureError::ParseQuickXML {
                                msg: "Error reading contest bytes".to_string(),
                                source: e,
                            })?;
                    }
                    if e == BytesStart::new(AUTHORIZATIONS_TAG) {
                        authorizations = Some(TagManyWithIterator::<Authorization>::new(
                            p,
                            reader.buffer_position() as usize,
                            AUTHORIZATION_TAG,
                        ));
                        let mut buffer = vec![];
                        xml_read_to_end_into_buffer(&mut reader, AUTHORIZATIONS_TAG, &mut buffer)
                            .map_err(|e| DataStructureError::ParseQuickXML {
                            msg: "Error reading authorizations bytes".to_string(),
                            source: e,
                        })?;
                    }
                    if e == BytesStart::new(REGISTER_TAG) {
                        register = Some(TagManyWithIterator::<Voter>::new(
                            p,
                            reader.buffer_position() as usize,
                            VOTER_TAG,
                        ));
                        let mut buffer = vec![];
                        xml_read_to_end_into_buffer(&mut reader, REGISTER_TAG, &mut buffer)
                            .map_err(|e| DataStructureError::ParseQuickXML {
                                msg: "Error reading regitrer bytes".to_string(),
                                source: e,
                            })?;
                    }
                }
                Ok(Event::End(e)) => {
                    if e == BytesEnd::new(SIGNATURE_TAG) {
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
        Ok(Self {
            path: p.to_path_buf(),
            header: config_header.ok_or(DataStructureError::DataError(
                "Header not found".to_string(),
            ))?,
            contest: contest.ok_or(DataStructureError::DataError(
                "Context not found".to_string(),
            ))?,
            authorizations: authorizations.ok_or(DataStructureError::DataError(
                "Authorizations not found".to_string(),
            ))?,
            register: register.ok_or(DataStructureError::DataError(
                "Register not found".to_string(),
            ))?,
            signature: signature.unwrap(),
        })
    }
}

impl ConfigHeader {
    pub fn read_header_from_reader<R: BufRead>(
        reader: &mut Reader<R>,
    ) -> Result<Self, DataStructureError> {
        let mut header_bytes = vec![];
        xml_read_to_end_into_buffer(reader, HEADER_TAG, &mut header_bytes).map_err(|e| {
            DataStructureError::ParseQuickXML {
                msg: "Error reading header bytes".to_string(),
                source: e,
            }
        })?;
        xml_de_from_str(&String::from_utf8_lossy(&header_bytes)).map_err(|e| {
            DataStructureError::ParseQuickXMLDE {
                msg: "Error deserializing header".to_string(),
                source: e,
            }
        })
    }
}

impl Contest {
    pub fn new(path: &Path, position_in_buffer: usize) -> Self {
        Self {
            file_path: path.to_path_buf(),
            position_in_buffer,
            votes: TagManyWithIterator::<VoteInformation>::new(
                path,
                position_in_buffer,
                VOTE_INFORMATION_TAG,
            ),
            election_groups: TagManyWithIterator::<ElectionGroupBallot>::new(
                path,
                position_in_buffer,
                ELECTION_GROUP_BALLOT_TAG,
            ),
        }
    }
}

pub struct VoterIter {
    reader: Reader<BufReader<File>>,
}

impl Iterator for VoterIter {
    type Item = Voter;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        loop {
            match self.reader.read_event_into(&mut buf) {
                Err(_) => return None,
                Ok(Event::Eof) => return None,
                Ok(Event::Start(e)) => {
                    if e == BytesStart::new("voter") {
                        let mut buffer = vec![];
                        return match xml_read_to_end_into_buffer(
                            &mut self.reader,
                            "voter",
                            &mut buffer,
                        ) {
                            Ok(_) => {
                                match xml_de_from_str::<Voter>(&String::from_utf8_lossy(&buffer)) {
                                    Ok(v) => Some(v),
                                    Err(_) => None,
                                }
                            }
                            Err(_) => None,
                        };
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

impl VoteInformation {
    pub fn has_authorization(&self, auth: &Authorization) -> bool {
        auth.authorization_object.iter().any(|a| {
            self.vote.domain_of_influence
                == a.domain_of_influence.domain_of_influence_identification
        })
    }
}

impl ElectionGroupBallot {
    pub fn has_authorization(&self, auth: &Authorization) -> bool {
        auth.authorization_object.iter().any(|a| {
            self.domain_of_influence == a.domain_of_influence.domain_of_influence_identification
        })
    }
}

impl<'a> VerifiySignatureTrait<'a> for ElectionEventConfiguration {
    fn get_hashable(&self) -> Result<HashableMessage, Box<VerifySignatureError>> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Config, "signature");
        let hash = hashable
            .recursive_hash()
            .map_err(|e| VerifySignatureError::XMLError {
                msg: String::default(),
                source: e,
            })?;
        Ok(HashableMessage::Hashed(hash))
    }

    fn get_context_data(&self) -> Vec<HashableMessage> {
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

    #[test]
    fn test_voters() {
        let data = get_data_res().unwrap();
        let mut it = data.register.iter().unwrap();
        assert_eq!(it.count(), 43);
        it = data.register.iter().unwrap();
        assert_eq!(it.next().unwrap().voter_identification, "100001");
    }

    #[test]
    fn test_authorizations() {
        let data = get_data_res().unwrap();
        let mut it = data.authorizations.iter().unwrap();
        assert_eq!(it.count(), 4);
        it = data.authorizations.iter().unwrap();
        let auth = it.next().unwrap();
        assert_eq!(
            auth.authorization_identification,
            "516e2551-ee42-3401-9988-7dfebd0ac0c0"
        );
        assert_eq!(
            auth.authorization_object[0]
                .domain_of_influence
                .domain_of_influence_identification,
            "doid-ch1-mu"
        );
    }

    #[test]
    fn test_votes() {
        let data = get_data_res().unwrap();
        let mut it = data.contest.votes.iter().unwrap();
        assert_eq!(it.count(), 1);
        it = data.contest.votes.iter().unwrap();
        assert_eq!(it.next().unwrap().vote.vote_identification, "ch_test");
    }

    #[test]
    fn test_elections() {
        let data = get_data_res().unwrap();
        let mut it = data.contest.election_groups.iter().unwrap();
        assert_eq!(it.count(), 2);
        it = data.contest.election_groups.iter().unwrap();
        assert_eq!(it.next().unwrap().election_group_identification, "nrw_test");
        assert_eq!(
            it.next().unwrap().election_group_identification,
            "majorz_test"
        );
    }
}
