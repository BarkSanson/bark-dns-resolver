use std::net;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;

use ip::IpAddressType;
use transport::SocketType;

use crate::dns::{DNSMessage, DomainName};
use crate::serialize::Serialize;

mod ip;
mod transport;
mod dns;
mod request;
mod serialize;
mod parser;
mod bytes;

const DEFAULT_NAME_SERVER: &str = "8.8.8.8";

fn get_ip_addresses(
    hostname: DomainName,
    family: IpAddressType,
    socket_type: SocketType
    ) -> Vec<net::Ipv4Addr> {
    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    let dns_server_socket_addr = SocketAddrV4::new(Ipv4Addr::from_str(DEFAULT_NAME_SERVER).unwrap(), 53);
    let udp_socket = UdpSocket::bind(socket_addr).expect("Could not bind to address");
    udp_socket.connect(dns_server_socket_addr).expect(format!("Couldn't connect to {:?}", dns_server_socket_addr).as_str());

    let query = DNSMessage::new_query_from_hostname(hostname);
    let b = query.as_bytes();
    let r = udp_socket.send(b.as_slice());

    let buf = &mut [0u8; 1024];

    let (amt, _src) = udp_socket.recv_from(buf).expect("Didn't receive data");

    Vec::new()
}


fn main() {
    get_ip_addresses(DomainName::from_string("hola.com"), IpAddressType::V4, SocketType::UDP);
}