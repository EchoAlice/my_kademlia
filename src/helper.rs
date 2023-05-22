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

// For testing. TODO: Create mod
pub fn create_dummy_nodes() -> Vec<Node> {
    // Should these nodes have different IP addresses?  Stupid question- but I'm asking anyways   :P
    let listen_addr = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
    let port_start = 9000_u16;
    let mut our_nodes = Vec::new();

    let local_node = Node {
        ip_address: listen_addr,
        udp_port: port_start,
        node_id: [0_u8; 32],
    };
    our_nodes.push(local_node);
    let first_node = Node {
        ip_address: listen_addr,
        udp_port: port_start + 1,
        node_id: [1_u8; 32],
    };
    our_nodes.push(first_node);
    let second_node = Node {
        ip_address: listen_addr,
        udp_port: port_start + 2,
        node_id: [2_u8; 32],
    };
    our_nodes.push(second_node);
    let third_node = Node {
        ip_address: listen_addr,
        udp_port: port_start + 3,
        node_id: [3_u8; 32],
    };
    our_nodes.push(third_node);

    our_nodes
}

// Create a dummy table to use for tests
pub fn create_test_table(dummy_nodes: Vec<Node>) -> KbucketTable {
    let listen_addr = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
    let port_start = 9000_u16;

    KbucketTable::new(dummy_nodes[0].node_id)
}
