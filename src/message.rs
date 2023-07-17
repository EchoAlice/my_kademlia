use crate::helper::Identifier;
use crate::node::Peer;

use std::collections::HashMap;
use std::convert::From;
use tokio::sync::oneshot;

// TODO: Alias u8 = session

#[derive(Debug)]
pub struct Message {
    pub target: Peer,
    pub inner: MessageInner,
}

// Get rid of this?
#[derive(Debug)]
pub struct MessageInner {
    pub session: u8,
    pub body: MessageBody,
}

#[derive(Debug)]
pub enum MessageBody {
    Ping(Identifier, Option<oneshot::Sender<bool>>), // b"01"
    Pong(Identifier),                                // b"02"
    FindNode([Identifier; 2]),                       // b"03"
}

impl MessageInner {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self.body {
            MessageBody::Ping(requester_id, _) => {
                out.extend_from_slice(b"01");
                out.push(self.session);
                out.extend_from_slice(&requester_id);
            }
            MessageBody::Pong(requester_id) => {
                out.extend_from_slice(b"02");
                out.push(self.session);
                out.extend_from_slice(&requester_id);
            }
            MessageBody::FindNode([requester_id, id_to_find]) => {
                out.extend_from_slice(b"03");
                out.push(self.session);
                out.extend_from_slice(&requester_id);
                out.extend_from_slice(&id_to_find);
            }
            _ => {}
        }
        out
    }
}

pub fn construct_msg(datagram: [u8; 1024], target: Peer) -> Message {
    let requester_id: [u8; 32] = datagram[3..35].try_into().expect("Invalid slice length");

    match &datagram[0..2] {
        b"01" => Message {
            target,
            inner: MessageInner {
                session: datagram[2],
                body: MessageBody::Ping(requester_id, None),
            },
        },
        b"02" => Message {
            target,
            inner: MessageInner {
                session: datagram[2],
                body: MessageBody::Pong(requester_id),
            },
        },
        b"03" => Message {
            target,
            inner: MessageInner {
                session: datagram[2],
                body: MessageBody::FindNode([
                    requester_id,
                    datagram[35..67].try_into().expect("Invalid slice length"),
                ]),
            },
        },
        _ => {
            panic!("Message wasn't legitimate");
        }
    }
}
