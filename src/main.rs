#[macro_use]
extern crate clap;
extern crate console;
extern crate exitcode;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate nom;

use std::io::{stderr, stdin, BufRead, BufReader, Write};
use std::fs::File;
use std::process::exit;
use failure::{err_msg, Error};

mod cli;
mod parser;

fn main() {
    match run() {
        Err(e) => {
            let stderr = &mut stderr();
            let errmsg = "Error writing to stderr.";
            writeln!(stderr, "{}", e).expect(errmsg);
            exit(1)
        }
        Ok(r) => exit(r),
    }
}

fn run() -> Result<exitcode::ExitCode, Error> {
    let args = cli::cli().get_matches();

    let file_val = args.value_of("FILE")
        .ok_or_else(|| err_msg("Expected FILE."))?;
    let reader: Box<BufRead> = if file_val == "-" {
        Box::new(BufReader::new(stdin()))
    } else {
        let file = File::open(file_val)?;
        Box::new(BufReader::new(file))
    };

    let hashes = reader.lines().filter_map(|line| {
        line.map_err(|e| e.into()).and_then(|l| parser::parse(l.as_bytes())).ok()
    }).flat_map(|a| a);

    println!("Found the following hashes: ");
    for h in hashes {
        println!("{}", h.hash);
    }

    Ok(exitcode::OK)
}
