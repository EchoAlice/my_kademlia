// NOTE: Silences `clippy` warning that originates from
// the `construct_uint` macro which we do not wish
// to address further
#![allow(clippy::assign_op_pattern)]

// use tokio::sync::mpsc;
use bytes::{BufMut, BytesMut};
use fastrlp::{encode_list, DecodeError, Encodable};
use std::net;
use uint::*;
pub const PING_MESSAGE_SIZE: usize = 1024;
pub type Identifier = [u8; 32];

construct_uint! {
    /// 256-bit unsigned integer (little endian).
    pub struct U256(4);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SocketAddr {
    pub addr: net::SocketAddr,
}

impl fastrlp::Encodable for SocketAddr {
    fn encode(&self, out: &mut dyn BufMut) {
        match self.addr {
            net::SocketAddr::V4(socket) => {
                let ip = socket.ip().octets();
                let port = socket.port();
                let mut enc: [&dyn Encodable; 3] = [b""; 3];

                enc[0] = &0_u8;
                enc[1] = &ip;
                enc[2] = &port;

                encode_list::<dyn Encodable, _>(&enc, out);
            }
            net::SocketAddr::V6(socket) => {
                let ip = socket.ip().octets();
                let port = socket.port();
                let mut enc: [&dyn Encodable; 3] = [b""; 3];

                enc[0] = &0_u8;
                enc[1] = &ip;
                enc[2] = &port;

                encode_list::<dyn Encodable, _>(&enc, out);
            }
        };
    }
}

pub fn encoded<T: fastrlp::Encodable>(t: &T) -> BytesMut {
    let mut out = BytesMut::new();
    t.encode(&mut out);

    println!("Out (encode): {:?}", out);
    out
}

impl fastrlp::Decodable for SocketAddr {
    // We know the size of the datagram before it's called to be decoded
    fn decode(data: &mut &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 7 {
            // TODO: Return error
            panic!()
        }

        let mut stream = fastrlp::Rlp::new(data)?;
        let typ = stream.get_next::<u8>()?.unwrap();

        let addr = match typ {
            0 => {
                let ip = stream.get_next::<[u8; 4]>()?.unwrap();
                let port = stream.get_next::<u16>()?.unwrap();
                net::SocketAddr::new(ip.into(), port)
            }
            1 => {
                if data.len() < 19 {
                    // TODO: Return error
                    panic!()
                }
                let ip = stream.get_next::<[u8; 16]>()?.unwrap();
                let port = stream.get_next::<u16>()?.unwrap();
                net::SocketAddr::new(ip.into(), port)
            }
            _ => panic!(),
        };
        Ok(Self { addr })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use fastrlp::Decodable;
    use std::net::IpAddr;
    #[test]
    fn socket_addr_serialization() {
        let ip = String::from("1.1.1.1").parse::<IpAddr>().unwrap();
        let socket_addr = SocketAddr {
            addr: net::SocketAddr::new(ip, 69),
        };

        let mut out = BytesMut::new();
        socket_addr.encode(&mut out);
        let result = SocketAddr::decode(&mut out.to_vec().as_slice());
        assert!(result.is_ok());
    }
}
