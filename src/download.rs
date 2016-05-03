use std::net::TcpStream;
use std::io::{self, Read, Write};

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

fn create_handshake(info: &MetaInfo, peer_id: String) -> Vec<u8> {
    let mut handshake = Vec::new();
    handshake.push(PROTOCOL.len() as u8);
    handshake.extend(PROTOCOL.bytes());
    handshake.extend(b"\x00\x00\x00\x00\x00\x00\x00\x00");
    handshake.append(&mut info.info_hash.clone());
    let mut peer_id_bytes = peer_id.into_bytes();
    handshake.append(&mut peer_id_bytes);
    handshake
}

enum HandshakeError {
    IoError(io::Error),
    ProtocolError(String),
}

impl From<io::Error> for HandshakeError {
    fn from(e: io::Error) -> HandshakeError {
        HandshakeError::IoError(e)
    }
}


impl From<String> for HandshakeError {
    fn from(e: String) -> HandshakeError {
        HandshakeError::ProtocolError(e)
    }
}

impl From<ReadError> for HandshakeError {
    fn from(e: ReadError) -> HandshakeError {
        HandshakeError::IoError(
            match e {
                ReadError::IoError(e) => e,
                ReadError::SocketClosed => io::Error::new(io::ErrorKind::UnexpectedEOF,
                                                          "Socket closed"),
            })
    }
}

fn receive_handshake(stream: &mut TcpStream, info: &MetaInfo) -> Result<(), HandshakeError> {
    let mut buf_pstrlen = [0; 1];
    try!(stream.read_exact(&mut buf_pstrlen));

    let pstrlen = buf_pstrlen[0];
    if (pstrlen as usize) != PROTOCOL.len() {
        return try!(Err(String::from("pstrlen isn't 19")));
    } 

    // Ignore actual protocol string. TODO
    try!(read_n(stream, pstrlen as u64));

    let mut buf_reserved = [0; 8];
    try!(stream.read_exact(&mut buf_reserved));

    let mut buf_infohash = [0; 20];
    try!(stream.read_exact(&mut buf_infohash));
    if buf_infohash != info.info_hash[..] {
        return try!(Err(String::from("Info hash doesn't match")));
    }
    Ok(())

}

enum ReadError {
    SocketClosed,
    IoError(io::Error),
}

fn read_n(stream: &mut TcpStream, n: u64) -> Result<Vec<u8>, ReadError> {
    let mut buf = Vec::new();
    try!(read_n_to_buf(stream, n, &mut buf));
    Ok(buf)
}

fn read_n_to_buf(stream: &mut TcpStream, n: u64, buf: &mut Vec<u8>) -> Result<(), ReadError> {
    let read = stream.take(n).read_to_end(buf);
    match read {
        Ok(0) => Err(ReadError::SocketClosed),
        Ok(num_read) if (num_read as u64) == n => Ok(()),
        Ok(num_read) => read_n_to_buf(stream, n - (num_read as u64), buf),
        Err(e) => Err(ReadError::IoError(e)),
    }
}

pub fn download(info: &MetaInfo, peers: &[Peer], peer_id: String) -> Result<(), io::Error> {
    let handshake = create_handshake(info, peer_id);

    for peer in peers {
        println!("trying to  connect to {:?}", peer.addr);
        let mut stream = match TcpStream::connect(peer.addr) {
            Ok(s) => { println!("successfully connected to {:?}", peer.addr); s },
            Err(_) => continue,
        };

        println!("about to send handshake");
        try!(stream.write_all(&handshake[..]));
        println!("timeout: {:?}", stream.read_timeout());
        receive_handshake(&mut stream, info);
    }

    Ok(())
}
