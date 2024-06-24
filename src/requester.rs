use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use crate::domain_name::DomainName;
use crate::msg::DNSMessage;
use crate::serialize::Serialize;

const DEFAULT_NAME_SERVER: &str = "8.8.8.8";

pub struct Requester;

impl Requester {
    pub fn get_ipv4_address(name: &str) -> Vec<DNSMessage>{
        let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
        let dns_server_socket_addr = SocketAddrV4::new(Ipv4Addr::from_str(DEFAULT_NAME_SERVER).unwrap(), 53);
        let udp_socket = UdpSocket::bind(socket_addr).expect("Could not bind to address");
        udp_socket.connect(dns_server_socket_addr).expect(format!("Couldn't connect to {:?}", dns_server_socket_addr).as_str());

        let query = DNSMessage::new_query_from_hostname(DomainName::from_string(name));
        let b = query.serialize();
        let _ = udp_socket.send(b.as_slice());

        let buf = &mut [0u8; 512];

        let (_, _src) = udp_socket.recv_from(buf).expect("Didn't receive data");

        let msg = DNSMessage::from(buf.to_vec());

        vec![msg]
    }
}