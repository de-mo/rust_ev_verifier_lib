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
        VerifierDataToTypeTrait, VerifierDataType,
    },
    direct_trust::{CertificateAuthority, VerifiySignatureTrait},
};
use chrono::NaiveDate;
pub use election::{Candidate, Election, ElectionInformation, List, WriteInCandidate};
use quick_xml::{
    de::from_str as xml_de_from_str,
    events::{BytesEnd, BytesStart, Event},
    Reader,
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
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
const CONTEST_IDENTIFICATION_TAG: &str = "contestIdentification";
const CONTEST_DATE_TAG: &str = "contestDate";

#[derive(Clone, Debug)]
pub struct ElectionEventConfiguration {
    pub path: PathBuf,
    pub header: ConfigHeader,
    pub contest: Contest,
    pub authorizations: TagManyWithIterator<Authorization>,
    pub register: TagManyWithIterator<Voter>,
    pub signature: Signature,
}

impl VerifierDataToTypeTrait for ElectionEventConfiguration {
    fn data_type() -> crate::data_structures::VerifierDataType {
        VerifierDataType::Context(super::VerifierContextDataType::ElectionEventConfiguration)
    }
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
    pub authorization: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Authorization {
    pub authorization_identification: String,
    pub authorization_alias: String,
    pub authorization_test: bool,
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

impl ElectionEventConfiguration {
    fn list_of_test_ballot_boxes(&self) -> Result<Vec<String>, DataStructureError> {
        Ok(self
            .authorizations
            .iter()?
            .filter(|auth| auth.authorization_test)
            .map(|auth| auth.authorization_identification)
            .collect())
    }
}

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
                            AUTHORIZATIONS_TAG,
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
                            REGISTER_TAG,
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
                CONTEST_TAG,
            ),
            election_groups: TagManyWithIterator::<ElectionGroupBallot>::new(
                path,
                position_in_buffer,
                ELECTION_GROUP_BALLOT_TAG,
                CONTEST_TAG,
            ),
        }
    }

    fn reader(&self) -> Result<Reader<BufReader<File>>, DataStructureError> {
        let mut reader =
            Reader::from_file(&self.file_path).map_err(|e| DataStructureError::ParseQuickXML {
                msg: format!(
                    "Error creating xml reader for file {}",
                    self.file_path.to_str().unwrap()
                ),
                source: e,
            })?;
        reader.stream().consume(self.position_in_buffer);
        Ok(reader)
    }

    fn read_tag(&self, tag_name: &str) -> Result<String, DataStructureError> {
        let mut buf = Vec::new();
        let mut reader = self.reader()?;
        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    return Err(DataStructureError::ParseQuickXML {
                        msg: format!("Error reading the tag {}", tag_name),
                        source: e,
                    })
                }
                Ok(Event::Eof) => {
                    return Err(DataStructureError::DataError(format!(
                        "Tag {} not found",
                        tag_name
                    )))
                }
                Ok(Event::Start(e)) => {
                    if e == BytesStart::new(tag_name) {
                        let mut buffer = vec![];
                        match xml_read_to_end_into_buffer(&mut reader, tag_name, &mut buffer) {
                            Ok(_) => {
                                return String::from_utf8(buffer)
                                    .map(|s| {
                                        s.strip_prefix(&format!("<{}>", tag_name))
                                            .unwrap()
                                            .strip_suffix(&format!("</{}>", tag_name))
                                            .unwrap()
                                            .to_string()
                                    })
                                    .map_err(|e| {
                                        DataStructureError::DataError(format!(
                                            "Error reading the content of the tag {}: {}",
                                            &tag_name, e
                                        ))
                                    });
                            }
                            Err(e) => {
                                return Err(DataStructureError::ParseQuickXML {
                                    msg: format!("Error finding the end of the tag {}", &tag_name),
                                    source: e,
                                })
                            }
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

    pub fn contest_identification(&self) -> Result<String, DataStructureError> {
        let res = self.read_tag(CONTEST_IDENTIFICATION_TAG)?;
        Ok(res)
    }

    pub fn contest_date(&self) -> Result<NaiveDate, DataStructureError> {
        let date_str = self.read_tag(CONTEST_DATE_TAG)?;
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
            DataStructureError::DataError(format!(
                "Contest date {} cannot be converted: {}",
                &date_str, e
            ))
        })
    }

    pub fn number_of_elections(&self) -> Result<usize, DataStructureError> {
        Ok(self
            .election_groups
            .iter()?
            .map(|el| el.number_of_elections())
            .sum())
    }

    pub fn number_of_votes_and_ballots(&self) -> Result<(usize, usize), DataStructureError> {
        let mut number_of_votes = 0;
        let mut number_of_ballots = 0;
        self.votes.iter()?.for_each(|vote| {
            number_of_votes += 1;
            number_of_ballots += vote.vote.ballot.len();
        });
        Ok((number_of_votes, number_of_ballots))
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

    pub fn number_of_elections(&self) -> usize {
        self.election_information.len()
    }
}

impl Voter {
    pub fn is_test_voter(
        &self,
        test_authorization_ids: &[String],
    ) -> Result<bool, DataStructureError> {
        Ok(test_authorization_ids.contains(&self.authorization))
    }
}

impl VerifiySignatureTrait<'_> for ElectionEventConfiguration {
    fn get_hashable(&self) -> Result<HashableMessage, DataStructureError> {
        let hashable = XMLFileHashable::new(&self.path, &SchemaKind::Config, "signature");
        let hash = hashable
            .recursive_hash()
            .map_err(DataStructureError::from)?;
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ManuelVerificationInputFromConfiguration {
    pub contest_identification: String,
    pub contest_date: NaiveDate,
    pub number_of_votes: usize,
    pub number_of_ballots: usize,
    pub number_of_elections: usize,
    pub number_of_productive_voters: usize,
    pub number_of_test_voters: usize,
    pub number_of_productive_ballot_boxes: usize,
    pub number_of_test_ballot_boxes: usize,
}

impl TryFrom<&ElectionEventConfiguration> for ManuelVerificationInputFromConfiguration {
    type Error = DataStructureError;
    fn try_from(value: &ElectionEventConfiguration) -> Result<Self, Self::Error> {
        let (number_of_votes, number_of_ballots) = value.contest.number_of_votes_and_ballots()?;
        let number_of_productive_ballot_boxes = value
            .authorizations
            .iter()?
            .filter(|auth| !auth.authorization_test)
            .count();

        let mut number_of_productive_voters = 0;
        let mut number_of_test_voters = 0;
        let test_authorization_ids = value.list_of_test_ballot_boxes()?;
        value
            .register
            .iter()?
            .map(|voter| match voter.is_test_voter(&test_authorization_ids) {
                Ok(b) => {
                    match b {
                        true => number_of_test_voters += 1,
                        false => number_of_productive_voters += 1,
                    };
                    Ok(())
                }
                Err(e) => Err(e),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            contest_identification: value.contest.contest_identification()?,
            contest_date: value.contest.contest_date()?,
            number_of_votes,
            number_of_ballots,
            number_of_elections: value.contest.number_of_elections()?,
            number_of_productive_voters,
            number_of_test_voters,
            number_of_productive_ballot_boxes,
            number_of_test_ballot_boxes: value.authorizations.iter()?.count()
                - number_of_productive_ballot_boxes,
        })
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
        assert_eq!(data.signature.signature_contents, "bWRtb1c9AJnnY5KDYI4wkmzhDKd5qGwhgzixyJ0c6dHpHANgpWUEmqHl8ky30gUEF4j6/vEsG17zP2e+KOM7fztPqCbZam7rNgQeK+ytK95WVaqyYXzLhoBdKseGXWZbwEK77PjEXDpc7Yoqre34vOj6TKLM697RFVOJS7eZy4yNyWGMqNrKQwkvCMse/iwq5aK0FYea3/BUa5UosMWaGhwub4ZUdrqODLnrYRrhGG0o565azm5TSv9Vp9buM1kwRPjbwUuiccY8e0WCWUFLaTnuiUrsZczrY3MiQ1JtPDrcXP3T461TExsSKMcSstnExNYcdWahwiHyB4USInkkeW5peUoOojO4TRvzlSu9ca1kIIVocOFxUCcJBS5A4HpkpD/QtlQKK2yBFfAcRtB1Y8OS06JclV8up2aeih5kKecWRSMhmLdgNnGrrX0OFRSN+IclHR+DlJKEZEfj1qbDF1NxjeoCxM/XHEYs1LJM9q0DuTA0suE1jIy+7BaAPha1");
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

    #[test]
    fn test_manual_data() {
        let data = get_data_res().unwrap();
        let manual_data = ManuelVerificationInputFromConfiguration::try_from(&data).unwrap();
        assert_eq!(
            manual_data,
            ManuelVerificationInputFromConfiguration {
                contest_identification: "Post_E2E_DEV".to_string(),
                contest_date: NaiveDate::from_ymd_opt(2027, 11, 25).unwrap(),
                number_of_votes: 1,
                number_of_ballots: 2,
                number_of_elections: 2,
                number_of_productive_voters: 0,
                number_of_test_voters: 43,
                number_of_productive_ballot_boxes: 0,
                number_of_test_ballot_boxes: 4
            }
        )
    }
}
