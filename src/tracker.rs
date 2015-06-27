use metainfo::MetaInfo;

use hyper::Client;
use hyper::header::Connection;
use std::io::Read;

pub fn get_tracker(metainfo: &MetaInfo) {
    // Create a client.
    let mut client = Client::new();

    println!("in get_tracker");
    // Creating an outgoing request.
    let send = client.get(&metainfo.announce)
                     .header(Connection::close())
                     .send();
    let mut res: () = match send {
        Ok(res) => res,
        Err(e) => panic!("Error: {:?}", e),
    };

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);
}
