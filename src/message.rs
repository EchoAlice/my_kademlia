use crate::helper::Identifier;
use crate::kbucket::TableRecord;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Ping([u8; 1024]),     // b"01"
    Pong([u8; 1024]),     // b"02"
    FindNode([u8; 1024]), // b"03"
}

pub fn create_message(
    mtype: &[u8; 2],
    local_id: &Identifier,
    session_number: &u8,
    peers: Option<&HashMap<[u8; 32], TableRecord>>,
) -> [u8; 1024] {
    let mut message = [0u8; 1024];
    message[0..2].copy_from_slice(mtype);
    message[2] = *session_number;
    message[3..35].copy_from_slice(local_id);

    // TODO: Use Some() None logic here

    if let Some(peers) = peers {
    } else {
    }

    if &message[0..2] == b"03" {
        println!("TODO: Place peers in message: {:?}", peers);
        /*
        ChatGPT says...

        for (i, item) in data.iter().enumerate() {
            let start = 35 + i * Identifier.len();
            let end = start + item.len();
            message[start..end].copy_from_slice(item);
        }
         */
    }

    message
}

// TODO: Consolidate logic from ping and find_node.  This may need to be placed
pub fn request_message() /*-> u8*/ {}
