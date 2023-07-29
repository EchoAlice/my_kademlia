use alloy_rlp::{encode_list, Decodable, Encodable, Error};
use bytes::{BufMut, BytesMut};
use std::net;

pub fn encoded<T: Encodable>(t: &T) -> BytesMut {
    let mut out = BytesMut::new();
    t.encode(&mut out);

    println!("Out (encode): {:?}", out);
    out
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SocketAddr {
    pub addr: net::SocketAddr,
}

impl Encodable for SocketAddr {
    fn encode(&self, out: &mut dyn BufMut) {
        match self.addr {
            net::SocketAddr::V4(socket) => {
                let ip = socket.ip().octets();
                let port = socket.port();
                let mut enc: [&dyn Encodable; 3] = [b""; 3];

                enc[0] = &0_u8;
                enc[1] = &ip;
                enc[2] = &port;

                encode_list::<_, dyn Encodable>(&enc, out);
            }
            net::SocketAddr::V6(socket) => {
                let ip = socket.ip().octets();
                let port = socket.port();
                let mut enc: [&dyn Encodable; 3] = [b""; 3];

                enc[0] = &1_u8;
                enc[1] = &ip;
                enc[2] = &port;

                encode_list::<_, dyn Encodable>(&enc, out);
            }
        };
    }
}

impl Decodable for SocketAddr {
    fn decode(data: &mut &[u8]) -> Result<Self, Error> {
        let mut payload = alloy_rlp::Header::decode_bytes(data, true)?;

        let typ = u8::decode(&mut payload)?;
        let addr = match typ {
            0 => {
                let ip = <[u8; 4]>::decode(&mut payload)?;
                let port = u16::decode(&mut payload)?;
                net::SocketAddr::new(ip.into(), port)
            }
            1 => {
                let ip = <[u8; 16]>::decode(&mut payload)?;
                let port = u16::decode(&mut payload)?;
                net::SocketAddr::new(ip.into(), port)
            }
            _ => panic!("Not a SocketAddr"),
        };

        Ok(Self { addr })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_rlp::Decodable;
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
        assert_eq!(result.unwrap(), socket_addr);
    }

    #[test]
    fn socket_addr_serialization_vec() {
        let ip = String::from("1.1.1.1").parse::<IpAddr>().unwrap();
        let foo = SocketAddr {
            addr: net::SocketAddr::new(ip, 8080),
        };
        let foos = vec![foo.clone(), foo];
        let mut out = vec![];
        foos.encode(&mut out);
        let recovered = Vec::<SocketAddr>::decode(&mut out.as_slice()).unwrap();
        assert_eq!(foos, recovered);
    }
}
