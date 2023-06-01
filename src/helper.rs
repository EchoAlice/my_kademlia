// NOTE: Silences `clippy` warning that originates from
// the `construct_uint` macro which we do not wish
// to address further
#![allow(clippy::assign_op_pattern)]

use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::sync::mpsc;
use uint::*;

pub type Identifier = [u8; 32];

construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub node_id: Identifier,
    pub ip_address: Ipv4Addr,
    pub udp_port: u16,
    pub socket: SocketAddrV4,
}

// Spin up a server that has a SocketAddress.  We need an address to send any UDP
// message to...  See https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html
// https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html
