use crate::dns::{Additional, Answer, Authority, DNSMessage, Header, Question};
use crate::serialize::Deserialize;

struct ResponseParser;

impl ResponseParser {
    pub fn parse_response(response: Vec<u8>) -> DNSMessage {
        let question = Question::from_bytes(&response);

        // TODO: handle case where answer, authority, and additional sections aren't present
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