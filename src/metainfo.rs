use bencode::{self, FromBencode, Bencode};
use std::collections::BTreeMap;
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

    // in Unix epoch format
    creation_date: Option<String>,

    // name and version of program that created the torrent file
    created_by: Option<String>,

    // encoding used for `pieces` portion of info dictionary
    encoding: Option<String>,
}

impl fmt::Debug for MetaInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "MetaInfo {{ announce: {:?}, created_by: {:?} }}",
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


pub type DecodeError = String;

#[derive(Debug)]
enum MetaInfoFieldType {
    ByteString(String),
    Number(i64),
}

// extract a Bencoded ByteString field from a BTreeMap (by key)
fn get_field(map: &BTreeMap<bencode::util::ByteString, Bencode>,
             key: &bencode::util::ByteString)
        -> Result<MetaInfoFieldType, DecodeError> {
    match map.get(key) {
        Some(&Bencode::ByteString(ref s)) => {
            match String::from_utf8(s.clone()) {
                Ok(s) => Ok(MetaInfoFieldType::ByteString(s)),
                Err(e) => Err(format!("Error: {}", e)),
            }
        },
        Some(&Bencode::Number(x)) => Ok(MetaInfoFieldType::Number(x)),
        Some(_) => Err(format!("{:?}'s value in the dictionary is not a string",
                               String::from_utf8(key.clone().unwrap() ))),
        None => Err(format!("{:?} not found in the dictionary", key)),
    }
}


impl FromBencode for MetaInfo {
    type Err = DecodeError;
    fn from_bencode(b: &Bencode) -> Result<MetaInfo, Self::Err> {
        match *b {
            Bencode::Dict(ref m) => {
                let announce_key = &bencode::util::ByteString::from_str("announce");
                let created_by_key = &bencode::util::ByteString::from_str("created by");
                let creation_date_key = &bencode::util::ByteString::from_str("creation date");
                let encoding_key = &bencode::util::ByteString::from_str("encoding");

                let announce = try!(get_field(m, announce_key));
                let created_by = try!(get_field(m, created_by_key));
                let creation_date = try!(get_field(m, creation_date_key));
                let encoding = try!(get_field(m, encoding_key));

                println!("announce = {:?},\n\
                          creation_date = {:?},\n\
                          created by = {:?},\n\
                          encoding = {:?}",
                          announce, creation_date, created_by, encoding);
                unimplemented!()
            },
            _ => Err(format!("Bencoded string is not a dictionary.")),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    BencodeDecodingError(bencode::streaming::Error),
    Other(String),
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

    println!("parse_torrent_file, path = {:?}\n", path);

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
