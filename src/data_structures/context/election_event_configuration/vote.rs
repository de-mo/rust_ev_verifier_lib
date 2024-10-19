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
