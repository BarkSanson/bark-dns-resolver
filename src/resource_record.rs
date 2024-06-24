use std::net::Ipv4Addr;
use crate::domain_name::DomainName;

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

pub(crate) enum ResponseData {
    A(Ipv4Addr),
    CName(DomainName)
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
    pub(crate) fn new(
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
