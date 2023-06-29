use crate::helper::Identifier;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Ping([u8; 1024]),
    Pong([u8; 1024]),
    FindNode([u8; 1024]),
    // FoundNode,
}

pub fn create_message(mtype: &[u8; 4], local_id: &Identifier, session_number: u8) -> [u8; 1024] {
    let mut message = [0u8; 1024];
    message[0..4].copy_from_slice(mtype);
    message[4] = session_number;
    message[5..37].copy_from_slice(local_id);
    message
}

// TODO: Consolidate logic from ping and find_node here.
pub fn request_message() {}
