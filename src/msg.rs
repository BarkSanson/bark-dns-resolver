// TODO:
// - Check if TryFrom instead of From is better. Just panicking in case of a wrong code
// is probably not the best option. However, this implementation will work for now.
// - Limit label length to 63 octets

use std::net::Ipv4Addr;

use crate::domain_name::DomainName;
use crate::resource_record::{Class, ResourceRecordHeader, ResponseData, Type};
use crate::serialize::{Deserialize, EncodingError, Serialize};

pub enum MessageError {
    InvalidOpcode,
    InvalidResponseCode,
    InvalidMessageType
}

#[derive(Copy, Clone)]
enum Opcode {
    StandardQuery = 0,
    StatusQuery = 2
}

impl TryFrom<u8> for Opcode {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::StandardQuery),
            2 => Ok(Opcode::StatusQuery),
            _ => Err(MessageError::InvalidOpcode)
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

impl TryFrom<u8> for ResponseCode {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ResponseCode::NoError),
            1 => Ok(ResponseCode::FormatError),
            2 => Ok(ResponseCode::ServerError),
            3 => Ok(ResponseCode::NameError),
            4 => Ok(ResponseCode::NotImplementedError),
            5 => Ok(ResponseCode::RefusedError),
            _ => Err(MessageError::InvalidResponseCode)
        }
    }
}

#[derive(Copy, Clone)]
enum MessageType {
    Query = 0,
    Response = 1
}

impl TryFrom<u8> for MessageType {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::Query),
            1 => Ok(MessageType::Response),
            _ => Err(MessageError::InvalidMessageType)
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
    type Error = EncodingError;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
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

        Ok(bytes)
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = MessageError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let id = u16::from_be_bytes([value[0], value[1]]);
        let flags = value[2];
        let qr = MessageType::try_from((flags & 0b10000000) >> 7)?;
        let opcode = Opcode::try_from((flags & 0b01111000) >> 3)?;
        let aa = (flags & 0b00000100) >> 2;
        let tc = (flags & 0b00000010) >> 1;
        let rd = flags & 0b00000001;
        let ra = (value[3] & 0b10000000) >> 7;
        let response_code = ResponseCode::try_from(value[3] & 0b00001111)?;
        let qdcount = u16::from_be_bytes([value[4], value[5]]);
        let ancount = u16::from_be_bytes([value[6], value[7]]);
        let nscount = u16::from_be_bytes([value[8], value[9]]);
        let arcount = u16::from_be_bytes([value[10], value[11]]);

        Ok(Self {
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
        })
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
    type Error = EncodingError;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let mut bytes = vec![];

        let hostname_bytes = self.qname.serialize()?;
        bytes.extend_from_slice(&hostname_bytes);

        let qtype_bytes = self.qtype as u16;
        bytes.extend_from_slice(&qtype_bytes.to_be_bytes());

        let qclass_bytes = self.qclass as u16;
        bytes.extend_from_slice(&qclass_bytes.to_be_bytes());

        Ok(bytes)
    }
}

pub struct DNSMessage {
    header: Header,
    question: Question,
    answers: Option<Vec<ResourceRecordHeader>>,
    authorities: Option<Vec<ResourceRecordHeader>>,
    additional: Option<Vec<ResourceRecordHeader>>
}

impl DNSMessage {
    pub(crate) fn new_from_components(
        header: Header,
        question: Question,
        answers: Option<Vec<ResourceRecordHeader>>,
        authorities: Option<Vec<ResourceRecordHeader>>,
        additional: Option<Vec<ResourceRecordHeader>>) -> Self {
        Self {
            header,
            question,
            answers,
            authorities,
            additional
        }
    }

    pub(crate) fn new_query_from_hostname(hostname: DomainName) -> Self {
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
    type Error = EncodingError;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let unflattened_bytes = vec![
            self.header.serialize()?,
            self.question.serialize()?
        ];

        Ok(unflattened_bytes.concat())
    }
}

impl Deserialize for DNSMessage {
    type Error = MessageError;

    fn deserialize(bytes: &[u8]) -> Result<Self, Self::Error>{
        let header = Header::try_from(&bytes[..12])?;
        let mut i= 12usize;

        let (qname_length, qname) = DomainName::from_with_bytes(&bytes, i);

        i += qname_length;

        let qtype = Type::from(u16::from_be_bytes([
            bytes[i],
            bytes[i + 1]]));
        let qclass = Class::from(u16::from_be_bytes([
            bytes[i + 2],
            bytes[i + 3]]));

        i += 4;

        let question = Question::new(qname, qtype, qclass);

        let mut answers = None;
        if header.ancount != 0 {
            let mut answers_vec = Vec::new();
            for _ in 0..header.ancount {
                let (read_bytes, name) = DomainName::from_with_bytes(bytes, i);

                i += read_bytes;

                let rr_type = Type::from(u16::from_be_bytes([
                    bytes[i],
                    bytes[i + 1]]));

                let rr_class = Class::from(u16::from_be_bytes([
                    bytes[i + 2],
                    bytes[i + 3]]));

                i += 4;

                let ttl = i32::from_be_bytes([
                    bytes[i],
                    bytes[i + 1],
                    bytes[i + 2],
                    bytes[i + 3],
                ]);

                i += 4;

                let rdlength = u16::from_be_bytes([
                    bytes[i],
                    bytes[i + 1],
                ]);

                i += 2;

                // RData depends on the values of class and type. However,
                // since all transactions usually occur using the IN class - the
                // Internet - and because I don't know what the Chaos class is (just yet),
                // I'll just implement it this way (temporally (or not 😇))
                let rdata = match rr_type {
                    Type::A => {
                        ResponseData::A(Ipv4Addr::from([
                            bytes[i],
                            bytes[i + 1],
                            bytes[i + 2],
                            bytes[i + 3],
                        ]))
                    },
                    Type::CName => {
                        let (read_bytes, dn) = DomainName::from_with_bytes(&bytes, i);
                        i += read_bytes;
                        ResponseData::CName(dn)
                    },
                    _ => panic!("Type not supported")
                    // Type::NameServer => {}
                    // Type::SOA => {}
                    // Type::WKS => {}
                    // Type::PTR => {}
                    // Type::MailExchange => {}
                };

                let rr = ResourceRecordHeader::new(
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

        Ok(Self {
            header,
            question,
            answers,
            authorities,
            additional
        })
    }
}
