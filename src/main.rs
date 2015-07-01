#![feature(ip_addr)]

extern crate bencode;
extern crate getopts;
extern crate hyper;
extern crate openssl;
extern crate url;

use getopts::Options;
use std::env;

mod metainfo;
mod tracker;
mod util;

//static DEFAULT_TORRENT_FILE: &'static str = "Fedora-Live-LXDE-x86_64-22.torrent";
//static DEFAULT_TORRENT_FILE: &'static str = "archlinux-2015.06.01-dual.iso.torrent";
static DEFAULT_TORRENT_FILE: &'static str = "flagfromserver.torrent";

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

    let torrent_file_name = match torrent_file {
        Some(ref file) => &file[..],
        None => DEFAULT_TORRENT_FILE,
    };

    let metainfo = metainfo::parse_torrent_file(torrent_file_name);

    // try to unwrap metainfo
    let metainfo = match metainfo {
        Ok(metainfo) => metainfo,
        Err(e) => return panic!("Error unwrapping metainfo: {:?}", e),
    };
    println!("metainfo = {:?}", metainfo);
    match tracker::get_tracker(&metainfo) {
        Ok(peers) => println!("peers.len() = {}", peers.len()),
        Err(e) => return panic!("Error calling get_tracker: {:?}", e),
    }
}
