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

use roxmltree::Node;

use crate::data_structures::xml::ElementChildren;

#[derive(Debug, Clone)]
pub struct Vote {
    pub vote_identification: String,
    pub domain_of_influence: String,
    pub vote_position: usize,
    pub ballots: Vec<Ballot>,
}

#[derive(Debug, Clone)]
pub struct Ballot {
    pub ballot_identification: String,
    pub ballot_position: usize,
    pub standard_ballot: Option<StandardQuestion>,
    pub variant_ballot: Option<VariantBallot>,
}

#[derive(Debug, Clone)]
pub struct VariantBallot {
    pub standard_questions: Vec<StandardQuestion>,
    pub tie_break_questions: Vec<StandardQuestion>,
}

#[derive(Debug, Clone)]
pub struct StandardQuestion {
    pub question_identification: String,
    pub ballot_question: BallotQuestion,
    pub answers: Vec<Answer>,
}

#[derive(Debug, Clone)]
pub struct BallotQuestion {
    pub ballot_question_info: Vec<BallotQuestionInfo>,
}

#[derive(Debug, Clone)]
pub struct BallotQuestionInfo {
    pub language: String,
    pub ballot_question: String,
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub answer_identification: String,
    pub answer_position: usize,
    pub hidden_answer: Option<bool>,
    pub answer_info: Vec<AnswerInfo>,
}

#[derive(Debug, Clone)]
pub struct AnswerInfo {
    pub language: String,
    pub answer: String,
}

impl Vote {
    pub fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        let vote_identification = children.next().unwrap().text().unwrap().to_string();
        let domain_of_influence = children.next().unwrap().text().unwrap().to_string();
        children.next(); // voteDescription
        let vote_position = children
            .next()
            .unwrap()
            .text()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let ballots = node
            .element_children()
            .filter(|n| n.has_tag_name("ballot"))
            .map(|vi| Ballot::from_node(&vi))
            .collect::<Vec<_>>();
        Self {
            vote_identification,
            domain_of_influence,
            vote_position,
            ballots,
        }
    }
}

impl Ballot {
    pub fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        let ballot_identification = children.next().unwrap().text().unwrap().to_string();
        let ballot_position = children
            .next()
            .unwrap()
            .text()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let standard_ballot = node
            .element_children()
            .find(|n| n.has_tag_name("standardBallot"))
            .map(|n| StandardQuestion::from_node(&n));
        let variant_ballot = node
            .element_children()
            .find(|n| n.has_tag_name("variantBallot"))
            .map(|n| VariantBallot::from_node(&n));
        Self {
            ballot_identification,
            ballot_position,
            standard_ballot,
            variant_ballot,
        }
    }
}

impl StandardQuestion {
    pub fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        let question_identification = node
            .first_element_child()
            .unwrap()
            .text()
            .unwrap()
            .to_string();
        let ballot_question = node
            .element_children()
            .find(|n| n.has_tag_name("ballotQuestion"))
            .map(|n| BallotQuestion::from_node(&n))
            .unwrap();
        BallotQuestion::from_node(&children.next().unwrap());
        let answers = node
            .element_children()
            .filter(|n| n.has_tag_name("answer"))
            .map(|n| Answer::from_node(&n))
            .collect::<Vec<_>>();
        Self {
            question_identification,
            ballot_question,
            answers,
        }
    }
}

impl VariantBallot {
    pub fn from_node(node: &Node) -> Self {
        Self {
            standard_questions: node
                .element_children()
                .filter(|n| n.has_tag_name("standardQuestion"))
                .map(|n| StandardQuestion::from_node(&n))
                .collect::<Vec<_>>(),
            tie_break_questions: node
                .element_children()
                .filter(|n| n.has_tag_name("tieBreakQuestion"))
                .map(|n| StandardQuestion::from_node(&n))
                .collect::<Vec<_>>(),
        }
    }
}

impl BallotQuestion {
    pub fn from_node(node: &Node) -> Self {
        Self {
            ballot_question_info: node
                .element_children()
                .map(|n| BallotQuestionInfo::from_node(&n))
                .collect::<Vec<_>>(),
        }
    }
}

impl BallotQuestionInfo {
    pub fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        let language = children.next().unwrap().text().unwrap().to_string();
        children.next(); // ballotQuestionTitle
        let ballot_question = children.next().unwrap().text().unwrap().to_string();
        Self {
            language,
            ballot_question,
        }
    }
}

impl Answer {
    pub fn from_node(node: &Node) -> Self {
        let mut children = node.element_children();
        let answer_identification = children.next().unwrap().text().unwrap().to_string();
        let answer_position = children
            .next()
            .unwrap()
            .text()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        Self {
            answer_identification,
            answer_position,
            hidden_answer: node
                .element_children()
                .find(|n| n.has_tag_name("hiddenAnswer"))
                .map(|n| n.text().unwrap() == "true"),
            answer_info: node
                .element_children()
                .filter(|n| n.has_tag_name("answerInfo"))
                .map(|n| AnswerInfo::from_node(&n))
                .collect::<Vec<_>>(),
        }
    }
}

impl AnswerInfo {
    pub fn from_node(node: &Node) -> Self {
        Self {
            language: node
                .first_element_child()
                .unwrap()
                .text()
                .unwrap()
                .to_string(),
            answer: node
                .element_children()
                .find(|n| n.has_tag_name("answer"))
                .unwrap()
                .text()
                .unwrap()
                .to_string(),
        }
    }
}
