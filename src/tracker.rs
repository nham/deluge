use metainfo::MetaInfo;
use util;

use bencode::{self, FromBencode, Bencode};
use hyper::Client;
use hyper::header::Connection;
use openssl::crypto::hash as openssl_hash;
use std::io::Read;
use std::net;
use url::percent_encoding::{percent_encode, FORM_URLENCODED_ENCODE_SET};

type Sha1Hash = Vec<u8>;

enum EventType {
    Started,
    Stopped,
    Completed,
    Empty,
}

impl EventType {
    fn as_str(&self) -> &str {
        use self::EventType::*;
        match *self {
            Started => "started",
            Stopped => "stopped",
            Completed => "completed",
            Empty => unimplemented!()
        }
    }
}

struct TrackerRequest {
    // sha1 hash of the value of "info key from the Metainfo file". value will be a dict?
    info_hash: Sha1Hash,

    // length 20 string, generated by client to use as its id
    peer_id: String,

    // port number client is listening on
    port: u16,

    // total number of bytes uploaded by client since client sent `started` event
    uploaded: u64,

    // total number of bytes downloaded by client since blah blah blah
    downloaded: u64,

    // number of bytes that remain to be downloaded by the client
    left: u64,

    // whether client accepts a compact response
    compact: Option<bool>,

    // indicates tracker can omit peer ids in the peers dict.
    // ignored if `compact` is true
    no_peer_id: Option<bool>,

    event: Option<EventType>,
}

impl TrackerRequest {
    fn new(peer_id: String, port: u16, ul: u64, dl: u64,
           left: u64, info_hash: Sha1Hash, event: Option<EventType>) -> TrackerRequest {
        TrackerRequest {
            info_hash: info_hash,
            peer_id: peer_id,
            port: port,
            uploaded: ul,
            downloaded: dl,
            left: left,
            compact: None,
            no_peer_id: None,
            event: event,
        }
    }

    fn get_query_string(&self) -> String {
        let mut v = Vec::new();
        let encoded_info_hash = percent_encode(&self.info_hash,
                                               FORM_URLENCODED_ENCODE_SET);
        v.push(format!("{}={}", "info_hash", encoded_info_hash));
        v.push(format!("{}={}", "left", self.left));
        v.push(format!("{}={}", "uploaded", self.uploaded));
        v.push(format!("{}={}", "downloaded", self.downloaded));
        v.push(format!("{}={}", "port", self.port));
        v.push(format!("{}={}", "peer_id", self.peer_id));
        if self.event.is_some() {
            v.push(format!("{}={}", "event", self.event.as_ref().unwrap()
                                                       .as_str()));
        }
        v.connect("&")
    }
}

pub fn get_tracker(metainfo: &MetaInfo) {
    // Create a client.
    let client = Client::new();

    let info_hash = openssl_hash::hash(openssl_hash::Type::SHA1,
                                       &metainfo.info_hash_bytes()[..]);
    println!("in get_tracker, info_hash = {:?}", info_hash);
    let req = TrackerRequest::new(String::from("1234567890abcdefghij"), 4567, 0, 0,
                                  metainfo.num_file_bytes() as u64, info_hash,
                                  Some(EventType::Started));

    let query_string = req.get_query_string();
    println!("TrackerRequest: {:?}", query_string);

    let url = format!("{}?{}", metainfo.announce, query_string);

    println!("get_tracker, url = {:?}", url);

    // Creating an outgoing request.
    let send = client.get(&url)
                     .header(Connection::close())
                     .send();
    let mut res = match send {
        Ok(res) => res,
        Err(e) => return panic!("Error: {:?}", e),
    };

    // Read the Response.
    let mut body = Vec::new();
    match res.read_to_end(&mut body) {
        Ok(_) => {},
        Err(e) => return panic!("Error reading to buffer: {:?}", e),
    }

    println!("Response: {:?}", body);

    let bencode = match bencode::from_buffer(&body) {
        Ok(b) => b,
        Err(e) => return panic!("Error creating Bencoded value from response string: {:?}", e),
    };

    println!("About to make TrackerResponse");
    let resp = <TrackerResponse>::from_bencode(&bencode);


}



struct TrackerResponse {
    pub failure_reason: Option<String>,

    // seconds a client should wait before sending requests to the tracker
    pub interval: Option<i64>,

    // a string to be sent by client on following announcements
    pub tracker_id: Option<String>,

    // number of peers with the entire file (seeders)
    pub complete: Option<i64>,

    // number of non-complete peers (leechers?)
    pub incomplete: Option<i64>,

    // This is perhaps the "dictionary model" in the unofficial spec?
    pub peers: Option<Vec<ResponsePeerInfo>>,

}

struct ResponsePeerInfo {
    peer_id: String,
    addr: net::SocketAddr,
}

impl FromBencode for ResponsePeerInfo {
    type Err = String;
    fn from_bencode(b: &Bencode) -> Result<ResponsePeerInfo, Self::Err> {
        match *b {
            Bencode::Dict(ref map) => {
                let peer_id = util::get_field(map, "peer id");
                let ip = util::get_field(map, "ip");
                let port = util::bencode_unwrap_number(
                               util::get_field(map, "port")
                           ) as u16;
                println!("ip: {:?}", ip);
                //let ip = net::IpAddr::V4(net::Ipv4Addr::new(a, b, c, d));
                unimplemented!()
                /*
                Ok(ResponsePeerInfo {
                    peer_id: util::bencode_string_unwrap_string(peer_id),
                    addr: net::SocketAddr::new(ip, port),
                })
                */
            },
            _ => Err(String::from("Cannot convert to ResponsePeerInfo, not a dictionary")),
        }
    }
}

impl FromBencode for TrackerResponse {
    type Err = String;
    fn from_bencode(b: &Bencode) -> Result<TrackerResponse, Self::Err> {
        match *b {
            Bencode::Dict(ref map) => {
                let failure_reason = util::maybe_get_field(map, "failure reason");
                let interval = util::maybe_get_field(map, "interval");
                let tracker_id = util::maybe_get_field(map, "tracker id");
                let complete = util::maybe_get_field(map, "complete");
                let incomplete = util::maybe_get_field(map, "incomplete");
                let peers = util::maybe_get_field(map, "peers");

                println!("failure reason = {:?},\n\
                          interval = {:?},\n\
                          tracker id = {:?},\n\
                          complete = {:?},\n\
                          incomplete = {:?},\n\
                          peers = {:?}",
                          failure_reason,
                          //failure_reason.map(|b| String::from_utf8(b.as_slice())),
                          interval, tracker_id,
                          complete, incomplete, peers);

                let resp = TrackerResponse {
                    failure_reason: failure_reason.map(|b| util::bencode_string_unwrap_string(b)),
                    interval: interval.map(|b| util::bencode_unwrap_number(b)),
                    tracker_id: tracker_id.map(|b| util::bencode_string_unwrap_string(b)),
                    complete: complete.map(|b| util::bencode_unwrap_number(b)),
                    incomplete: incomplete.map(|b| util::bencode_unwrap_number(b)),
                    peers: unimplemented!(),
                };
                Ok(resp)
            },
            _ => Err(String::from("Bencoded value is not a dictionary.")),
        }
    }
}
