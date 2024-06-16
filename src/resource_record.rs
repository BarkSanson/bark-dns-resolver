pub enum Type {
    A = 1,
    NameServer = 2,
    CName = 5,
    SOA = 6,
    WKS = 11,
    PTR = 12,
    MailExchange = 15
}

pub enum Class {
    Internet = 1,
    Chaos = 3 // ??
}

pub struct ResourceRecord {
    name: String,
    rr_type: Type, // Name's rr_type and not type because of the keyword
    class: Class,
    ttl: u32,
    rdlength: u16,
    rdata: usize // TODO: change usize
}