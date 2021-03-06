#![feature(ip_addr, read_exact)]

extern crate bencode;
extern crate getopts;
extern crate hyper;
extern crate openssl;
extern crate rand;
extern crate url;

use getopts::Options;
use rand::Rng;
use std::env;

mod download;
mod metainfo;
mod tracker;
mod util;

//static DEFAULT_TORRENT_FILE: &'static str = "Fedora-Live-LXDE-x86_64-22.torrent";
//static DEFAULT_TORRENT_FILE: &'static str = "archlinux-2015.06.01-dual.iso.torrent";
//static DEFAULT_TORRENT_FILE: &'static str = "flagfromserver.torrent";
static DEFAULT_TORRENT_FILE: &'static str = "ubuntu-15.04-desktop-amd64.iso.torrent";
//static DEFAULT_TORRENT_FILE: &'static str = "debian-8.2.0-amd64-netinst.iso.torrent";

const PEER_ID_PREFIX: &'static str = "-NH0001-";

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("t", "", "set torrent file name", "NAME");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let torrent_file = matches.opt_str("t");

    if !matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }

    println!("torrent_file = {:?}", torrent_file);

    let torrent_filename = match torrent_file {
        Some(ref file) => &file[..],
        None => DEFAULT_TORRENT_FILE,
    };

    match run(torrent_filename) {
        Err(e) => panic!("Error running: {:?}", e),
        _ => {},
    }
}

#[derive(Debug)]
enum RunError {
    FileError(metainfo::ParseError),
    TrackerError(tracker::TrackerError),
}

impl From<metainfo::ParseError> for RunError {
    fn from(e: metainfo::ParseError) -> RunError {
        RunError::FileError(e)
    }
}

impl From<tracker::TrackerError> for RunError {
    fn from(e: tracker::TrackerError) -> RunError {
        RunError::TrackerError(e)
    }
}

fn run(filename: &str) -> Result<(), RunError> {
    let metainfo = try!(metainfo::parse_torrent_file(filename));
    println!("metainfo = {:?}", metainfo);

    // send GET to tracker
    let peer_id = gen_peer_id();
    let peers = try!(tracker::get_tracker(&metainfo, peer_id.clone()));
    println!("peers.len() = {}", peers.len());

    download::download(&metainfo, &peers[..], peer_id.clone());
    Ok(())
}


fn gen_peer_id() -> String {
    let mut rng = rand::thread_rng();
    let prefix_len = PEER_ID_PREFIX.len();
    let s: String = rng.gen_ascii_chars().take(20 - prefix_len).collect();
    format!("{}{}", PEER_ID_PREFIX, s)
}
