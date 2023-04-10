//! Module implementing the verifications for setup
pub mod authenticity;
pub mod completness;
pub mod consistency;
pub mod evidence;
pub mod integrity;

use super::VerificationList;

/// Collect the verifications of the submodules
pub fn get_verifications() -> VerificationList {
    let mut res: VerificationList = vec![];
    res.append(&mut authenticity::get_verifications());
    res.append(&mut completness::get_verifications());
    res.append(&mut consistency::get_verifications());
    res.append(&mut evidence::get_verifications());
    res.append(&mut integrity::get_verifications());
    res
}
