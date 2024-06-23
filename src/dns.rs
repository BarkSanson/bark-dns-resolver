// TODO: Check if TryFrom instead of From is better. Just panicking in case of a wrong code
// is probably not the best option. However, this implementation will work for now.

use std::io::Read;
use std::net::Ipv4Addr;

use regex::Regex;

use crate::bytes::FromWithBytes;
use crate::serialize::Serialize;

pub struct DomainName(String);

impl DomainName {
    pub fn from_string(domain_name: &str) -> Self {
        //Self::is_valid(domain_name.to_string());

        Self(domain_name.to_string())
    }

    fn is_valid(domain_name: String) -> bool {
        // I literally copied this regex from stackoverflow
        let reg = Regex::new(
            r"^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$")
            .unwrap();

        if !reg.is_match(&domain_name) {
            panic!("Invalid domain name");
        }

        true
    }
}

impl FromWithBytes for DomainName {
    fn from_with_bytes(bytes: &[u8]) -> (usize, DomainName) {
        let mut labels_vec = Vec::new();
        let mut i = 0;

        while bytes[i] != 0 {
            let label_length = bytes[i];

            if label_length == 0 {
                break;
            }

            let label =
                String::from_utf8(bytes[i + 1..i + 1 + label_length as usize].to_vec()).unwrap();
            labels_vec.push(label);

            i += 1 + label_length as usize;
        }

        let labels = labels_vec.join(".");

        let dn = DomainName::from_string(&labels);

        (i, dn)
    }
}

impl Serialize for DomainName {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        for label in self.0.split('.') {
            let label_length = label.len() as u8;
            bytes.push(label_length);
            bytes.extend_from_slice(label.as_bytes());
        }

        bytes.push(0);

        bytes
    }
}

#[derive(Copy, Clone)]
pub enum Type {
    A = 1,
    NameServer = 2,
    CName = 5,
    SOA = 6,
    WKS = 11,
    PTR = 12,
    MailExchange = 15
}

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        match value {
            1 => Type::A,
            2 => Type::NameServer,
            5 => Type::CName,
            6 => Type::SOA,
            11 => Type::WKS,
            12 => Type::PTR,
            15 => Type::MailExchange,
            _ => panic!("Type not implemented")
        }
    }
}

#[derive(Copy, Clone)]
pub enum Class {
    Internet = 1,
    Chaos = 3 // ??
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match value {
            1 => Class::Internet,
            3 => Class::Chaos,
            _ => panic!("Invalid class code")
        }
    }
}

enum ResponseData {
    A(Ipv4Addr),
    // TODO: implement:
    // - CNAME
    // - SOA
    // - WKS
    // - PTR
    // - HINFO
    // - MINFO
    // - MX
    // - TXT
}

pub struct ResourceRecord {
    name: DomainName,
    rr_type: Type,
    rr_class: Class,
    ttl: i32,
    rdlength: u16,
    rdata: ResponseData
}

impl ResourceRecord {
    fn new(
        name: DomainName,
        rr_type: Type,
        rr_class: Class,
        ttl: i32,
        rdlength: u16,
        rdata: ResponseData
    ) -> Self {
        Self {
            name,
            rr_type,
            rr_class,
            ttl,
            rdlength,
            rdata
        }
    }
}

#[derive(Copy, Clone)]
enum Opcode {
    StandardQuery = 0,
    StatusQuery = 2
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Opcode::StandardQuery,
            2 => Opcode::StatusQuery,
            _ => panic!("Invalid opcode")
        }
    }
}

#[derive(Copy, Clone)]
enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerError = 2,
    NameError = 3,
    NotImplementedError = 4,
    RefusedError = 5
}

impl From<u8> for ResponseCode {
    fn from(value: u8) -> Self {
        match value {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerError,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplementedError,
            5 => ResponseCode::RefusedError,
            _ => panic!("Invalid response code")
        }
    }
}

#[derive(Copy, Clone)]
enum MessageType {
    Query = 0,
    Response = 1
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::Query,
            1 => MessageType::Response,
            _ => panic!("Invalid message type")
        }
    }
}

pub struct Header {
    id: u16,
    qr: MessageType,
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

impl Header {
    fn standard_query_from_id(id: u16) -> Self {
        Self {
            id,
            qr: MessageType::Query,
            opcode: Opcode::StandardQuery,
            authoritative: false,
            truncation: false,
            recursion_desired: true,
            recursion_available: false,
            response_code: ResponseCode::NoError,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0
        }
    }
}

impl Serialize for Header {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let id_bytes = self.id.to_be_bytes();
        bytes.extend_from_slice(&id_bytes);

        let qr= self.qr as u8;
        let opcode = self.opcode as u8;
        let aa: u8 = if self.authoritative { 1 } else { 0 };
        let tc: u8 = if self.truncation { 1 } else { 0 };
        let rd: u8 = if self.recursion_desired { 1 } else { 0 };
        let ra: u8 = if self.recursion_available { 1 } else { 0 };
        let rcode = self.response_code as u8;

        let upper_flags = qr << 7 | opcode << 3 | aa << 2 | tc << 1 | rd;
        let lower_flags = ra << 7 | 0 << 6 | 0 << 5 | 0 << 4 | rcode;
        bytes.extend_from_slice(&[upper_flags, lower_flags]);

        let counts = [self.qdcount, self.ancount, self.nscount, self.arcount];

        for count in counts.iter() {
            let count_bytes = count.to_be_bytes();
            bytes.extend_from_slice(&count_bytes);
        }

        bytes
    }
}

impl From<&[u8]> for Header {
    fn from(value: &[u8]) -> Self {
        let id = u16::from_be_bytes([value[0], value[1]]);
        let flags = value[2];
        let qr = MessageType::from((flags & 0b10000000) >> 7);
        let opcode = Opcode::from((flags & 0b01111000) >> 3);
        let aa = (flags & 0b00000100) >> 2;
        let tc = (flags & 0b00000010) >> 1;
        let rd = flags & 0b00000001;
        let ra = (value[3] & 0b10000000) >> 7;
        let response_code = ResponseCode::from(value[3] & 0b00001111);
        let qdcount = u16::from_be_bytes([value[4], value[5]]);
        let ancount = u16::from_be_bytes([value[6], value[7]]);
        let nscount = u16::from_be_bytes([value[8], value[9]]);
        let arcount = u16::from_be_bytes([value[10], value[11]]);

        Self {
            id,
            qr,
            opcode,
            authoritative: aa == 1,
            truncation: tc == 1,
            recursion_desired: rd == 1,
            recursion_available: ra == 1,
            response_code,
            qdcount,
            ancount,
            nscount,
            arcount
        }

    }
}

pub struct Question {
    qname: DomainName,
    qtype: Type,
    qclass: Class
}

impl Question {
    fn new(qname: DomainName, qtype: Type, qclass: Class) -> Self {
        Self {
            qname,
            qtype,
            qclass
        }
    }

    fn new_from_domain_name(qname: DomainName) -> Self {
        Self {
            qname,
            qtype: Type::A,
            qclass: Class::Internet
        }
    }
}

impl Serialize for Question {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let hostname_bytes = self.qname.as_bytes();
        bytes.extend_from_slice(&hostname_bytes);

        let qtype_bytes = self.qtype as u16;
        bytes.extend_from_slice(&qtype_bytes.to_be_bytes());

        let qclass_bytes = self.qclass as u16;
        bytes.extend_from_slice(&qclass_bytes.to_be_bytes());

        bytes
    }
}

pub struct DNSMessage {
    header: Header,
    question: Question,
    answers: Option<Vec<ResourceRecord>>,
    authorities: Option<Vec<ResourceRecord>>,
    additional: Option<Vec<ResourceRecord>>
}

impl DNSMessage {
    pub fn new_from_components(
        header: Header,
        question: Question,
        answers: Option<Vec<ResourceRecord>>,
        authorities: Option<Vec<ResourceRecord>>,
        additional: Option<Vec<ResourceRecord>>) -> Self {
        Self {
            header,
            question,
            answers,
            authorities,
            additional
        }
    }

    pub fn new_query_from_hostname(hostname: DomainName) -> Self {
        let id = rand::random::<u16>();
        let header = Header::standard_query_from_id(id);
        let question = Question::new_from_domain_name(hostname);

        Self {
            header,
            question,
            answers: None,
            authorities: None,
            additional: None
        }
    }
}

impl Serialize for DNSMessage {
    fn as_bytes(&self) -> Vec<u8> {
        let unflattened_bytes = vec![
            self.header.as_bytes(),
            self.question.as_bytes()
        ];

        unflattened_bytes.concat()
    }
}

impl From<Vec<u8>> for DNSMessage {
    fn from(value: Vec<u8>) -> Self {
        let header = Header::from(&value[..12]);
        let mut i= 12usize;

        let (qname_length, qname) =
            DomainName::from_with_bytes(&value[i..]);

        i += qname_length;

        let qtype = Type::from(u16::from_be_bytes([
            value[i + 1],
            value[i + 2]]));
        let qclass = Class::from(u16::from_be_bytes([
            value[i + 3],
            value[i + 4]]));

        i += 5;

        let question = Question::new(qname, qtype, qclass);

        let mut answers = None;
        if header.ancount != 0 {
            let mut answers_vec = Vec::new();
            for _ in 0..header.ancount {
                let (name_length, name) = DomainName::from_with_bytes(&value[i..]);

                i += name_length;

                let rr_type = Type::from(u16::from_be_bytes([
                    value[i + 1],
                    value[i + 2]]));

                let rr_class = Class::from(u16::from_be_bytes([
                    value[i + 3],
                    value[i + 4]]));

                i += 4;

                let ttl = i32::from_be_bytes([
                    value[i + 1],
                    value[i + 2],
                    value[i + 3],
                    value[i + 4],
                ]);

                i += 4;

                let rdlength = u16::from_be_bytes([
                    value[i + 1],
                    value[i + 2],
                ]);

                i += 4;

                // RData depends on the values of class and type. However,
                // since all transactions usually occur using the IN class - the
                // Internet - and because I don't know what the Chaos class is (just yet),
                // I'll just implement it this way (temporally (or not 😇))
                let rdata = match rr_type {
                    Type::A => {
                        ResponseData::A(Ipv4Addr::from([
                            value[i],
                            value[i + 1],
                            value[i + 2],
                            value[i + 3],
                        ]))
                    },
                    _ => panic!("Type not supported")
                    // Type::NameServer => {}
                    // Type::CName => {}
                    // Type::SOA => {}
                    // Type::WKS => {}
                    // Type::PTR => {}
                    // Type::MailExchange => {}
                };

                let rr = ResourceRecord::new(
                    name,
                    rr_type,
                    rr_class,
                    ttl,
                    rdlength,
                    rdata
                );

                answers_vec.push(rr);
            }

            answers = Some(answers_vec);
        }

        let authorities = None;
        if header.nscount != 0 {

        }

        let additional = None;
        if header.arcount != 0 {

        }

        Self {
            header,
            question,
            answers,
            authorities,
            additional
        }
    }
}