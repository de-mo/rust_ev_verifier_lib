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

use rust_ev_system_library::preliminaries::PTableElement;

use crate::{
    data_structures::{
        context::{
            election_event_configuration::{
                Answer, Candidate, Election, ElectionInformation, List, StandardQuestion, Vote,
                WriteInCandidate,
            },
            election_event_context_payload::ElectionEventContext,
        },
        ElectionEventConfiguration,
    },
    verification::{VerificationEvent, VerificationResult},
};
use std::{collections::HashMap, slice::Iter};

pub fn verification_2_3_same_than_xml(
    ee_context: &ElectionEventContext,
    ee_config: &ElectionEventConfiguration,
) -> VerificationResult {
    let mut result = VerificationResult::new();
    for vcs_context in &ee_context.verification_card_set_contexts {
        let p_table_calculated = match generate_all_p_table_elements(
            &vcs_context.verification_card_set_alias,
            ee_config,
        ) {
            Ok(r) => r
                .iter()
                .map(|e| (e.actual_voting_option.clone(), e.clone()))
                .collect::<HashMap<_, _>>(),
            Err(e) => {
                result.push(VerificationEvent::new_error(&e).add_context(format!(
                    "calculating p_table for vcs_id {}",
                    vcs_context.verification_card_set_id
                )));
                continue;
            }
        };
        for p_table_element in &vcs_context.primes_mapping_table.p_table {
            let p_table_element_without_encoded =
                PTableElementWithoutEncoding::from(p_table_element);
            match p_table_calculated.get(&p_table_element.actual_voting_option) {
                Some(elt) => {
                    if elt != &p_table_element_without_encoded {
                        result.push(
                        VerificationEvent::new_failure(&format!(
                            "PTable Element not the same for the actual voting option {}\n Original: {:?} \n Calculated: {:?}",
                            &p_table_element.actual_voting_option, &p_table_element_without_encoded, elt
                        ))
                        .add_context(format!(
                            "Verification 2 for vcs_id {}",
                            vcs_context.verification_card_set_id
                        )),
                    )
                    }
                }
                None => result.push(
                    VerificationEvent::new_failure(&format!(
                        "voting option {} not found in xml",
                        &p_table_element.actual_voting_option
                    ))
                    .add_context(format!(
                        "Verification 2 for vcs_id {}",
                        vcs_context.verification_card_set_id
                    )),
                ),
            }
        }
        let expected_count = p_table_calculated.len();
        let p_table_count = vcs_context.primes_mapping_table.p_table.len();
        if expected_count != p_table_count {
            result.push(
            VerificationEvent::new_failure(&format!(
                "The number of entries in the p_table (={p_table_count}) is not the same than the number of entries calculated (={expected_count})"
            ))
            .add_context(format!(
                "Verification 2 for vcs_id {}",
                vcs_context.verification_card_set_id
            )),
        )
        }
    }
    result
}

/// Element in pTable according the spefication of Swiss Post
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PTableElementWithoutEncoding {
    pub actual_voting_option: String,
    pub semantic_information: String,
    pub correctness_information: String,
}

impl From<&PTableElement> for PTableElementWithoutEncoding {
    fn from(value: &PTableElement) -> Self {
        Self {
            actual_voting_option: value.actual_voting_option.clone(),
            semantic_information: value.semantic_information.clone(),
            correctness_information: value.correctness_information.clone(),
        }
    }
}

macro_rules! generate_text_4_languages {
    ($elt: expr, $l: ident, $f: ident) => {
        ["de", "fr", "it", "rm"]
            .iter()
            .map(|&l| match $elt.iter().find(|v| v.$l.as_str() == l) {
                Some(v) => v.$f.clone(),
                None => String::default(),
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("|")
            .as_str()
    };
}

fn generate_all_p_table_elements(
    vcs_alias: &str,
    ee_config: &ElectionEventConfiguration,
) -> Result<Vec<PTableElementWithoutEncoding>, String> {
    let auth_id = vcs_alias.chars().skip(4).collect::<String>();
    let auth = match ee_config
        .authorizations
        .iter()
        .map_err(|e| format!("Error getting authorization {}: {}", &auth_id, e))?
        .find(|a| a.authorization_identification == auth_id)
    {
        Some(a) => a,
        None => return Err(format!("Authorization id {auth_id} not found")),
    };
    let mut res = vec![];
    res.append(
        &mut ee_config
            .contest
            .votes
            .iter()
            .map_err(|e| format!("Error iterating votes: {e}"))?
            .filter(|v| v.has_authorization(&auth))
            .flat_map(|v| generate_p_table_for_vote(&v.vote))
            .collect(),
    );
    res.append(
        &mut ee_config
            .contest
            .election_groups
            .iter()
            .map_err(|e| format!("Error iterating election groups: {e}"))?
            .filter(|v| v.has_authorization(&auth))
            .flat_map(|eg| {
                eg.election_information
                    .iter()
                    .flat_map(generate_p_table_for_election_information)
                    .collect::<Vec<_>>()
            })
            .collect(),
    );
    Ok(res)
}

fn generate_p_table_element_for_answer(
    answer: &Answer,
    question: &StandardQuestion,
) -> PTableElementWithoutEncoding {
    PTableElementWithoutEncoding {
        actual_voting_option: format!(
            "{}|{}",
            question.question_identification, answer.answer_identification
        ),
        semantic_information: [
            match answer.hidden_answer {
                Some(true) => "BLANK",
                _ => "NON_BLANK",
            },
            generate_text_4_languages!(
                question.ballot_question.ballot_question_info.as_slice(),
                language,
                ballot_question
            ),
            generate_text_4_languages!(answer.answer_info.as_slice(), language, answer),
        ]
        .join("|"),
        correctness_information: question.question_identification.clone(),
    }
}

fn generate_p_table_for_question(question: &StandardQuestion) -> Vec<PTableElementWithoutEncoding> {
    question
        .answer
        .iter()
        .map(|answer| generate_p_table_element_for_answer(answer, question))
        .collect()
}

fn generate_p_table_for_vote(vote: &Vote) -> Vec<PTableElementWithoutEncoding> {
    let mut res = vec![];
    for ballot in vote.ballot.iter() {
        if let Some(standard_ballot) = &ballot.standard_ballot {
            res.append(&mut generate_p_table_for_question(standard_ballot));
        }
        if let Some(variant_ballot) = &ballot.variant_ballot {
            res.append(
                &mut variant_ballot
                    .standard_question
                    .iter()
                    .flat_map(generate_p_table_for_question)
                    .collect(),
            );
            res.append(
                &mut variant_ballot
                    .tie_break_question
                    .iter()
                    .flat_map(generate_p_table_for_question)
                    .collect(),
            );
        }
    }
    res
}

fn generate_p_table_for_election_information(
    election_info: &ElectionInformation,
) -> Vec<PTableElementWithoutEncoding> {
    let mut res = vec![];
    // lists, only if proportional election
    if election_info.election.type_of_election == 1 {
        res.append(
            &mut election_info
                .list
                .iter()
                .map(|l| generate_p_table_element_for_list(l, &election_info.election))
                .collect::<Vec<_>>(),
        );
    }
    // candidates
    res.append(
        &mut election_info
            .candidate
            .iter()
            .flat_map(|c| generate_p_table_element_for_candidate(c, &election_info.election))
            .collect::<Vec<_>>(),
    );
    // write in candidates
    if let Some(write_in_candidates) = &election_info.write_in_candidate {
        res.append(&mut generate_p_table_element_for_write_in_candidate(
            write_in_candidates.as_slice(),
            &election_info.election,
        ));
    }
    // Empty positions
    res.append(&mut generate_p_table_element_for_blank_position_candidate(
        &mut election_info.list.iter(),
        &election_info.election,
    ));
    res
}

fn generate_p_table_element_for_list(
    list: &List,
    election: &Election,
) -> PTableElementWithoutEncoding {
    PTableElementWithoutEncoding {
        actual_voting_option: format!(
            "{}|{}",
            election.election_identification, list.list_identification
        ),
        semantic_information: [
            match list.list_empty {
                true => "BLANK",
                _ => "NON_BLANK",
            },
            generate_text_4_languages!(
                list.list_description.list_description_info.as_slice(),
                language,
                list_description
            ),
        ]
        .join("|"),
        correctness_information: format!("L|{}", election.election_identification),
    }
}

fn generate_p_table_element_for_candidate(
    candidate: &Candidate,
    election: &Election,
) -> Vec<PTableElementWithoutEncoding> {
    (0..election.candidate_accumulation)
        .map(|i| PTableElementWithoutEncoding {
            actual_voting_option: format!(
                "{}|{}|{}",
                election.election_identification, candidate.candidate_identification, i
            ),
            semantic_information: [
                "NON_BLANK",
                &candidate.family_name,
                &candidate.first_name,
                &candidate.call_name,
                &candidate.date_of_birth,
            ]
            .join("|"),
            correctness_information: format!("C|{}", election.election_identification),
        })
        .collect()
}

fn generate_p_table_element_for_write_in_candidate(
    candidates: &[WriteInCandidate],
    election: &Election,
) -> Vec<PTableElementWithoutEncoding> {
    candidates
        .iter()
        .map(|c| PTableElementWithoutEncoding {
            actual_voting_option: format!(
                "{}|{}",
                election.election_identification, c.write_in_candidate_identification
            ),
            semantic_information: [
                "WRITE_IN",
                format!("WRITE_IN_POSITION-{}", c.position).as_str(),
            ]
            .join("|"),
            correctness_information: format!("C|{}", election.election_identification),
        })
        .collect()
}

fn generate_p_table_element_for_blank_position_candidate(
    it_lists: &mut Iter<'_, List>,
    election: &Election,
) -> Vec<PTableElementWithoutEncoding> {
    it_lists
        .filter(|l| l.list_empty)
        .flat_map(|l| {
            l.candidate_position
                .iter()
                .map(|c| PTableElementWithoutEncoding {
                    actual_voting_option: format!(
                        "{}|{}",
                        election.election_identification, c.candidate_list_identification
                    ),
                    semantic_information: [
                        "BLANK",
                        format!("EMPTY_CANDIDATE_POSITION-{}", c.position_on_list).as_str(),
                    ]
                    .join("|"),
                    correctness_information: format!("C|{}", election.election_identification),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
