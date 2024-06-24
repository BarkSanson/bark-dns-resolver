use bark_dns_resolver::requester::Requester;

fn main() {
    Requester::get_ipv4_address("gmail.google.com");
}