mod resource_record;

use std::net;
use std::net::IpAddr;
use resource_record::{Type, Class};

enum Opcode {
    StandardQuery,
    StatusQuery
}

enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerError = 2,
    NameError = 3,
    NotImplementedError = 4,
    RefusedError = 5
}


struct Header {
    id: u16,
    qr: bool,
    opcode: Opcode,
    authoritative: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    response_code: ResponseCode,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16
}

struct Question {
    qname: String,
    qtype: Type,
    qclass: Class
}

struct Answer;
struct Authority;
struct Additional;

struct DNSMessage {
    header: Header,
    question: Question,
    answer: Option<Answer>,
    authority: Option<Authority>,
    additional: Option<Additional>
}

fn get_ip_addresses(
    domain_name: &str,
    family: IpAddr,
    ) -> Vec<net::Ipv4Addr> {
    Vec::new()
}


fn main() {

}