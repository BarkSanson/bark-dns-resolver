use std::net;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;

use ip::IpAddressType;
use transport::SocketType;

use crate::dns::{DNSMessage, Hostname};

mod ip;
mod transport;
mod dns;
mod udp;

const DEFAULT_NAME_SERVER: &str = "8.8.8.8";

fn get_ip_addresses(
    hostname: Hostname,
    family: IpAddressType,
    socket_type: SocketType
    ) -> Vec<net::Ipv4Addr> {
    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    let dns_server_socket_addr = SocketAddrV4::new(Ipv4Addr::from_str(DEFAULT_NAME_SERVER).unwrap(), 53);
    let udp_socket = UdpSocket::bind(socket_addr).expect("Could not bind to address");
    udp_socket.connect(dns_server_socket_addr).expect(format!("Couldn't connect to {:?}", dns_server_socket_addr).as_str());

    let query = DNSMessage::new_query_from_hostname(hostname);
    udp_socket.send(); // TODO



    Vec::new()
}


fn main() {
    get_ip_addresses(Hostname::from_string("hola.com"), IpAddressType::V4, SocketType::UDP);
}