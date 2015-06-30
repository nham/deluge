use util;

use bencode::{self, FromBencode, Bencode};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::fmt;
use std::io::{self, Read};

static TORRENT_FILE_DIR: &'static str = "data";

pub struct MetaInfo {
    pub info: Box<InfoDictionary>,

    // announce URL of tracker
    pub announce: String,

    // in Unix epoch format
    pub creation_date: Option<i64>,

    // name and version of program that created the torrent file
    pub created_by: Option<String>,

    // encoding used for `pieces` portion of info dictionary
    pub encoding: Option<String>,
}

impl MetaInfo {
    pub fn num_file_bytes(&self) -> u32 {
        self.info.num_file_bytes()
    }

    pub fn info_hash_bytes(&self) -> Vec<u8> {
        self.info.info_hash_bytes()
    }
}

impl fmt::Debug for MetaInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "MetaInfo {{ announce: {:?}, created_by: {:?} }}",
               self.announce, self.created_by)
    }
}

pub trait InfoDictionary {
    fn info_hash_bytes(&self) -> Vec<u8>;
    fn num_file_bytes(&self) -> u32;
}

// "a dictionary that describes the file(s) of the torrent"
pub struct SingleFileInfo {
    pub piece_length: u32,
    pub pieces: Vec<u8>,

    // name of file name
    pub name: String,

    // length of file in bytes
    pub length: u32,

    pub md5sum: Option<[char; 32]>,
}

fn bencode_dict_insert(dict: &mut Bencode,
                       key: bencode::util::ByteString,
                       value: Bencode) -> Option<Bencode> {
    match *dict {
        Bencode::Dict(ref mut map) => map.insert(key, value),
        _ => return panic!("bencode_dict_insert: `dict` is not a `Bencode::Dict`"),
    }
}

impl InfoDictionary for SingleFileInfo {
    fn info_hash_bytes(&self) -> Vec<u8> {
        let mut dict = Bencode::Dict(BTreeMap::new());
        bencode_dict_insert(&mut dict, bencode::util::ByteString::from_str("piece length"),
                                       Bencode::Number(self.piece_length as i64));
        bencode_dict_insert(&mut dict, bencode::util::ByteString::from_str("pieces"),
                                       Bencode::ByteString(self.pieces.clone()));
        bencode_dict_insert(&mut dict, bencode::util::ByteString::from_str("name"),
                                       Bencode::ByteString(self.name.clone().into_bytes() ));
        bencode_dict_insert(&mut dict, bencode::util::ByteString::from_str("length"),
                                       Bencode::Number(self.length as i64));
        if self.md5sum.is_some() {
            return panic!("md5sum isn't implemented as part of info_hash_bytes yet");
        }
        match dict.to_bytes() {
            Ok(bytes) => bytes,
            Err(e) => panic!("Error converting dict to bytes: {:?}", e),
        }
    }

    fn num_file_bytes(&self) -> u32 { self.length }
}

/*
pub struct MultiFileInfo {
    pub piece_length: u32,
    pub pieces: Vec<u8>,

    // name of directory that contains the files
    pub name: String,

    files: Vec<MultipleFileIndividualFileInfo>,
}

pub enum FileInfo {
    Single(SingleFileInfo),
    Multiple(MultiFileInfo),
}

// this name is less than ideal...
struct MultipleFileIndividualFileInfo {
    length: u32,
    md5sum: Option<[char; 32]>,
    path: Vec<String>, // maybe should be a std::path::Path
}
*/


pub type DecodeError = String;


impl FromBencode for MetaInfo {
    type Err = DecodeError;
    fn from_bencode(b: &Bencode) -> Result<MetaInfo, Self::Err> {
        match *b {
            Bencode::Dict(ref m) => {
                let announce = util::get_field(m, "announce");
                let created_by = util::maybe_get_field(m, "created by");
                let creation_date = util::maybe_get_field(m, "creation date");
                let encoding = util::maybe_get_field(m, "encoding");

                println!("announce = {:?},\n\
                          creation_date = {:?},\n\
                          created by = {:?},\n\
                          encoding = {:?}",
                          announce, creation_date, created_by, encoding);

                let info_dict_map = match util::get_field(m, "info") {
                    Bencode::Dict(map) => map,
                    _ => return Err(String::from("`info` field is not a dictionary.")),
                };

                let piece_length = util::get_field(&info_dict_map, "piece length");
                let pieces = util::get_field(&info_dict_map, "pieces");
                let name = util::get_field(&info_dict_map, "name");
                let length = util::get_field(&info_dict_map, "length");

                println!("piece_length = {:?},\n\
                          pieces = {:?},\n\
                          name = {:?},\n\
                          length = {:?}",
                          piece_length, pieces, name, length);

                // TODO: md5sum
                let info = SingleFileInfo {
                    piece_length: util::bencode_unwrap_number(piece_length) as u32,
                    pieces: util::bencode_string_unwrap_bytes(pieces),
                    name: util::bencode_string_unwrap_string(name),
                    length: util::bencode_unwrap_number(length) as u32,
                    md5sum: None,
                };

                fn unwrap_bencode_bytestring(b: Bencode, field_name: &str) -> String {
                    let bytes = util::bencode_string_unwrap_bytes(b);
                    match String::from_utf8(bytes) {
                        Ok(s) => s,
                        Err(e) => return panic!("Error converting {} to string: {:?}",
                                                field_name, e),
                    }
                }

                Ok(MetaInfo {
                    info: Box::new(info) as Box<InfoDictionary>,
                    announce: unwrap_bencode_bytestring(announce, "announce"),
                    creation_date: creation_date.map(|cd| util::bencode_unwrap_number(cd)),
                    created_by: created_by.map(|cb| unwrap_bencode_bytestring(cb,
                                                                              "created_by")),
                    encoding: encoding.map(|enc| unwrap_bencode_bytestring(enc,
                                                                           "encoding")),
                })
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

pub fn parse_torrent_file(torrent_file: &str) -> Result<MetaInfo, ParseError> {
    let mut path = PathBuf::from(TORRENT_FILE_DIR);
    path.push(torrent_file);

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
