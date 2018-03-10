#[macro_use]
extern crate clap;
extern crate console;
extern crate dialoguer;
extern crate exitcode;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate nom;

use std::io::{stderr, stdin, BufRead, BufReader, Write};
use std::fs::File;
use std::iter::Iterator;
use std::process::exit;
use failure::{err_msg, Error};
use dialoguer::Select;
use types::RevLocations;
use itertools::Itertools;

mod cli;
mod parser;
mod vcs;
mod types;

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

fn highlight_revs<'a>(vlines: &Vec<String>, rls: &RevLocations) {
    let grouped = rls.iter().group_by(|e| e.line);

    for (lno, line) in &grouped {
        let vl = &vlines[lno];
        println!("{:?}", &vl[0..10]);
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
    let lines: Vec<String> = reader.lines().filter_map(|f| f.ok()).collect();

    let cwd = std::env::current_dir()?;
    let vcs_ = vcs::detect_vcs(&cwd)?;

    let revs: RevLocations = parser::parse_lines(lines.iter());

    // TODO: Use location info.
    // let hashes: Vec<String> = revs.into_iter().map(|r| r.el.hash).collect();

    // if hashes.len() == 0 {
    //     return Ok(exitcode::OK);
    // }

    highlight_revs(&lines, &revs);

    // let selection = Select::new()
    //     .default(0)
    //     .items(&hashes.as_slice())
    //     .interact()
    //     .unwrap();

    // let selected_hash = &hashes.get(selection);

    // if let &Some(h) = selected_hash {
    //     println!("Checking out {} revision: {}", vcs_.name(), h);
    //     vcs_.checkout(h)?;
    // }

    Ok(exitcode::OK)
}
