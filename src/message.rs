use crate::helper::Identifier;
use crate::kbucket::TableRecord;
use std::collections::HashMap;
use std::convert::From;

// TODO: Alias u8 = session

#[derive(Debug)]
pub struct Message {
    pub session: u8,
    pub body: MessageBody,
}

// TODO: Properly implement From trait.
impl From<Message> for Vec<u8> {
    fn from(item: Message) -> Self {
        let mut out = Vec::new();
        match item.body {
            MessageBody::Ping(body) => {
                unimplemented!()
            }
            MessageBody::Pong(body) => {
                unimplemented!()
            }
            MessageBody::FindNode(id) => {
                for byte in b"03" {
                    out.push(*byte);
                }
                out.push(item.session);
                for byte in id {
                    out.push(byte);
                }
            }
        };
        out
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageBody {
    Ping(u8),             // b"01"
    Pong(u8),             // b"02"
    FindNode(Identifier), // b"03"
}

// TODO: Consolidate logic from ping and find_node.  This may need to be placed
// pub fn request_message() /*-> u8*/ {}
