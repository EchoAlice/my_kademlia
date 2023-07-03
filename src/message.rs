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
    Ping(u8), // b"01"
    Pong(u8), // b"02"
    // TODO:  Leverage lifetime to pass a reference to an Identifier to FindNode
    FindNode(Identifier), // b"03"
}

impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self.body {
            MessageBody::Ping(body) => {
                unimplemented!()
            }
            MessageBody::Pong(body) => {
                unimplemented!()
            }
            MessageBody::FindNode(id) => {
                out.extend_from_slice(b"03");
                out.push(self.session);
                out.extend_from_slice(&id);
            }
        }
        out
    }
}
