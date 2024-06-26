use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str::FromStr;

use crate::domain_name::DomainName;
use crate::msg::{DNSMessage, MessageError};
use crate::serialize::{Deserialize, DeserializationError, Serialize};

const DEFAULT_NAME_SERVER: &str = "8.8.8.8";

pub enum DNSError {
    Io(io::Error),
    Encoding(DeserializationError),
    Message(MessageError)
}

impl From<io::Error> for DNSError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<DeserializationError> for DNSError {
    fn from(value: DeserializationError) -> Self {
        Self::Encoding(value)
    }
}

impl From<MessageError> for DNSError {
    fn from(value: MessageError) -> Self {
        Self::Message(value)
    }
}

pub struct Requester;

impl Requester {
    pub fn get_ipv4_address(name: &str) -> Result<Vec<Ipv4Addr>, DNSError> {
        // 1. Create socket
        let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
        let dns_server_socket_addr = SocketAddrV4::new(Ipv4Addr::from_str(DEFAULT_NAME_SERVER).unwrap(), 53);
        let udp_socket = UdpSocket::bind(socket_addr).expect("Could not bind to address");

        // 2. Connect socket
        udp_socket.connect(dns_server_socket_addr).expect(format!("Couldn't connect to {:?}", dns_server_socket_addr).as_str());

        // 3. Generate standard query, serialize it and send it through the UDP socket
        let query = DNSMessage::new_query_from_hostname(DomainName::from_string(name));
        let b = query.serialize();
        let _ = udp_socket.send(b.as_slice());

        // 5. Prepare buffer for response, deserialize message and get the result
        let buf = &mut [0u8; 512];
        let (_, _src) = udp_socket.recv_from(buf).expect("Didn't receive data");

        let msg = DNSMessage::deserialize(buf, 0)?;

        Ok(vec![])
    }
}