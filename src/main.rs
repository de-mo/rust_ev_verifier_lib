pub mod crypto_primitives;
pub mod error;

use crypto_primitives::byte_array::ByteArray;
use num::bigint::{BigUint, ToBigUint};

fn main() {
    println!("Hello, world!");
    let ba: ByteArray = ByteArray::new();
    println!("Example byte array {:?}", ba);
    let n: BigUint = 123isize.to_biguint().unwrap();
    println!("Example bigint {}", n);
}
