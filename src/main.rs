#[macro_use]
extern crate clap;
extern crate console;
extern crate exitcode;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate nom;
extern crate dialoguer;

use std::io::{stderr, stdin, BufRead, BufReader, Write};
use std::fs::File;
use std::process::exit;
use failure::{err_msg, Error};
use dialoguer::Select;

mod cli;
mod parser;
mod vcs;

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

    let cwd = std::env::current_dir()?;
    let vcs_ = vcs::detect_vcs(&cwd)?;

    let revs = reader
        .lines()
        .filter_map(|line| {
            line.map_err(|e| e.into())
                .and_then(|l| parser::parse(l.as_bytes()))
                .ok()
        })
        .flat_map(|a| a);

    let hashes: Vec<String> = revs.map(|r| r.hash).collect();
    let selection = Select::new()
        .default(0)
        .items(&hashes.as_slice())
        .interact()
        .unwrap();

    let selected_hash = &hashes.get(selection);

    if let &Some(h) = selected_hash {
        println!("Checking out {} revision: {}", vcs_.name(), h);
        vcs_.checkout(h)?;
    }

    Ok(exitcode::OK)
}
