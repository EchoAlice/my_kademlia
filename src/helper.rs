#![allow(unused)]
use uint::*;

pub type Identifier = [u8; 32];

construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub ip_address: &'static str,
    pub udp_port: &'static str,
    pub node_id: Identifier,
}