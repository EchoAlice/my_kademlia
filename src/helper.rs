// NOTE: Silences `clippy` warning that originates from
// the `construct_uint` macro which we do not wish
// to address further
#![allow(clippy::assign_op_pattern)]

use crate::kbucket::MAX_BUCKETS;
use uint::*;

// TODO: pub type Identifier = U256;
pub type Identifier = [u8; 32];

construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}

pub fn xor_bucket_index(x: &Identifier, y: &Identifier) -> usize {
    let x = U256::from(x);
    let y = U256::from(y);
    let xor_distance = x ^ y;

    MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
}
