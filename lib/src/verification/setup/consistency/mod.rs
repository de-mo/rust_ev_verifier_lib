mod v0301_encryption_group_consistency;
mod v0302_setup_file_names_consistency;
mod v0303_ccr_choice_return_codes_pk_consistency;
mod v0304_ccm_election_pk_consistency;
mod v0305_ccm_and_ccr_schnorr_proofs_consistency;
mod v0306_choice_return_codes_public_key_consistency;
mod v0307_election_pk_consistency;
mod v0308_primes_mapping_table_consistency;
mod v0309_election_event_id_consistency;
mod v0313_total_voters_consistency;
mod v0315_chunk_consistency;

use super::super::{
    meta_data::VerificationMetaDataList, suite::VerificationList, verification::Verification,
};

pub(crate) fn get_verifications(metadata_list: &VerificationMetaDataList) -> VerificationList {
    let mut res = vec![];
    res.push(
        Verification::new(
            "03.01",
            v0301_encryption_group_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.02",
            v0302_setup_file_names_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.03",
            v0303_ccr_choice_return_codes_pk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.04",
            v0304_ccm_election_pk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.05",
            v0305_ccm_and_ccr_schnorr_proofs_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.06",
            v0306_choice_return_codes_public_key_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.07",
            v0307_election_pk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.08",
            v0308_primes_mapping_table_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.09",
            v0309_election_event_id_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.13",
            v0313_total_voters_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    res.push(
        Verification::new(
            "03.15",
            v0315_chunk_consistency::fn_verification,
            metadata_list,
        )
        .unwrap(),
    );
    VerificationList(res)
}
