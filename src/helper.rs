#![allow(clippy::assign_op_pattern)]

use crate::kbucket::KbucketTable;
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

pub mod testing {
    use super::*;

    // Add parameter for number of nodes
    pub fn mk_nodes() -> Vec<Node> {
        // Should these nodes have different IP addresses?
        let listen_addr = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
        let port_start = 9000_u16;

        let our_nodes: Vec<Node> = (0..5)
            .into_iter()
            .map(|i| mk_node(&listen_addr, port_start, i))
            .collect();

        our_nodes
    }

    fn mk_node(listen_addr: &Ipv4Addr, port_start: u16, index: usize) -> Node {
        Node {
            ip_address: listen_addr.clone(),
            udp_port: port_start + index as u16,
            node_id: [index as u8; 32],
        }
    }

    // TODO:
    // pub fn populate_table {}
}
