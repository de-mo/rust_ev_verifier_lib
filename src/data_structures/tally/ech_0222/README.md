# Comparison of eCH-0222

## Introduction

According to Swiss Post (Algorithm 4.4), the verification of eCH-0222 is done creating a new one, and calculating the signature. The hash of the delivered one and of the caluclating one must be the same. This is only possible if the two files are exactly constructed on the same way (sorting, namespaces, etc.).

The proposed implementation is different than the proposed one by Swiss Post. We propose an alternative way to validate the eCH-0222 through validation of business logics, which reduce the dependencies and the complication of the generation of the XML.

## Some information about eCH-0222

The file eCH-0222 is defined by the eCH standard [eCH-0222 Version 1.2.0](https://www.ech.ch/fr/ech/ech-0222/1.2.0).

Swiss Post took the following decisions by the implementation of the standard:
- Votations:
    - Pro counting circle only on `voteRawData` pro votation is delivered. 
    - All the ballots `ballotRawData` concerning the same votation are grouped under `voteRawData`
    - There is one `ballotRawData` with all the questions pro casted vote
- Elections:
    - Pro counting circle there is one `electionGroupBallotRawData` pro casted vote

## Definitions

The delivered file is imported in a structure `data_imported`, containing all the information of the eCH-0222.  In the same structure, the values of `data_calculated` will be loaded, based on the tally and config data.

The algorithm validates that `data_imported` and `data_calculated` are similar.

## Creation of `data_calculated`

### Introduction

The data structure `data_calculated` will be loaded, based on the tally and config data.
- `ElectionEventConfiguration` (configuration.xml)
- `ElectionEventContext`
- `TallyComponentVotesPayload`, for each ballot box

A ballot box is mapped to only one counting circle. Many counting circles can be merged to one counting circle (in case of voters in the same municipality, but with different voting rights)

The construction of the `decodedVotes` is defined in the specifications of the E-Voting system of Swiss Post.

### Creation algorithm

**Input**
- `ElectionEventConfiguration`
- `ElectionEventContext`
- All the `TallyComponentVotesPayload` (for each ballot box)

**Steps**
1. Create the `rawData` with the constest identification. It is in the `ElectionEventContext`.
2. For each ballot box:
    1. Find the counting circle id, the relevant votations and the relevant election groups in `ElectionEventConfiguration` ([algorithm](#find-the-counting-circle-id-the-relevant-votations-and-the-relevant-election-groups)).
    2. If the counting circle does not exist in `rawData`, create a new one in `rawData` with the following values:
        - counting circle id 
        - Default `votingCardsInformation` where both values (valid votes and invalid votes) are `0` 
    3. Update the `votingCardsInformation` of the counting circle with the ballot ([algorithm](#update-the-votingcardsinformation))
    4. For each relevant votation, add the votes concerning this votation from the ballots ([algorithm](#add-the-votation))
    5. Collect all the `electionGroupBallotRawData` based on the relevation election groups, the decoded votes and the decoded write-ins ([algorithm](#collect-the-electiongrouprawdata))

**Output**
- `rawData`

### Find the counting circle id, the relevant votations and the relevant election groups

**Input**
- `ElectionEventConfiguration`
- `ElectionEventContext`
- `TallyComponentVotesPayload`

**Steps**
1. Take the ballot box identification in `TallyComponentVotesPayload` (field `ballotBoxId`)
2. In the `ElectionEventContext` find in `verificationCardSetContexts` the verification card set for the ballot box identification
3. Calculate the authorisation id: take `verificationCardSetAlias` and remove `"vcs_"`
4. In `ElectionEventConfiguration`, find the authorization with the calculated authorization id
5. The counting circle id is the first entry in the domain of influences of the authorization -> counting circle id
6. Take all the domain of influence identifications in the authorization
7. Find all the votations with the taken domain of influence identifications -> Relevant votations
8. Find all the election groups with the taken domain of influence identifications -> Relevant election groups

**Output**
- counting circle id
- relevant votations
- relevant election groups

### Update the `votingCardsInformation`

**Input**
- `TallyComponentVotesPayload`

**Steps**
1. Add the number of elements of `decoded_votes` to the `countOfReceivedValidVotingCardsTotal`

### Add the votation

**Input**
- Votation from `ElectionEventConfiguration`
- `decodedVotes` from `TallyComponentVotesPayload`

**Steps**
1. If `voteRawData` for the `voteIdentification` does not exist, create it
2. Collect the `ballotRawData` for all the decoded ballots in the votation and all the `decodedVotes` and add them to `voteRawData` ([algorithm](#collect-ballotrawdata))
2. If `voteRawData` is empty (no ballots for this votation), remove it (not empty `voteRawData` are not allowed in eCH-0222)

### Collect `ballotRawData`

**Input**
- `ballot` from `ElectionEventConfiguration`
- `decodedVotes` from `TallyComponentVotesPayload`

**Steps**
1. for each decoded ballot in the `decodedVotes`, create a `ballotRawData` 
    1. Set the id as the `electronic_ballot_identification` of the `ballot` 
    2. Set the `ballotCasted` as the list of votes based on the questions of the `ballot` and the corresponding entry decoded ballot ([algorithm](#collect-the-votes-for-ballotcasted))

**Output**
- List of reated `ballotRawData`

### Collect the votes for `ballotCasted`

**Input**
- List of questions from `ballot` from `ElectionEventConfiguration`
- One decoded ballot from `decodedVotes` from `TallyComponentVotesPayload`

**Steps**
1. For each question in the questions
    1. Find the the relevant answer in the decoded ballot, where the first part of the decoded option (before "|") is the question id
    2. Create a `VoteCasted` where 
        - `ballot_casted_number` is null 
        - `question_raw_data` is get from the question and from the answer id, which is the second part of the decoded option (after "|") ([algorithm](#create-the-questionrawdata)) 

**Output**
- List of created `VoteCasted`

### Create the `questionRawData`

**Input**
- Question from `ballot` from `ElectionEventConfiguration`
- Answer id from decoded option

**Steps**
1. Find the answer in `electionEventConfiguration` with the answer id
1. Create a `questionRawData` where
    - `questionIdentification` is the question id
    - `casted` is calculated as  trivial way regarding the definition of eCH-0222

**Output**
- Created `questionRawData`

### Collect the `electionGroupRawData`

**Input**
- Relevant election groups from `ElectionEventConfiguration`
- `decodedVotes` from `TallyComponentVotesPayload`
- `decodedWriteIns` from `TallyComponentVotesPayload`

**Steps**
1. for each decoded ballot in the `decodedVotes`, 
    1. Take the decoded writeins at the same position in `decodedWriteins`
    2. Calculate the association between each decoded vote and the decoded write-in ([algorithm](#association-between-decoded-votes-and-decoded-write-ins)) -> Wrtieins association
    3. For each relevant election group, create an `electionGroupRawData` 
        - Set the id as the `election_group_identification` of the `electionGroup`
        - Set the `election_raw_data` as the list of elections `electionRawData` ([algorithm](#create-the-electionrawdata))

**Output**
- List of created `electionGroupRawData`

### Association between decoded votes and decoded write-ins

**Input**
- decoded ballot
- decoded writeins

**Steps**
1. Extract the vote with writeins in the decoded ballot (the id in the 2. position in the decoded vote is a `write_in_position_identification` in `electionEventConfiguration`)
2. For each `i` from `0` to the length of the extracted votes with writeins
    - associated the vote with writein `i` with the decoded write-in at position `i`

**Output**
- Association between decoded votes and decoded write-ins

### Create the `electionRawData`

**Input**
- Election from `electionEventConfiguration`
- Decoded ballot
- Association between decoded votes and decoded write-ins

**steps**
1. Extract the votes in the decoded ballot for the election (the id in the 1. position in the decoded vote is the `election_identification` in the election, incl empty list)
2. Create `electionRawData` with the following information
    1. `electionIdentification` is the election id
    2. if a `list_identification` is in the decoded vote (2. position of the decoded vote), set `listRawData` accordingly (trivial way regarding the definition of eCH-0222)
    3. Calculate the list of `ballotPosition` based on `listRawData`, the extracted votes and the association between decoded votes and decoded write-ins ([algorithm](#create-the-list-of-ballotposition))
    4. Set `isUnchangedBallot` ([algorithm](#determine-if-unchanged-ballot))

**output**
- Created `electionRawData`

### Create the list of `ballotPosition`

**Input**
- Election from `electionEventConfiguration`
- `listRawData` (can be null)
- Extracted votes
- Association between decoded votes and decoded write-ins

**Steps**
1. For each decoded vote in the extracted votes that are not a list vote (the id in the 2. position of the decoded vote is not a `list_identification` in `electionEventConfiguration`)
    1. Create a `ballotPosition` as follow, using the 2. position of the decoded vote:
        - if the 2. position of the decoded vote is related to an entry in the empty list, set the ballot position as `isEmpty` (trivial way regarding the definition of eCH-0222)
        - if the 2. position of the decoded vote is related to a write-in position, set the ballot position as a write-in candidate using the associated decoded write-in (trivial way regarding the definition of eCH-0222)
        - Else the 2. position of the decoded vote refers to a candidate. set the ballot possition as the candidate with the following informations:
            - `candidateIdentification` as the 2. position of the decoded vote
            - `candidateReferenceOnPosition` according the the [algorithm](#determine-the-candidatereferenceonposition)

**output**
- Created list of `ballotPosition`

### Determine the `candidateReferenceOnPosition`

**Input**
- Election from `electionEventConfiguration`
- Candidate id (2. position of the decoded vote)
- Candidate accumulation (3. position of the decoded vote, can be null)

**Steps**
1. Lookup the `candidatePosition` with `candidateIdentification` equals candidate id (2. position of the decoded vote) in the lists of the election. If many entries found (pre-accumuation), take the entries according to the candidate accumulation (3. position of the decoded vote).
2. if `candidatePosition` is null
    1. Find the `candidate` with `candidateIdentification` equals candidate id (2. position of the decoded vote) in the election.
    2. Return `candidateReferenceOnPosition` as `candidateReferenceOnPosition` of the  found `candidate`
3. else
    1. Return `candidateReferenceOnPosition` as `candidateReferenceOnPosition` of the found `candidatePosition`

**Output**
- Calculated `candidateReferenceOnPosition`

### Determine if unchanged ballot

**Input**
- Election from `electionEventConfiguration`
- `listRawData` (can be null)
- List of `ballotPosition`

**Steps**
1. If `listRawData` is null, return false
2. Else
    1. If `listRawData` is the empty list, return true if all `ballotPosition` are `isEmpty`, else return false
    2. Else
        1. If the candidates of `ballotPosition` are exactly the candidates of `listRawData` (incl accumulation and sorting), return true, else return false

**Output**
- `isUnchangedBallot`

## Comparison Algorithm

### Scope and restrictions

The following conditions are used for the algorithm
- The file eCH-0222 is generated by Tally and delivered to the Verifier
- The file ist compliant to the specification of the standard, version 1.2.0, and thus to the schema (Version 3.0)
- The signature of the file is validated

Following restrictions apply to the algorithm
- Only the values that are business relevant are validated. The following information are not validated:
    - `reportingBody`: This information has no impact to the correctnes of the results
    - `exentsion`: This optional structure is used to add some additional information. The result must be correct without these additional informations. Swiss Post don't deliver any data actually.

The algorithm lists all differences between `data_imported` and `data_calculated`. It starts the comparison for the structure `rawData`.

### Compare the main structure

**Input**
- `rawData_calculated`: `rawData` of `data_imported`
- `rawData_imported`: `rawData` of `data_calculated`

**Steps**
1. Compare `contestIdentification` of both `rawData_calculated` and `rawData_imported`
2. Validate that `rawData_calculated` and `rawData_imported` have exactly the same counting circles based on `countingCircleIdentification`. Any missing counting circle in one or other raw data will be outputed. The order of the counting circles is not relevant.
3. For each counting circle in both `rawData_calculated` and `rawData_imported`, compare the content of the counting circle ([algorithm](#compare-a-counting-circle))

**Output**
- All differences found in the steps 1 to 3

### Compare a counting circle

**Input**
- `cc_calculated`
- `cc_imported`

**Steps**
1. Compare `countingCircleIdentification` of both `cc_calculated` and `cc_imported`
2. compare `votingCardsInformation` of both `cc_calculated` and `cc_imported`
3. Validate that `cc_calculated` and `cc_imported` have exactly the same votations (`voteRawData`) based on `voteIdentification`. Any missing votation in one or other raw data will be outputed. The order of the votations is not relevant.
4. For each `voteRawData` in both `cc_calculated` and `cc_imported`, compare the content of the votation ([algorithm](#compare-voterawdata))
5. Validate that `cc_calculated` and `cc_imported` have exactly the same entries of `electionGroupBallotRawData` (order is not relevant).

**Output**
- All differences found in the steps 1 to 5

### Compare voteRawData

**Input**
- `voteRawData_calculated`
- `voteRawData_imported`

**Steps**
1. Compare `voteIdentification` of both `voteRawData_calculated` and `voteRawData_imported`
2. Validate that `voteRawData_calculated` and `voteRawData_imported` have exactly the same entries of `ballotRawData` (order is not relevant).


**Output**
- All differences found in the steps 1 to 2

