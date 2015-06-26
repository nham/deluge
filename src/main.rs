extern crate bencode;
extern crate getopts;

use getopts::Options;
use std::env;

mod metainfo;

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

    let metainfo = metainfo::parse_torrent_file(torrent_file.as_ref()
                                                            .map(|s| &s[..]));

    println!("metainfo = {:?}", metainfo);

    println!("Hello, universe! torrent_file = {:?}", torrent_file);
}
