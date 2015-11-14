use std::net::TcpStream;
use std::io::{Read, Write};

use metainfo::MetaInfo;
use tracker::Peer;

struct PeerConnection {
    am_choking: bool,
    am_interested: bool,
    peer_choking: bool,
    peer_interested: bool,
    peer: Peer,
}

const PROTOCOL: &'static str = "BitTorrent protocol";

pub fn download(info: &MetaInfo, peers: &[Peer], peer_id: String) -> Result<(), ::std::io::Error> {
    let mut handshake = Vec::new();
    handshake.push(PROTOCOL.len() as u8);
    handshake.extend(PROTOCOL.bytes());
    handshake.extend(b"\x00\x00\x00\x00\x00\x00\x00\x00");
    handshake.append(&mut info.info_hash.clone());
    let mut peer_id_bytes = peer_id.into_bytes();
    handshake.append(&mut peer_id_bytes);

    for peer in peers {
        println!("trying to  connect to {:?}", peer.addr);
        let mut stream = match TcpStream::connect(peer.addr) {
            Ok(s) => { println!("successfully connected to {:?}", peer.addr); s },
            Err(_) => continue,
        };

        println!("about to send handshake");
        try!(stream.write_all(&handshake[..]));
        println!("timeout: {:?}", stream.read_timeout());
        let mut buf = Vec::new();
        let bytes_read = stream.read_to_end(&mut buf);
        println!("bytes_read: {:?}, buf: {:?}", bytes_read, buf);
    }

    Ok(())
}
