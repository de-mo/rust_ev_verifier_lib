use super::super::byte_array::ByteArray;
use openssl::pkey::{PKey, Public};

pub fn verify(pkey: &PKey<Public>, bytes: &ByteArray, signature: &ByteArray) -> bool {
    todo!()
}
