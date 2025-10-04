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

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    iter::once,
};

use roxmltree::Node;

use crate::data_structures::{
    context::election_event_configuration::{Answer, AnswerInfo, Ballot, StandardQuestion},
    tally::ech_0222::{ECH0222CalculatedErrorImpl, ECH0222Difference, ECh0222differencesTrait},
    xml::ElementChildren,
};

#[derive(Debug, Clone)]
pub struct VoteRawData {
    pub vote_identification: String,
    pub ballot_raw_data: Vec<BallotRawData>,
}

#[derive(Debug, Clone, Hash)]
pub struct BallotRawData {
    pub electronic_ballot_identification: String,
    pub ballot_casted: BallotCasted,
}

#[derive(Debug, Clone)]
pub struct BallotCasted {
    pub ballot_casted_number: Option<usize>,
    pub question_raw_data: Vec<QuestionRawData>,
}

#[derive(Debug, Clone, Hash)]
pub struct QuestionRawData {
    pub question_identification: String,
    pub casted: Option<Casted>,
}

#[derive(Debug, Clone, Hash)]
pub struct Casted {
    pub casted_vote: usize,
    pub answer_option_identification: Option<AnswerOptionIdentification>,
}

#[derive(Debug, Clone)]
pub struct AnswerOptionIdentification {
    pub answer_option_identification: String,
    pub answer_sequence_number: usize,
    pub answer_text_information: Vec<AnswerTextInformation>,
}

#[derive(Debug, Clone, Hash)]
pub struct AnswerTextInformation {
    pub language: String,
    pub answer_text: String,
}

impl VoteRawData {
    pub(super) fn new(vote_identification: &str) -> Self {
        Self {
            vote_identification: vote_identification.to_string(),
            ballot_raw_data: vec![],
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ballot_raw_data.is_empty()
    }

    pub(super) fn from_node(node: &Node) -> (String, Self) {
        let mut children = node.element_children();
        let vote_identification_str = children.next().unwrap().text().unwrap();
        (
            vote_identification_str.to_string(),
            Self {
                vote_identification: vote_identification_str.to_string(),
                ballot_raw_data: children
                    .map(|n| BallotRawData::from_node(&n))
                    .collect::<Vec<_>>(),
            },
        )
    }

    pub(super) fn add_ballots(
        &mut self,
        ballots: &[Ballot],
        decoded_votes: &[Vec<String>],
    ) -> Result<(), ECH0222CalculatedErrorImpl> {
        self.ballot_raw_data.extend(
            &mut ballots
                .iter()
                .map(|b| {
                    BallotRawData::collect_ballot_raw_data_from_decoded_votes(b, decoded_votes)
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten(),
        );
        Ok(())
    }
}

impl ECh0222differencesTrait for VoteRawData {
    fn calculate_differences(&self, expected: &Self) -> Vec<ECH0222Difference> {
        let mut res = vec![];
        if self.vote_identification != expected.vote_identification {
            res.push(ECH0222Difference::new_with_messsage(
                "vote_identification not the same".to_string(),
            ))
        }
        let mut ballot_raw_data_hm: HashMap<u64, &BallotRawData> = HashMap::new();
        let mut ballot_raw_data_histo_hm_self: HashMap<u64, usize> = HashMap::new();
        for raw in self.ballot_raw_data.iter() {
            let mut s = DefaultHasher::new();
            raw.hash(&mut s);
            let hash = s.finish();
            ballot_raw_data_hm.entry(hash).or_insert(raw);
            ballot_raw_data_histo_hm_self
                .entry(hash)
                .and_modify(|v| *v += 1)
                .or_insert(1);
        }
        let mut ballot_raw_data_histo_hm_expected: HashMap<u64, usize> = HashMap::new();
        for raw in expected.ballot_raw_data.iter() {
            let mut s = DefaultHasher::new();
            raw.hash(&mut s);
            let hash = s.finish();
            ballot_raw_data_hm.entry(hash).or_insert(raw);
            ballot_raw_data_histo_hm_expected
                .entry(hash)
                .and_modify(|v| *v += 1)
                .or_insert(1);
        }
        for (hash, self_nb) in ballot_raw_data_histo_hm_self.iter() {
            match ballot_raw_data_histo_hm_expected.get(hash) {
                Some(expected_nd) => {
                    if self_nb != expected_nd {
                        let ballot = ballot_raw_data_hm.get(hash).unwrap();
                        res.push(ECH0222Difference::new_with_messsage(format!(
                            "Found {} ballots and expeted {} for the ballot -> {}",
                            self_nb,
                            expected_nd,
                            ballot.difference_found_text()
                        )));
                    }
                }
                None => res.push(ECH0222Difference::new_with_messsage(format!(
                    "ballot is missing in expected: {}",
                    ballot_raw_data_hm
                        .get(hash)
                        .unwrap()
                        .difference_found_text()
                ))),
            }
        }
        for (hash, _) in ballot_raw_data_histo_hm_expected.iter() {
            if !ballot_raw_data_histo_hm_self.contains_key(hash) {
                res.push(ECH0222Difference::new_with_messsage(format!(
                    "ballot is missing in loaded: {}",
                    ballot_raw_data_hm
                        .get(hash)
                        .unwrap()
                        .difference_found_text()
                )))
            }
        }
        res
    }
}

impl BallotRawData {
    pub(super) fn from_node(node: &Node) -> Self {
        let first_child = node.first_element_child().unwrap();
        Self {
            electronic_ballot_identification: first_child.text().unwrap().to_string(),
            ballot_casted: BallotCasted::from_node(&first_child.next_sibling_element().unwrap()),
        }
    }

    pub(super) fn collect_ballot_raw_data_from_decoded_votes(
        ballot: &Ballot,
        decoded_votes: &[Vec<String>],
    ) -> Result<Vec<Self>, ECH0222CalculatedErrorImpl> {
        decoded_votes
            .iter()
            .enumerate()
            .map(
                |(i, decoded_ballot)| -> Result<Self, ECH0222CalculatedErrorImpl> {
                    Ok(Self {
                        electronic_ballot_identification: ballot.ballot_identification.clone(),
                        ballot_casted: BallotCasted::collect_votes(
                            &ballot.questions(),
                            decoded_ballot,
                        )
                        .map_err(|e| {
                            ECH0222CalculatedErrorImpl::ErrorOnDecodedVote {
                                i,
                                source: Box::new(e),
                            }
                        })?,
                    })
                },
            )
            .collect::<Result<_, _>>()
    }

    fn difference_found_text(&self) -> String {
        let q_text = self
            .ballot_casted
            .question_raw_data
            .iter()
            .map(|q| {
                (
                    &q.question_identification,
                    q.casted.as_ref().map(|a| a.casted_vote).unwrap_or(100),
                )
            })
            .map(|(q, a)| format!("id {} and answer position {}", q, a))
            .collect::<Vec<_>>()
            .join(";");
        format!(
            "ballot_id {} / questions: {} ",
            self.electronic_ballot_identification, q_text
        )
    }
}

impl BallotCasted {
    pub(super) fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        let (ballot_casted_number, question_raw_data) = {
            let first_child = children.next().unwrap();
            match first_child.has_tag_name("ballotCastedNumber") {
                true => (
                    Some(first_child.text().unwrap().parse::<usize>().unwrap()),
                    children
                        .map(|n| QuestionRawData::from_node(&n))
                        .collect::<Vec<_>>(),
                ),
                false => (
                    None,
                    once(QuestionRawData::from_node(&first_child))
                        .chain(children.map(|n| QuestionRawData::from_node(&n)))
                        .collect::<Vec<_>>(),
                ),
            }
        };
        Self {
            ballot_casted_number,
            question_raw_data,
        }
    }

    pub(super) fn collect_votes(
        questions: &[&StandardQuestion],
        decoded_ballot: &[String],
    ) -> Result<Self, ECH0222CalculatedErrorImpl> {
        let q_raw_data = questions
            .iter()
            .map(
                |q| -> Result<(&StandardQuestion, &str), ECH0222CalculatedErrorImpl> {
                    let decoded_vote = decoded_ballot
                        .iter()
                        .find(|dv| dv.starts_with(&q.question_identification))
                        .ok_or(ECH0222CalculatedErrorImpl::QuestionIdMissing {
                            q_id: q.question_identification.clone(),
                        })?;
                    Ok((
                        q,
                        decoded_vote.split("|").nth(1).ok_or(
                            ECH0222CalculatedErrorImpl::MalformedDecodedVote {
                                decoded_vote: decoded_vote.clone(),
                                msg: "Missing second element",
                            },
                        )?,
                    ))
                },
            )
            .map(|res| match res {
                Ok((q, a_id)) => QuestionRawData::new_with(q, a_id),
                Err(e) => Err(e),
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            ballot_casted_number: None,
            question_raw_data: q_raw_data,
        })
    }
}

impl Hash for BallotCasted {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let question_raw_data_sorted = {
            let mut qr = self.question_raw_data.iter().collect::<Vec<_>>();
            qr.sort_by(|q1, q2| q1.question_identification.cmp(&q2.question_identification));
            qr
        };
        self.ballot_casted_number.hash(state);
        question_raw_data_sorted.hash(state);
    }
}

impl QuestionRawData {
    pub(super) fn from_node(node: &Node) -> Self {
        let first_child = node.first_element_child().unwrap();
        let question_identification = first_child.text().unwrap().to_string();
        let casted = first_child.next_sibling().map(|n| Casted::from_node(&n));
        Self {
            question_identification,
            casted,
        }
    }

    pub(super) fn new_with(
        question: &StandardQuestion,
        answer_id: &str,
    ) -> Result<Self, ECH0222CalculatedErrorImpl> {
        Ok(Self {
            question_identification: question.question_identification.clone(),
            casted: Some(Casted::from(
                question
                    .answers
                    .iter()
                    .find(|a| a.answer_identification == answer_id)
                    .ok_or(ECH0222CalculatedErrorImpl::AnswerIdMissing {
                        a_id: answer_id.to_string(),
                        q_id: question.question_identification.clone(),
                    })?,
            )),
        })
    }
}

impl Casted {
    pub(super) fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        Self {
            casted_vote: children
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<usize>()
                .unwrap(),
            answer_option_identification: children
                .next()
                .map(|n| AnswerOptionIdentification::from_node(&n)),
        }
    }
}

impl From<&Answer> for Casted {
    fn from(value: &Answer) -> Self {
        Self {
            casted_vote: value.answer_position,
            answer_option_identification: Some(AnswerOptionIdentification::from(value)),
        }
    }
}

impl AnswerOptionIdentification {
    pub(super) fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        Self {
            answer_option_identification: children.next().unwrap().text().unwrap().to_string(),
            answer_sequence_number: children
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<usize>()
                .unwrap(),
            answer_text_information: children
                .map(|n| AnswerTextInformation::from_node(&n))
                .collect::<Vec<_>>(),
        }
    }
}

impl From<&Answer> for AnswerOptionIdentification {
    fn from(value: &Answer) -> Self {
        Self {
            answer_option_identification: value.answer_identification.clone(),
            answer_sequence_number: value.answer_position,
            answer_text_information: value
                .answer_info
                .iter()
                .map(AnswerTextInformation::from)
                .collect(),
        }
    }
}

impl Hash for AnswerOptionIdentification {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let answer_text_information_sorted = {
            let mut ati = self.answer_text_information.iter().collect::<Vec<_>>();
            ati.sort_by(|a1, a2| a1.language.cmp(&a2.language));
            ati
        };
        self.answer_option_identification.hash(state);
        self.answer_sequence_number.hash(state);
        answer_text_information_sorted.hash(state);
    }
}

impl AnswerTextInformation {
    pub(super) fn from_node(node: &Node) -> Self {
        Self {
            language: node
                .first_element_child()
                .unwrap()
                .text()
                .unwrap()
                .to_string(),
            answer_text: node
                .element_children()
                .find(|n| n.has_tag_name("answerText"))
                .unwrap()
                .text()
                .unwrap()
                .to_string(),
        }
    }
}

impl From<&AnswerInfo> for AnswerTextInformation {
    fn from(value: &AnswerInfo) -> Self {
        Self {
            language: value.language.clone(),
            answer_text: value.answer.clone(),
        }
    }
}
