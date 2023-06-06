// NOTE: Silences `clippy` warning that originates from
// the `construct_uint` macro which we do not wish
// to address further
#![allow(clippy::assign_op_pattern)]

use tokio::sync::mpsc;
use uint::*;

pub type Identifier = [u8; 32];

construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}

/*
pub struct Packet {}
pub struct Handler {
    // Channel to respond to send requests
    handler_recv: mpsc::Receiver<Packet>,
}
*/
