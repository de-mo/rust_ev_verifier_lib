pub mod v300_encryption_group_consistency;
pub mod v301_setup_file_names_consistency;
pub mod v302_ccr_choice_return_codes_pk_consistency;
pub mod v303_ccm_election_pk_consistency;
pub mod v304_ccm_and_ccr_schnorr_proofs_consistency;
pub mod v305_choice_return_codes_public_key_consistency;
pub mod v306_election_pk_consistency;
pub mod v307_primes_mapping_table_consistency;
pub mod v308_election_event_id_consistency;
pub mod v312_total_voters_consistency;
pub mod v314_chunk_consistency;

use super::super::{
    meta_data::VerificationMetaDataList, verification::Verification,
    verification_suite::VerificationList,
};

pub fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(
        Verification::new(
            "s300",
            v300_encryption_group_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s301",
            v301_setup_file_names_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s302",
            v302_ccr_choice_return_codes_pk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s303",
            v303_ccm_election_pk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s304",
            v304_ccm_and_ccr_schnorr_proofs_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s305",
            v305_choice_return_codes_public_key_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s306",
            v306_election_pk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s307",
            v307_primes_mapping_table_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s308",
            v308_election_event_id_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s312",
            v312_total_voters_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "s314",
            v314_chunk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res
}
