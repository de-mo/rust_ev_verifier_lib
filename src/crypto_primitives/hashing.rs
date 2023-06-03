//! Module to implement hashing functions
//!

use super::{byte_array::ByteArray, openssl_wrapper::sha3_256};
use chrono::NaiveDateTime;
use num_bigint::BigUint;

/// Enum to represent an element that is hashable
///
/// The specifiction of Swiss Post give the list of possible
/// elements that can be hashable.
///
/// To avoid copy of existing elements (and big memory growth), the HashableMessage
/// contains only references to data. HashableMessage has the lifetime of the reference.
///  
/// For simplification, the enum contains the possibility to reference String or &str and the possibility
/// to reference to BigUint or usize
///
/// Example:
/// ```rust
/// let r = HashableMessage::from("test string").recursive_hash();
/// let expected = ByteArray::base64_decode(&"m1a11iWW/Tcihy/IChyY51AO8UdZe48f5oRFh7RL+JQ=".to_string()).unwrap();
/// assert_eq!(r, e);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashableMessage<'a> {
    RByteArray(&'a ByteArray),
    RInt(&'a BigUint),
    RUSize(&'a usize),
    RString(&'a String),
    RStr(&'a str),
    String(String),
    Composite(Vec<HashableMessage<'a>>),
}

impl<'a> HashableMessage<'a> {
    fn to_hashable_byte_array(&self) -> ByteArray {
        match self {
            HashableMessage::RByteArray(b) => b.prepend_byte(0u8),
            HashableMessage::RInt(i) => ByteArray::from(*i).prepend_byte(1u8),
            HashableMessage::RUSize(i) => ByteArray::from(*i).prepend_byte(1u8),
            HashableMessage::RString(s) => ByteArray::from(*s).prepend_byte(2u8),
            HashableMessage::String(s) => ByteArray::from(s).prepend_byte(2u8),
            HashableMessage::RStr(s) => ByteArray::from(*s).prepend_byte(2u8),
            HashableMessage::Composite(c) => {
                let mut res = ByteArray::from_bytes(b"\x03");
                for e in c {
                    res.append(&e.recursive_hash());
                }
                res
            }
        }
    }

    /// Calculate the recursive hash according to the specification of Swiss Post
    pub fn recursive_hash(&self) -> ByteArray {
        let b = self.to_hashable_byte_array();
        sha3_256(&b)
    }
}

impl<'a> From<&'a ByteArray> for HashableMessage<'a> {
    fn from(value: &'a ByteArray) -> Self {
        HashableMessage::RByteArray(value)
    }
}

impl<'a> From<&'a BigUint> for HashableMessage<'a> {
    fn from(value: &'a BigUint) -> Self {
        HashableMessage::RInt(value)
    }
}

impl<'a> From<&'a usize> for HashableMessage<'a> {
    fn from(value: &'a usize) -> Self {
        HashableMessage::RUSize(value)
    }
}

impl<'a> From<&'a String> for HashableMessage<'a> {
    fn from(value: &'a String) -> Self {
        HashableMessage::RString(value)
    }
}

impl<'a> From<String> for HashableMessage<'a> {
    fn from(value: String) -> Self {
        HashableMessage::String(value.clone())
    }
}

impl<'a> From<&'a str> for HashableMessage<'a> {
    fn from(value: &'a str) -> Self {
        HashableMessage::RStr(value)
    }
}

impl<'a> From<&'a NaiveDateTime> for HashableMessage<'a> {
    fn from(value: &'a NaiveDateTime) -> Self {
        let s = value.format("%Y-%m-%dT%H:%M").to_string();
        println!("s = {:?}", s);
        HashableMessage::String(s)
    }
}

impl<'a> From<bool> for HashableMessage<'a> {
    fn from(value: bool) -> Self {
        match value {
            true => HashableMessage::String("true".to_string()),
            false => HashableMessage::String("false".to_string()),
        }
    }
}

impl<'a> From<Vec<HashableMessage<'a>>> for HashableMessage<'a> {
    fn from(value: Vec<HashableMessage<'a>>) -> Self {
        HashableMessage::Composite(value)
    }
}
impl<'a> From<&'a Vec<HashableMessage<'a>>> for HashableMessage<'a> {
    fn from(value: &'a Vec<HashableMessage<'a>>) -> Self {
        HashableMessage::Composite(value.clone())
    }
}

impl<'a> From<&'a Vec<String>> for HashableMessage<'a> {
    fn from(value: &'a Vec<String>) -> Self {
        let l: Vec<HashableMessage> = value.iter().map(|s| HashableMessage::from(s)).collect();
        HashableMessage::from(l)
    }
}

impl<'a> From<&'a Vec<&'a String>> for HashableMessage<'a> {
    fn from(value: &'a Vec<&'a String>) -> Self {
        let l: Vec<HashableMessage> = value.iter().map(|s| HashableMessage::from(*s)).collect();
        HashableMessage::from(l)
    }
}

impl<'a> From<&'a Vec<ByteArray>> for HashableMessage<'a> {
    fn from(value: &'a Vec<ByteArray>) -> Self {
        let l: Vec<HashableMessage> = value.iter().map(|b| HashableMessage::from(b)).collect();
        HashableMessage::from(l)
    }
}

impl<'a> From<&'a Vec<BigUint>> for HashableMessage<'a> {
    fn from(value: &'a Vec<BigUint>) -> Self {
        let l: Vec<HashableMessage> = value.iter().map(|n| HashableMessage::from(n)).collect();
        HashableMessage::from(l)
    }
}

impl<'a> From<&'a Vec<&'a BigUint>> for HashableMessage<'a> {
    fn from(value: &'a Vec<&'a BigUint>) -> Self {
        let l: Vec<HashableMessage> = value.iter().map(|n| HashableMessage::from(*n)).collect();
        HashableMessage::from(l)
    }
}

impl<'a> From<&'a Vec<usize>> for HashableMessage<'a> {
    fn from(value: &'a Vec<usize>) -> Self {
        let l: Vec<HashableMessage> = value.iter().map(|n| HashableMessage::from(n)).collect();
        HashableMessage::from(l)
    }
}

#[cfg(test)]
mod test {
    use super::super::num_bigint::Hexa;
    use super::*;
    use crate::crypto_primitives::byte_array::Decode;

    #[test]
    fn test_simple_byte_array() {
        let b = ByteArray::base64_decode(&"t+FRYortKmq/cViAnPTzx2LnFg84tNpWp4TZBFGQz+8yTnc4kmz75fS/jY2MMddj2gbICrsRhetPfHtXV/WVhJDP1H18GbtCFY2VVPe0a87VXE15/V8k1mE8McODmi3fipona8+/och3xWKE2rec1MKzKT0g6eXq8CrGCsyT7YdEIqUuyyOP7uWrat2DX9GgdT0Kj3jlN9K5W7edjcrsZCwenyO4KbXCeAvzhzffi7MA0BM0oNC9hkXL+nOmFg/+OTxIy7vKBg8P+OxtMb61zO7X8vC7CIAXFjvGDfRaDssbzSibBsu/6iGtCOGEfz9zeNVs7ZRkDW7w09N75p0AYw==".to_string()).unwrap();
        let r = HashableMessage::from(&b).recursive_hash();
        let e =
            ByteArray::base64_decode(&"0SHVZ9hTTmR+NRhanLPF/qPg3NmQbXyAzLYw9QVxYOg=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_biguint() {
        let i = BigUint::from_hexa_string(&"0xB7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF324E7738926CFBE5F4BF8D8D8C31D763DA06C80ABB1185EB4F7C7B5757F5958490CFD47D7C19BB42158D9554F7B46BCED55C4D79FD5F24D6613C31C3839A2DDF8A9A276BCFBFA1C877C56284DAB79CD4C2B3293D20E9E5EAF02AC60ACC93ED874422A52ECB238FEEE5AB6ADD835FD1A0753D0A8F78E537D2B95BB79D8DCAEC642C1E9F23B829B5C2780BF38737DF8BB300D01334A0D0BD8645CBFA73A6160FFE393C48CBBBCA060F0FF8EC6D31BEB5CCEED7F2F0BB088017163BC60DF45A0ECB1BCD289B06CBBFEA21AD08E1847F3F7378D56CED94640D6EF0D3D37BE69D0063".to_string()).unwrap();
        let r = HashableMessage::from(&i).recursive_hash();
        let e =
            ByteArray::base64_decode(&"YXHR0NvojiUMGz7RCTcO48ZQ1uqRtS64goB6XMFW01E=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_string() {
        let s = "test string".to_string();
        let r = HashableMessage::from(&s).recursive_hash();
        let e =
            ByteArray::base64_decode(&"m1a11iWW/Tcihy/IChyY51AO8UdZe48f5oRFh7RL+JQ=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_str() {
        let r = HashableMessage::from("test string").recursive_hash();
        let e =
            ByteArray::base64_decode(&"m1a11iWW/Tcihy/IChyY51AO8UdZe48f5oRFh7RL+JQ=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_biguint_list() {
        let inputs = [
            "0x41AFF17DA7F61150FCBC221E26D5BBEC1F540A3A3F13106FB45EB0E7C330C108AB338C525220A1D2D20EB77C642E7F360879A7B42BD2D191891F5A8CDBE7858407A7E7945A3518B0CC89A05BD3A61FD08235E0608F0AD678A99A385A668953A5591778CEBFCC8E3AF6F60DBA277320A58423FA436BEAACDEE2D5A2CDE86060BA8CF5BE70C4418E67B27FFEB96742FE6546C0ED533191B78BF88C8605D9ACF212016CB1735B1EC2ECC1491B73B82A5B348DB70A87FE0199899658CCD198CC53C7DD774D386A44867BB65EFF6704A6DD14AD462B13847B932FE4258C70F5FC20996FD9B2093EC0FD849070B5DDDDF741B8DFEFB972CFFE3A91E778CBEDE3A9CE1D",
            "0x35E854073500849CB2807B093D5F86176533B04DD81309D771A6461064E4A6E2B7F464D0502E9F2E2F5AD7AB4E225025E65A98CEEE2906C86158E7C432C4F50A149CD31A6C17CA1A000EC879B5CC0EF8E825EF8B83D4111D8AB59FCAB34694F112F5D3C2527F9121A50C95D975D3653972A9F17BFFBA26D542508EC57274202CCFF787EBC5E2E89F3EBEBFF17419B9338D47BF745901BE43D4A132FC503C9D07D7C3D3C35D303CD86C0F44B138E116CAA72B2DEFDA6D56BE841B980732EAE986710882143DAE385EE1832487F824A7AB404DFDFA903BEBDFC7682CE8D08F77B37E3B0AB99F40CAC2BA0EE8B6F64DE4BA3568A22359B114AE560656B8F59D0357",
            "0xA2A11C203F431AE713385CDF5F7346EF5A5E8B7B8CF971C947033978CF5F7263938D6B56754BAFBBCF8FC0A5CB2E0AF02D8433883326744E69247F0578A688A4225036F1D22D692ADA0C9515C3DE290797BE0E76FB04C9C17EF96E65F632329FC85C955C828A4DF5DF11962B3E24F32B7F87C47C0496F47ECF77C24C433740B4D3BCE077A7CEEE4EEE2E4B8D21E6DB21C05231EB1CB03D679D0D0B5D9E7BD205F9667FE6C18627E006191A987E5471E73D557B33FAD16D37C0B516D3948FE5B4690CE26059E6FC8B5853EE5AED99B6345206CCD5290CE0AC297163F57058A1ECE8718FBE8DAB9C2C5322D5726A16748F3F259B87FB00B1D54DDB063C7DBB8FF4"
        ];
        let bis: Vec<BigUint> = inputs
            .iter()
            .map(|e| BigUint::from_hexa_slice(e).unwrap())
            .collect();
        let r = HashableMessage::from(&bis).recursive_hash();
        let e =
            ByteArray::base64_decode(&"Qn1sWr2uZ87jwjeEoJa9zS6dc6S92oC0X83yxpyv2ZA=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_biguint_list_len1() {
        let inputs = [
            "0xA4D9B0B481FB03073E4B3EEE862FA2AA667AED37DD201FF41F786166C98D01AB3CEED0249FA1F12F23DEF203A98C53A294F5DE1A54A98EAA36F7232336FDFE89F28AD86789BCB67B5E41AFF9CE6EE5639A12B763D2A170E0B8208838079A622B11FC7DCDAC3DE178803E767028FEB607C2954834A8A53B400894E2CF7591D9E68CB987D2B5F05C5A799A38A513E53C451E6DF746C5C32FBAFE9AED6B8A1722AC15D40F1CA1DAC5F058618829514811F13516A18A4142D1B69830803A4910A89A5938491F75AFE9C07AC138CCB9B548814794A7B5A6E4F22CD2365FED5011A1E7DD26955958C8A9FCDEE31B9C6AABB6B50CC8E595144F4CCCAFFC74656DA135E3",
        ];
        let bis: Vec<BigUint> = inputs
            .iter()
            .map(|e| BigUint::from_hexa_slice(e).unwrap())
            .collect();
        let r = HashableMessage::from(&bis).recursive_hash();
        let e =
            ByteArray::base64_decode(&"+e9LVZg0L5uHLbnUv8pIVVm28y+QZMtfG1edAFx2oPM=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_mix_content() {
        let mut l: Vec<HashableMessage> = vec![];
        l.push(HashableMessage::from("common reference string"));
        let bi1 = BigUint::from_hexa_string(&"0xB7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF324E7738926CFBE5F4BF8D8D8C31D763DA06C80ABB1185EB4F7C7B5757F5958490CFD47D7C19BB42158D9554F7B46BCED55C4D79FD5F24D6613C31C3839A2DDF8A9A276BCFBFA1C877C56284DAB79CD4C2B3293D20E9E5EAF02AC60ACC93ED874422A52ECB238FEEE5AB6ADD835FD1A0753D0A8F78E537D2B95BB79D8DCAEC642C1E9F23B829B5C2780BF38737DF8BB300D01334A0D0BD8645CBFA73A6160FFE393C48CBBBCA060F0FF8EC6D31BEB5CCEED7F2F0BB088017163BC60DF45A0ECB1BCD289B06CBBFEA21AD08E1847F3F7378D56CED94640D6EF0D3D37BE69D0063".to_string()).unwrap();
        l.push(HashableMessage::from(&bi1));
        let bi2 = BigUint::from_hexa_string(&"0x5BF0A8B1457695355FB8AC404E7A79E3B1738B079C5A6D2B53C26C8228C867F799273B9C49367DF2FA5FC6C6C618EBB1ED0364055D88C2F5A7BE3DABABFACAC24867EA3EBE0CDDA10AC6CAAA7BDA35E76AAE26BCFEAF926B309E18E1C1CD16EFC54D13B5E7DFD0E43BE2B1426D5BCE6A6159949E9074F2F5781563056649F6C3A21152976591C7F772D5B56EC1AFE8D03A9E8547BC729BE95CADDBCEC6E57632160F4F91DC14DAE13C05F9C39BEFC5D98068099A50685EC322E5FD39D30B07FF1C9E2465DDE5030787FC763698DF5AE6776BF9785D84400B8B1DE306FA2D07658DE6944D8365DFF510D68470C23F9FB9BC6AB676CA3206B77869E9BDF34E8031".to_string()).unwrap();
        l.push(HashableMessage::from(&bi2));
        let ba = ByteArray::base64_decode(&"YcOpYm5zaXRwcSBi".to_string()).unwrap();
        l.push(HashableMessage::from(&ba));
        let r = HashableMessage::Composite(l).recursive_hash();
        let e =
            ByteArray::base64_decode(&"rHGUCWqWKTj9KBY3GgSeNEXZfraTDK+ZGIhlSxpVs5c=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_mixed_content_nested() {
        let mut nl: Vec<HashableMessage> = vec![];
        let bu1 = BigUint::from_hexa_string(&"0x4".to_string()).unwrap();
        nl.push(HashableMessage::from(&bu1));
        let bu2= BigUint::from_hexa_string(&"0x3896D05A527747E840CEB0A10454DE39955529297AC4CB21010E9287A21F826FA7221215E1C7EE8362223DF51215A7F4CD14F158980154EE0794B599639A6FBC171A97F376A4DD95945C476F0DC6836FCEA68C9B28F901CE7F30DC03F406947E6245BF741650F5164BFC24F4B23948A5D6642C36D61016E63E943DB9717335EEB04373BFAE10BB4FB20EA9FD1BE48CA9A02B8E8C6639AD8E43D714ED16D4764D258E9A70BABD5497C09E148052C1C6A965F18F71F7B03385178B4991AA790611FA3B98E9C2F1EE1E0369F496A1D6928D718650513439D01898AAB87BC968F76D9DB8089809142A0C79A84C689D02314CEDE64F4C9615B79D49D2BE641BE8D4AB".to_string()).unwrap();
        nl.push(HashableMessage::from(&bu2));
        let mut l: Vec<HashableMessage> = vec![];
        l.push(HashableMessage::from("common reference string"));
        let bu3=BigUint::from_hexa_string(&"0xB7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF324E7738926CFBE5F4BF8D8D8C31D763DA06C80ABB1185EB4F7C7B5757F5958490CFD47D7C19BB42158D9554F7B46BCED55C4D79FD5F24D6613C31C3839A2DDF8A9A276BCFBFA1C877C56284DAB79CD4C2B3293D20E9E5EAF02AC60ACC93ED874422A52ECB238FEEE5AB6ADD835FD1A0753D0A8F78E537D2B95BB79D8DCAEC642C1E9F23B829B5C2780BF38737DF8BB300D01334A0D0BD8645CBFA73A6160FFE393C48CBBBCA060F0FF8EC6D31BEB5CCEED7F2F0BB088017163BC60DF45A0ECB1BCD289B06CBBFEA21AD08E1847F3F7378D56CED94640D6EF0D3D37BE69D0063".to_string()).unwrap();
        l.push(HashableMessage::from(&bu3));
        let bu4 = BigUint::from_hexa_string(&"0x5BF0A8B1457695355FB8AC404E7A79E3B1738B079C5A6D2B53C26C8228C867F799273B9C49367DF2FA5FC6C6C618EBB1ED0364055D88C2F5A7BE3DABABFACAC24867EA3EBE0CDDA10AC6CAAA7BDA35E76AAE26BCFEAF926B309E18E1C1CD16EFC54D13B5E7DFD0E43BE2B1426D5BCE6A6159949E9074F2F5781563056649F6C3A21152976591C7F772D5B56EC1AFE8D03A9E8547BC729BE95CADDBCEC6E57632160F4F91DC14DAE13C05F9C39BEFC5D98068099A50685EC322E5FD39D30B07FF1C9E2465DDE5030787FC763698DF5AE6776BF9785D84400B8B1DE306FA2D07658DE6944D8365DFF510D68470C23F9FB9BC6AB676CA3206B77869E9BDF34E8031".to_string()).unwrap();
        l.push(HashableMessage::from(&bu4));
        let ba = ByteArray::base64_decode(&"YcOpYm5zaXRwcSBi".to_string()).unwrap();
        l.push(HashableMessage::from(&ba));
        l.push(HashableMessage::Composite(nl));
        let r = HashableMessage::Composite(l).recursive_hash();
        let e =
            ByteArray::base64_decode(&"HYq9bWhqsm+/Sh8omWJGg2om5sQ2zosPIEhaIQ2m9GE=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }

    #[test]
    fn test_mixed_content_nested2() {
        let mut nl: Vec<HashableMessage> = vec![];
        let n = 4 as usize;
        nl.push(HashableMessage::from(&n));
        let bu2= BigUint::from_hexa_string(&"0x3896D05A527747E840CEB0A10454DE39955529297AC4CB21010E9287A21F826FA7221215E1C7EE8362223DF51215A7F4CD14F158980154EE0794B599639A6FBC171A97F376A4DD95945C476F0DC6836FCEA68C9B28F901CE7F30DC03F406947E6245BF741650F5164BFC24F4B23948A5D6642C36D61016E63E943DB9717335EEB04373BFAE10BB4FB20EA9FD1BE48CA9A02B8E8C6639AD8E43D714ED16D4764D258E9A70BABD5497C09E148052C1C6A965F18F71F7B03385178B4991AA790611FA3B98E9C2F1EE1E0369F496A1D6928D718650513439D01898AAB87BC968F76D9DB8089809142A0C79A84C689D02314CEDE64F4C9615B79D49D2BE641BE8D4AB".to_string()).unwrap();
        nl.push(HashableMessage::from(&bu2));
        let mut l: Vec<HashableMessage> = vec![];
        l.push(HashableMessage::from("common reference string"));
        let bu3=BigUint::from_hexa_string(&"0xB7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF324E7738926CFBE5F4BF8D8D8C31D763DA06C80ABB1185EB4F7C7B5757F5958490CFD47D7C19BB42158D9554F7B46BCED55C4D79FD5F24D6613C31C3839A2DDF8A9A276BCFBFA1C877C56284DAB79CD4C2B3293D20E9E5EAF02AC60ACC93ED874422A52ECB238FEEE5AB6ADD835FD1A0753D0A8F78E537D2B95BB79D8DCAEC642C1E9F23B829B5C2780BF38737DF8BB300D01334A0D0BD8645CBFA73A6160FFE393C48CBBBCA060F0FF8EC6D31BEB5CCEED7F2F0BB088017163BC60DF45A0ECB1BCD289B06CBBFEA21AD08E1847F3F7378D56CED94640D6EF0D3D37BE69D0063".to_string()).unwrap();
        l.push(HashableMessage::from(&bu3));
        let bu4 = BigUint::from_hexa_string(&"0x5BF0A8B1457695355FB8AC404E7A79E3B1738B079C5A6D2B53C26C8228C867F799273B9C49367DF2FA5FC6C6C618EBB1ED0364055D88C2F5A7BE3DABABFACAC24867EA3EBE0CDDA10AC6CAAA7BDA35E76AAE26BCFEAF926B309E18E1C1CD16EFC54D13B5E7DFD0E43BE2B1426D5BCE6A6159949E9074F2F5781563056649F6C3A21152976591C7F772D5B56EC1AFE8D03A9E8547BC729BE95CADDBCEC6E57632160F4F91DC14DAE13C05F9C39BEFC5D98068099A50685EC322E5FD39D30B07FF1C9E2465DDE5030787FC763698DF5AE6776BF9785D84400B8B1DE306FA2D07658DE6944D8365DFF510D68470C23F9FB9BC6AB676CA3206B77869E9BDF34E8031".to_string()).unwrap();
        l.push(HashableMessage::from(&bu4));
        let ba = ByteArray::base64_decode(&"YcOpYm5zaXRwcSBi".to_string()).unwrap();
        l.push(HashableMessage::from(&ba));
        l.push(HashableMessage::Composite(nl));
        let r = HashableMessage::Composite(l).recursive_hash();
        let e =
            ByteArray::base64_decode(&"HYq9bWhqsm+/Sh8omWJGg2om5sQ2zosPIEhaIQ2m9GE=".to_string())
                .unwrap();
        assert_eq!(r, e);
    }
}
