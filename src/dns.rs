pub struct Hostname(String);

impl Hostname {
    pub fn from_string(hostname: &str) -> Self {
        Hostname(hostname.to_string())
    }
}


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

enum MessageType {
    Query,
    Response
}

struct Header {
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

struct Question {
    qname: Hostname,
    qtype: Type,
    qclass: Class
}

impl Question {
    fn new(hostname: Hostname) -> Self {
        Self {
            qname: hostname,
            qtype: Type::A,
            qclass: Class::Internet
        }
    }
}

struct Answer;
struct Authority;
struct Additional;

pub struct DNSMessage {
    header: Header,
    question: Question,
    answer: Option<Answer>,
    authority: Option<Authority>,
    additional: Option<Additional>
}

impl DNSMessage {
    pub fn new_query_from_hostname(hostname: Hostname) -> Self {
        let id = rand::random::<u16>();
        let header = Header::standard_query_from_id(id);
        let question = Question::new(hostname);

        Self {
            header,
            question,
            answer: None,
            authority: None,
            additional: None
        }
    }
}
