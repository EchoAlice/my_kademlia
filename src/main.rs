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
    println!("Let's build this thing");
}
