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

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectionInformation {
    pub election: Election,
    pub candidate: Vec<Candidate>,
    pub list: Vec<List>,
    pub write_in_candidate: Option<Vec<WriteInCandidate>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub election_identification: String,
    pub type_of_election: usize,
    pub number_of_mandates: usize,
    pub write_ins_allowed: bool,
    pub candidate_accumulation: usize,
    pub minimal_candidate_selection_in_list: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub candidate_identification: String,
    pub family_name: String,
    pub first_name: String,
    pub call_name: String,
    pub date_of_birth: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub list_identification: String,
    pub list_description: ListDescription,
    pub list_empty: bool,
    pub candidate_position: Vec<CandidatePosition>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListDescription {
    pub list_description_info: Vec<ListDescriptionInfo>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListDescriptionInfo {
    pub language: String,
    pub list_description: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandidatePosition {
    pub candidate_list_identification: String,
    pub position_on_list: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WriteInCandidate {
    pub write_in_candidate_identification: String,
    pub position: usize,
}
