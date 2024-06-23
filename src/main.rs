use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str::FromStr;

use bark_dns_resolver::ip::IpAddressType;
use bark_dns_resolver::transport::SocketType;
use bark_dns_resolver::dns::{DNSMessage, DomainName};
use bark_dns_resolver::serialize::Serialize;

const DEFAULT_NAME_SERVER: &str = "8.8.8.8";

fn get_ip_addresses(
    hostname: DomainName,
    family: IpAddressType,
    socket_type: SocketType
    ) -> Vec<DNSMessage> {
    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    let dns_server_socket_addr = SocketAddrV4::new(Ipv4Addr::from_str(DEFAULT_NAME_SERVER).unwrap(), 53);
    let udp_socket = UdpSocket::bind(socket_addr).expect("Could not bind to address");
    udp_socket.connect(dns_server_socket_addr).expect(format!("Couldn't connect to {:?}", dns_server_socket_addr).as_str());

    let query = DNSMessage::new_query_from_hostname(hostname);
    let b = query.as_bytes();
    let _ = udp_socket.send(b.as_slice());

    let buf = &mut [0u8; 1024];

    let (_, _src) = udp_socket.recv_from(buf).expect("Didn't receive data");

    let msg = DNSMessage::from(buf.to_vec());

    vec![msg]
}


fn main() {
    get_ip_addresses(DomainName::from_string("gmail.google.com"), IpAddressType::V4, SocketType::UDP);
}