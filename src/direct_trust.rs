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

#[cfg(test)]
mod test {
    use crate::config::test::CONFIG_TEST;

    use super::*;

    #[test]
    fn test_create() {
        let dt = CONFIG_TEST.keystore().unwrap();
        //let dt = DirectTrustCertificate::new(, CertificateAuthority::Canton);
        assert!(dt.certificate(String::from(CertificateAuthority::Canton).as_str()).is_ok());
        assert!(dt.certificate(String::from(CertificateAuthority::SdmConfig).as_str()).is_ok());
        assert!(dt.certificate(String::from(CertificateAuthority::SdmTally).as_str()).is_ok());
        assert!(dt.certificate(String::from(CertificateAuthority::VotingServer).as_str()).is_ok());
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
