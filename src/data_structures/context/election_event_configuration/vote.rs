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
pub struct Vote {
    pub vote_identification: String,
    pub domain_of_influence: String,
    pub vote_position: usize,
    pub ballot: Vec<Ballot>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ballot {
    pub ballot_identification: String,
    pub standard_ballot: Option<StandardQuestion>,
    pub variant_ballot: Option<VariantBallot>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariantBallot {
    pub standard_question: Vec<StandardQuestion>,
    pub tie_break_question: Vec<StandardQuestion>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StandardQuestion {
    pub question_identification: String,
    pub ballot_question: BallotQuestion,
    pub answer: Vec<Answer>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BallotQuestion {
    pub ballot_question_info: Vec<BallotQuestionInfo>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BallotQuestionInfo {
    pub language: String,
    pub ballot_question: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    pub answer_identification: String,
    pub hidden_answer: Option<bool>,
    pub answer_info: Vec<AnswerInfo>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnswerInfo {
    pub language: String,
    pub answer: String,
}
