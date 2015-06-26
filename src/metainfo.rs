use bencode::{self, FromBencode, Bencode};
use std::path::PathBuf;
use std::fs::File;
use std::fmt;
use std::io::{self, Read};

static DEFAULT_TORRENT_FILE: &'static str = "flagfromserver.torrent";
static TORRENT_FILE_DIR: &'static str = "data";

pub struct MetaInfo {
    info: Info,

    // announce URL of tracker
    announce: String,

    // name and version of program that created the torrent file
    created_by: String, 
}

impl fmt::Debug for MetaInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "MetaInfo {{ announce: {}, created_by: {} }}",
               self.announce, self.created_by)
    }
}

// "a dictionary that describes the file(s) of the torrent"
struct Info {
    piece_length: u32,
    pieces: Vec<u8>,
    private: Option<bool>,
    mode_info: ModeInfo,
}

enum ModeInfo {
    Single(SingleFileInfo),
    Multiple(MultipleFileInfo),
}

struct SingleFileInfo {
    name: String,
    length: u32, // length of file in bytes
    md5sum: Option<[char; 32]>,
}

struct MultipleFileInfo {
    name: String,
    files: Vec<MultipleFileIndividualFileInfo>,
}

// this name is less than ideal...
struct MultipleFileIndividualFileInfo {
    length: u32,
    md5sum: Option<[char; 32]>,
    path: Vec<String>, // maybe should be a std::path::Path
}


impl FromBencode for MetaInfo {
    type Err = &'static str;
    fn from_bencode(b: &Bencode) -> Result<MetaInfo, Self::Err> {
        match *b {
            Bencode::Dict(ref d) => {
                unimplemented!()
            },
            _ => Err("Bencoded string is not a dictionary."),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    BencodeDecodingError(bencode::streaming::Error),
    Other(&'static str),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::IoError(err)
    }
}

impl From<bencode::streaming::Error> for ParseError {
    fn from(err: bencode::streaming::Error) -> ParseError {
        ParseError::BencodeDecodingError(err)
    }
}

pub fn parse_torrent_file(torrent_file: Option<&str>) -> Result<MetaInfo, ParseError> {
    let fname = torrent_file.unwrap_or(DEFAULT_TORRENT_FILE);
    let mut path = PathBuf::from(TORRENT_FILE_DIR);
    path.push(fname);

    println!("parse_torrent_file, path = {:?}", path);

    let mut f = try!(File::open(path));
    let mut buf = Vec::new();
    try!(f.read_to_end(&mut buf));

    let bencode = try!(bencode::from_vec(buf));
    let metainfo = match FromBencode::from_bencode(&bencode) {
        Ok(metainfo) => metainfo,
        Err(e) => return Err(ParseError::Other(e))
    };

    Ok(metainfo)
}
