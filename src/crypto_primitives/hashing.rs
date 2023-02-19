use super::byte_array::ByteArray;

pub trait RecursiveHash {
    fn to_hashable_byte_array(&self) -> ByteArray;
    //fn recursive_hash(&self) -> ByteArray;
}

impl RecursiveHash for ByteArray {
    fn to_hashable_byte_array(&self) -> ByteArray {
        self.prepend_byte(0u8)
    }
}

impl RecursiveHash for String {
    fn to_hashable_byte_array(&self) -> ByteArray {
        todo!()
    }
}
