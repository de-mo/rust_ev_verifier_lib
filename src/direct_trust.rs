use super::config::Config;

/// List of valide Certificate authorities
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl From<&CertificateAuthority> for String {
    fn from(value: &CertificateAuthority) -> Self {
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
    use super::*;
    /*
    use std::path::PathBuf;

    fn get_location() -> PathBuf {
        Path::new("./").join("test_data").join("direct-trust")
    }

    #[test]
    fn test_create() {
        let dt = DirectTrust::new(&get_location().join(Path(KEYSTORE_FILE_NAME))).unwrap();
        //let dt = DirectTrustCertificate::new(, &CertificateAuthority::Canton);
        assert!(dt.certificate(&CertificateAuthority::Canton).is_ok());
        assert!(dt.certificate(&CertificateAuthority::SdmConfig).is_ok());
        assert!(dt.certificate(&CertificateAuthority::SdmTally).is_ok());
        assert!(dt.certificate(&CertificateAuthority::VotingServer).is_ok());
        assert!(dt
            .certificate(&CertificateAuthority::ControlComponent1)
            .is_ok());
        assert!(dt
            .certificate(&CertificateAuthority::ControlComponent2)
            .is_ok());
        assert!(dt
            .certificate(&CertificateAuthority::ControlComponent3)
            .is_ok());
        assert!(dt
            .certificate(&CertificateAuthority::ControlComponent4)
            .is_ok());
        let dt_err = DirectTrust::new(Path::new("./toto"));
        assert!(dt_err.is_err());
    } */
}
