use crate::helper::Identifier;
use crate::kbucket::TableRecord;
use std::collections::HashMap;
use std::convert::From;

// TODO: Alias u8 = session

#[derive(Debug, Clone)]
pub struct Message {
    pub session: u8,
    pub body: MessageBody,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageBody {
    Ping(Identifier),          // b"01"
    Pong(Identifier),          // b"02"
    FindNode([Identifier; 2]), // b"03"
}

impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self.body {
            MessageBody::Ping(requester_id) => {
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
