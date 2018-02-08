extern crate console;
extern crate exitcode;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate nom;

use std::io::{stderr, stdin, Write, Read, BufReader, BufRead};
use std::fs::File;
use std::process::{exit};
use failure::{Error, err_msg};

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
        Ok(r) => exit(r)
    }
}

fn run() -> Result<exitcode::ExitCode, Error> {
    let args = cli::cli().get_matches();

    let file_val = args.value_of("FILE").ok_or_else(|| err_msg("Expected FILE."))?;
    let reader: Box<BufRead> = if file_val == "-" {
        Box::new(BufReader::new(stdin()))
    } else {
        let file = File::open(file_val)?;
        Box::new(BufReader::new(file))
    };

    for line in reader.lines() {
        let res = line.map(|l| parser::parse(l.as_bytes()));
        println!("res: {:?}", res);

    }

    Ok(exitcode::OK)
}