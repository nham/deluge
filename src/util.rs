use bencode::{self, Bencode};
use std::collections::BTreeMap;

pub fn bencode_unwrap_number(b: Bencode) -> i64 {
    match b {
        Bencode::Number(x) => x,
        _ => panic!("Failed to unwrap, Bencoded value is not a Number."),
    }
}

pub fn bencode_unwrap_bytestring(b: Bencode) -> Vec<u8> {
    match b {
        Bencode::ByteString(v) => v,
        _ => panic!("Failed to unwrap, Bencoded value is not a ByteString."),
    }
}


// panics if key isn't found
pub fn get_field(map: &BTreeMap<bencode::util::ByteString, Bencode>,
             key: &bencode::util::ByteString)
        -> Bencode {
    map.get(key).unwrap().clone()
}

pub fn maybe_get_field(map: &BTreeMap<bencode::util::ByteString, Bencode>,
             key: &bencode::util::ByteString)
        -> Option<Bencode>  {
    map.get(key).map(|b| b.clone())
}
