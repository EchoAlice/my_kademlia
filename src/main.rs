#![allow(unused)]

use std::net::Ipv4Addr;

use crate::{helper::Node, kbucket::KbucketTable};
use sha2::{Digest, Sha256};

pub mod helper;
pub mod kbucket;

fn main() {
    /// Bootstrapping protocol -
    /// "To join the network, a node u ust have a contact (bootstrap node) to an already
    /// participating node w. u inserts w into the appropriate k-bucket. u then performs
    /// a node lookup for its own node ID.  Finally, u refreshes all k-buckets further away
    /// than its closest neighbor."
    ///
    /// TODO:  Implement bootstrapping
    ///
    ///
    // Should these nodes have different IP addresses?  Stupid question- but I'm asking anyways   :P
    let listen_addr = String::from("127.0.0.1").parse::<Ipv4Addr>().unwrap();
    let port_start = 9000_u16;

    // Dummy Nodes
    let local_node = Node {
        ip_address: listen_addr,
        udp_port: port_start,
        node_id: [0_u8; 32],
    };
    let first_node_to_add = Node {
        ip_address: listen_addr,
        udp_port: port_start + 1,
        node_id: [1_u8; 32],
    };
    let second_node_to_add = Node {
        ip_address: listen_addr,
        udp_port: port_start + 2,
        node_id: [2_u8; 32],
    };
    let third_node_to_add = Node {
        ip_address: listen_addr,
        udp_port: port_start + 3,
        node_id: [3_u8; 32],
    };

    let mut local_nodes_rt = KbucketTable::new(local_node.node_id);

    // Testing node is added only once
    local_nodes_rt.add_node(first_node_to_add);
    println!("\n");
    local_nodes_rt.add_node(second_node_to_add);
    println!("\n");
    local_nodes_rt.add_node(second_node_to_add);
    println!("\n");
    local_nodes_rt.add_node(first_node_to_add);
    println!("\n");

    // Testing find_node()
    let result = local_nodes_rt.find_node(first_node_to_add.node_id);
    println!("\n");
    let result = local_nodes_rt.find_node(third_node_to_add.node_id);
    println!("\n");

    // Verify Table at a glance
    // println!("Node's routing table: {:?}", local_nodes_rt.buckets);
}
