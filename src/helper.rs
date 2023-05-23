// NOTE: Silences `clippy` warning that originates from
// the `construct_uint` macro which we do not wish
// to address further
#![allow(clippy::assign_op_pattern)]

use std::net::Ipv4Addr;
use uint::*;

pub type Identifier = [u8; 32];

construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub ip_address: Ipv4Addr,
    pub udp_port: u16,
    pub node_id: Identifier,
}
