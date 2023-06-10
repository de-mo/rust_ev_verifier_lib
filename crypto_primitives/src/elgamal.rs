//! Module to implement El Gamal functions

use super::{
    byte_array::ByteArray,
    num_bigint::Constants,
    number_theory::{
        is_probable_prime, is_quadratic_residue, is_small_prime, miller_rabin_test, SMALL_PRIMES,
    },
    openssl_wrapper::shake128,
};
use num_bigint::BigUint;
use thiserror::Error;

// Get small prime group members according to the specifications
pub fn get_small_prime_group_members(
    p: &BigUint,
    desired_number: usize,
) -> Result<Vec<usize>, ElgamalError> {
    let mut current = 5usize;
    let mut res = vec![];
    while res.len() < desired_number && BigUint::from(current) < *p && current < usize::pow(2, 31) {
        let is_prime = is_small_prime(current).unwrap();
        if is_prime && is_quadratic_residue(&BigUint::from(current), p) {
            res.push(current);
        }
        current += 2;
    }
    if res.len() != desired_number {
        return Err(ElgamalError::TooFewSmallPrimeNumbers {
            expected: desired_number,
            found: res.len(),
        });
    }
    Ok(res)
}

// GetEncryptionParameters according to the specification
pub fn get_encryption_parameters(
    seed: &String,
) -> Result<(BigUint, BigUint, BigUint), ElgamalError> {
    let q_b_1 = shake128(&ByteArray::from(seed));
    let q_b = q_b_1.prepend_byte(2u8);
    let q_prime: BigUint = q_b.into_biguint() >> 3;
    let q = &q_prime - (&q_prime % 6u8) + 5u8;
    let rs: Vec<BigUint> = SMALL_PRIMES.iter().map(|sp| &q % sp).collect();
    let jump: usize = 6;
    let mut delta: usize = 0;
    loop {
        delta += jump;
        let mut i: usize = 0;
        while i < rs.len() {
            if (rs[i].clone() + delta) % SMALL_PRIMES[i] == BigUint::zero()
                || ((rs[i].clone() + delta) * 2u8 + 1u8) % SMALL_PRIMES[i] == BigUint::zero()
            {
                delta += jump;
                i = 0
            } else {
                i += 1
            }
        }
        if is_probable_prime(&(&q + delta)) && is_probable_prime(&((&q + delta) * 2u8 + 1u8)) {
            break;
        }
    }
    let q_final = &q + delta;
    let p = q_final.clone() * 2u8 + 1u8;
    let mut g: u8 = 3;
    if is_quadratic_residue(&BigUint::two(), &p) {
        g = 2;
    }
    match miller_rabin_test(&q_final, 64) {
        true => match miller_rabin_test(&p, 64) {
            true => Ok((p, q_final, BigUint::from(g))),
            false => Err(ElgamalError::NotPrime("p".to_string(), p)),
        },
        false => Err(ElgamalError::NotPrime("q".to_string(), q)),
    }
}

#[derive(Error, Debug)]
pub enum ElgamalError {
    #[error("To few number of small primes found. Expcted: {expected}, found: {found}")]
    TooFewSmallPrimeNumbers { expected: usize, found: usize },
    #[error("Number {0} with value {1} is not prime")]
    NotPrime(String, BigUint),
}

#[cfg(test)]
mod test {

    use super::super::num_bigint::Hexa;
    use super::*;

    #[test]
    fn test_get_small_prime_group_members() {
        let p = BigUint::from_hexa_string("0xCE9E0307D2AE75BDBEEC3E0A6E71A279417B56C955C602FFFD067586BACFDAC3BCC49A49EB4D126F5E9255E57C14F3E09492B6496EC8AC1366FC4BB7F678573FA2767E6547FA727FC0E631AA6F155195C035AF7273F31DFAE1166D1805C8522E95F9AF9CE33239BF3B68111141C20026673A6C8B9AD5FA8372ED716799FE05C0BB6EAF9FCA1590BD9644DBEFAA77BA01FD1C0D4F2D53BAAE965B1786EC55961A8E2D3E4FE8505914A408D50E6B99B71CDA78D8F9AF1A662512F8C4C3A9E72AC72D40AE5D4A0E6571135CBBAAE08C7A2AA0892F664549FA7EEC81BA912743F3E584AC2B2092243C4A17EC98DF079D8EECB8B885E6BBAFA452AAFA8CB8C08024EFF28DE4AF4AC710DCD3D66FD88212101BCB412BCA775F94A2DCE18B1A6452D4CF818B6D099D4505E0040C57AE1F3E84F2F8E07A69C0024C05ACE05666A6B63B0695904478487E78CD0704C14461F24636D7A3F267A654EEDCF8789C7F627C72B4CBD54EED6531C0E54E325D6F09CB648AE9185A7BDA6553E40B125C78E5EAA867").unwrap();
        assert_eq!(
            get_small_prime_group_members(&p, 5).unwrap(),
            vec![5, 17, 19, 37, 41]
        );
    }

    #[test]
    #[ignore]
    fn test_get_encryption_parameters() {
        let eg_res = get_encryption_parameters(&"31".to_string());
        assert!(eg_res.is_ok());
        let (p, q, g) = eg_res.unwrap();
        let p_exp = BigUint::from_hexa_string("0xBFF67CCCAE0F61B38BA70AD736CFA8EA284B5D6CAEBF2FED2FC88D0ADFF9E2B220BFD9CCDA59BD3BD52B12CDFCCF41AA3D9BF81F95A7D59452690BF45F7993BE760ABBCA3E29705D473A66638DCD6EA78663C0DB91E3E0AB1DFE1AFF25181D4D2C3BA059F9131D95D37F431233EA2276E052C960DCB130F9DFFDC0BE977C9947E7AE05EA516AA81B2528FEF03625ACFCF495C3AB5D5F176E06F1382AE96A470321092C0C1C02A196AB4DA20D3605B4E72A5CFD16CF9381C83513EBD18A8A4A21BF95B864EDA4C0214583E99A3180F7A561F19D451BC4354E7A284DC7EB0C5A05DC58856C6DC8CF3A57B42D866D85F453D1BD8CC61117FB606A40AF0A0EF76D603C7A307C0B8854355D5836774C6BB12238E09806782A487BB9888AE1DB54DECA3FEC374D30CC9A722D3052585069D212B62FD6758710337CA17411E82FF7E7E7B754F4C9F3A1C49AA15E0D0A0E9B05A2EA880216D052B780E68168CA336309D3C1802A278AFCF1C0F8FA3381C145DA0864892221B960ECD6D46165E057B55EEB").unwrap();
        let q_exp = BigUint::from_hexa_string("0x5FFB3E665707B0D9C5D3856B9B67D4751425AEB6575F97F697E446856FFCF159105FECE66D2CDE9DEA958966FE67A0D51ECDFC0FCAD3EACA293485FA2FBCC9DF3B055DE51F14B82EA39D3331C6E6B753C331E06DC8F1F0558EFF0D7F928C0EA6961DD02CFC898ECAE9BFA18919F5113B702964B06E58987CEFFEE05F4BBE4CA3F3D702F528B5540D92947F781B12D67E7A4AE1D5AEAF8BB703789C1574B52381908496060E0150CB55A6D1069B02DA73952E7E8B67C9C0E41A89F5E8C5452510DFCADC3276D26010A2C1F4CD18C07BD2B0F8CEA28DE21AA73D1426E3F5862D02EE2C42B636E4679D2BDA16C336C2FA29E8DEC663088BFDB035205785077BB6B01E3D183E05C42A1AAEAC1B3BA635D8911C704C033C15243DDCC44570EDAA6F651FF61BA698664D391698292C2834E9095B17EB3AC38819BE50BA08F417FBF3F3DBAA7A64F9D0E24D50AF0685074D82D17544010B68295BC07340B46519B184E9E0C01513C57E78E07C7D19C0E0A2ED0432449110DCB0766B6A30B2F02BDAAF75").unwrap();
        let g_exp = BigUint::from(3u8);
        assert_eq!(p, p_exp);
        assert_eq!(q, q_exp);
        assert_eq!(g, g_exp);
    }
}
