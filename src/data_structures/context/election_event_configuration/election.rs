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
