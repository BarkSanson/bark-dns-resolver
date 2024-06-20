use crate::dns::{Additional, Answer, Authority, DNSMessage, Header, Question};
use crate::serialize::Deserialize;

struct ResponseParser;

impl ResponseParser {
    pub fn parse_response(response: Vec<u8>) -> DNSMessage {
        let header = Header::from_bytes(response.as_slice());
        let question = Question::from_bytes(&response);
        let answer = Answer::from_bytes(&response);
        let authority = Authority::from_bytes(&response);
        let additional = Additional::from_bytes(&response);

        DNSMessage::new_from_components(
            header,
            question,
            answer,
            authority,
            additional
        )
    }
}