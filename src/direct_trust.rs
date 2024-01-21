use anyhow::Context;
use rust_ev_crypto_primitives::{verify_signature, ByteArray, HashableMessage, Keystore};

/// List of valide Certificate authorities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateAuthority {
    Canton,
    SdmConfig,
    SdmTally,
    VotingServer,
    ControlComponent1,
    ControlComponent2,
    ControlComponent3,
    ControlComponent4,
}

impl CertificateAuthority {
    pub fn get_ca_cc(node: &usize) -> Option<Self> {
        match node {
            1 => Some(Self::ControlComponent1),
            2 => Some(Self::ControlComponent2),
            3 => Some(Self::ControlComponent3),
            4 => Some(Self::ControlComponent4),
            _ => None,
        }
    }
}

impl From<CertificateAuthority> for String {
    fn from(value: CertificateAuthority) -> Self {
        match value {
            CertificateAuthority::Canton => "canton".to_string(),
            CertificateAuthority::SdmConfig => "sdm_config".to_string(),
            CertificateAuthority::SdmTally => "sdm_tally".to_string(),
            CertificateAuthority::VotingServer => "voting_server".to_string(),
            CertificateAuthority::ControlComponent1 => "control_component_1".to_string(),
            CertificateAuthority::ControlComponent2 => "control_component_2".to_string(),
            CertificateAuthority::ControlComponent3 => "control_component_3".to_string(),
            CertificateAuthority::ControlComponent4 => "control_component_4".to_string(),
        }
    }
}

/// Trait that must be implemented for each object implementing a signature to be verified
///
/// The following function are to be implemented for the object to make it running:
/// - [VerifiySignatureTrait::get_hashable] Get the [HashableMessage] for the object
/// - [VerifiySignatureTrait::get_context_data] Get the context data as [HashableMessage] for the object, according to the specifications
/// - [VerifiySignatureTrait::get_certificate_authority] Certificate Authority of the certificate to fin the certificate in the keystore
/// - [VerifiySignatureTrait::get_signature] Get the signature of the object
pub trait VerifiySignatureTrait<'a>
where
    Self: 'a,
{
    /// Get the hashable from the object
    fn get_hashable(&'a self) -> anyhow::Result<HashableMessage<'a>>;

    /// Get the context data of the object according to the specifications
    fn get_context_data(&'a self) -> Vec<HashableMessage<'a>>;

    /// Get the Certificate Authority to the specifications
    fn get_certificate_authority(&self) -> anyhow::Result<String>;

    /// Get the signature of the object
    fn get_signature(&self) -> ByteArray;

    /// Get the context data of the object according to the context data
    fn get_context_hashable(&'a self) -> HashableMessage {
        if self.get_context_data().len() == 1 {
            return self.get_context_data()[0].clone();
        }
        HashableMessage::from(self.get_context_data())
    }

    /// Verfiy the signature according to the specifications of Verifier
    fn verifiy_signature(&'a self, keystore: &Keystore) -> anyhow::Result<bool> {
        let ca = &self
            .get_certificate_authority()
            .context("Error getting ca")?;
        let hashable_message = self
            .get_hashable()
            .context("Error getting the hashable message")?;
        verify_signature(
            keystore,
            ca.as_str(),
            &hashable_message,
            &self.get_context_hashable(),
            &self.get_signature(),
        )
        .context("Error verifying the signature")
    }
}

#[cfg(test)]
mod test {
    use crate::config::test::CONFIG_TEST;

    use super::*;

    #[test]
    fn test_create() {
        let dt = CONFIG_TEST.keystore().unwrap();
        //let dt = DirectTrustCertificate::new(, CertificateAuthority::Canton);
        assert!(dt
            .certificate(String::from(CertificateAuthority::Canton).as_str())
            .is_ok());
        assert!(dt
            .certificate(String::from(CertificateAuthority::SdmConfig).as_str())
            .is_ok());
        assert!(dt
            .certificate(String::from(CertificateAuthority::SdmTally).as_str())
            .is_ok());
        assert!(dt
            .certificate(String::from(CertificateAuthority::VotingServer).as_str())
            .is_ok());
        assert!(dt
            .certificate(String::from(CertificateAuthority::ControlComponent1).as_str())
            .is_ok());
        assert!(dt
            .certificate(String::from(CertificateAuthority::ControlComponent2).as_str())
            .is_ok());
        assert!(dt
            .certificate(String::from(CertificateAuthority::ControlComponent3).as_str())
            .is_ok());
        assert!(dt
            .certificate(String::from(CertificateAuthority::ControlComponent4).as_str())
            .is_ok());
    }
}
