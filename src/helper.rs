// NOTE: Silences `clippy` warning that originates from
// the `construct_uint` macro which we do not wish
// to address further
#![allow(clippy::assign_op_pattern)]

use crate::node::{Node, Peer};
use crate::socket::SocketAddr;
use std::net;
use std::net::IpAddr;
use uint::*;
pub const PING_MESSAGE_SIZE: usize = 1024;

pub type Identifier = [u8; 32];
//  TODO:
// pub struct Identifier {
//     inner: [u8; 32],
// }
// impl From<u64> for Identifier {}

construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}

// Helper Functions for Tests
// -------------------------------------------------------------------------
pub async fn make_nodes(n: u8) -> (Node, Vec<Node>) {
    let local_node = make_node(0).await;
    let mut remote_nodes = Vec::new();

    for i in 1..n {
        remote_nodes.push(make_node(i).await);
    }

    (local_node, remote_nodes)
}

pub async fn make_node(index: u8) -> Node {
    let peer = make_peer(index);
    Node::new(peer).await
}

pub fn make_peer(index: u8) -> Peer {
    let ip = String::from("127.0.0.1").parse::<IpAddr>().unwrap();
    let port_start = 9000_u16;

    let mut id = [0_u8; 32];
    id[31] += index;
    let port = port_start + index as u16;

    let socket_addr = net::SocketAddr::new(ip, port);

    Peer {
        id,
        socket_addr: SocketAddr { addr: socket_addr },
    }
}
// -------------------------------------------------------------------------
