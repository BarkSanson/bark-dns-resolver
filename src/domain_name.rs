use regex::Regex;
use crate::serialize::{Deserialize, DeserializationError, Serialize};

#[derive(Debug)]
pub(crate) struct DomainName(String);

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

impl Serialize for DomainName {

    fn serialize(&self) -> Vec<u8> {
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

impl Deserialize for DomainName {
    fn deserialize(bytes: &[u8], offset: usize)
        -> Result<(usize, Self), DeserializationError>
    where
        Self: Sized
    {
        let mut labels_vec = Vec::new();
        let mut idx = offset;
        let mut is_compression_byte = false;
        let mut read_bytes = 0;
        let mut nested = false;

        while bytes[idx] != 0 {
            // Check if compression is being used. This raw comparison is
            // safe since labels are restricted to 63 octets or less
            // (see RFC 1035).
            is_compression_byte = (bytes[idx] >> 6) == 0b00000011;

            if is_compression_byte  {
                // 0x3FFF discards the first two bits of the word, since these are not
                // necessary because they only indicate that the word is a pointer
                idx = (u16::from_be_bytes([bytes[idx], bytes[idx + 1]]) & 0x3FFF) as usize;

                if !nested {
                    read_bytes += 2;
                    nested = true;
                }
                continue;
            }

            let label_length = bytes[idx] as usize;
            idx += 1;

            if label_length == 0 {
                break;
            }

            let label =
                String::from_utf8(bytes[idx..idx + label_length].to_vec()).unwrap();
            labels_vec.push(label);

            idx += label_length;
            if !nested {
                read_bytes += label_length + 1;
            }
        }

        // Also count the 0 byte
        if !nested {
            read_bytes += 1;
        }

        let labels = labels_vec.join(".");

        let dn = DomainName::from_string(&labels);

        Ok((read_bytes, dn))

    }
}