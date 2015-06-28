use bencode::{self, FromBencode, Bencode};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::fmt;
use std::io::{self, Read};

static DEFAULT_TORRENT_FILE: &'static str = "flagfromserver.torrent";
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
    pub fn piece_length(&self) -> u32 {
        self.info.piece_length()
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
    fn piece_length(&self) -> u32;
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

    fn piece_length(&self) -> u32 { self.piece_length }
}

pub struct MultiFileInfo {
    pub piece_length: u32,
    pub pieces: Vec<u8>,

    // name of directory that contains the files
    pub name: String,

    files: Vec<MultipleFileIndividualFileInfo>,
}

// TODO: probably delete this and use Bencode type instead and generalize
// get_field and maybe_get_field
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


pub type DecodeError = String;

#[derive(Debug)]
enum MetaInfoFieldType {
    ByteString(String),
    Number(i64),
}

impl MetaInfoFieldType {
    // panics if its not a ByteString
    pub fn as_string(self) -> String {
        use self::MetaInfoFieldType::*;

        match self {
            ByteString(s) => s,
            _ => panic!("not a ByteString"),
        }
    }

    // panics if its not a Number
    pub fn as_i64(self) -> i64 {
        use self::MetaInfoFieldType::*;

        match self {
            Number(x) => x,
            _ => panic!("not a Number"),
        }
    }
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
        Some(_) => Err(format!("{:?} is in the dict, but not a string or number",
                               String::from_utf8(key.clone().unwrap()))),
        None => Err(format!("{:?} not found in the dict",
                               String::from_utf8(key.clone().unwrap()))),

    }
}

fn maybe_get_field(map: &BTreeMap<bencode::util::ByteString, Bencode>,
             key: &bencode::util::ByteString)
        -> Result<Option<MetaInfoFieldType>, DecodeError> {
    match map.get(key) {
        Some(&Bencode::ByteString(ref s)) => {
            match String::from_utf8(s.clone()) {
                Ok(s) => Ok(Some(MetaInfoFieldType::ByteString(s))),
                Err(e) => Err(format!("Error: {}", e)),
            }
        },
        Some(&Bencode::Number(x)) => Ok(Some(MetaInfoFieldType::Number(x))),
        _ => Ok(None),
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
                let created_by = try!(maybe_get_field(m, created_by_key));
                let creation_date = try!(maybe_get_field(m, creation_date_key));
                let encoding = try!(maybe_get_field(m, encoding_key));

                println!("announce = {:?},\n\
                          creation_date = {:?},\n\
                          created by = {:?},\n\
                          encoding = {:?}",
                          announce, creation_date, created_by, encoding);

                // TODO: this Info is bogus
                let info = SingleFileInfo {
                    piece_length: 0,
                    pieces: Vec::new(),
                    name: String::new(),
                    length: 0,
                    md5sum: None,
                };

                Ok(MetaInfo {
                    info: Box::new(info) as Box<InfoDictionary>,
                    announce: announce.as_string(),
                    creation_date: creation_date.map(|cd| cd.as_i64()),
                    created_by: created_by.map(|cb| cb.as_string()),
                    encoding: encoding.map(|enc| enc.as_string()),
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
