#[macro_use]
extern crate clap;
extern crate colored;
extern crate console;
extern crate exitcode;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate im;

use std::io::{stderr, stdin, BufRead, BufReader, Write};
use std::ops::Rem;
use std::process;
use std::fs::File;
use std::iter::Iterator;
use std::process::exit;
use failure::{err_msg, Error};
use console::{Key, Term};
use types::RevLocations;

mod cli;
mod parser;
mod vcs;
mod types;
mod highlight;

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

fn select(term: &Term, lines: &Vec<String>, revs: &RevLocations) -> Result<Option<usize>, Error> {
    let mut selected = 0usize;

    loop {
        for line in highlight::revs(&lines, &revs, revs.get(selected)) {
            term.write_line(&line)?;
        }

        match term.read_key()? {
            Key::ArrowDown | Key::ArrowRight | Key::Char('j') => {
                selected = (selected as u64 + 1).rem(revs.len() as u64) as usize;
            }
            Key::ArrowUp | Key::ArrowLeft | Key::Char('k') => {
                selected = ((selected as i64 - 1 + revs.len() as i64) % revs.len() as i64) as usize;
            }
            Key::Enter => {
                term.clear_last_lines(lines.len())?;
                break;
            }
            Key::Escape => return Ok(None),
            _ => {}
        }

        term.clear_last_lines(lines.len())?;
    }

    Ok(Some(selected))
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
    let term = Term::stderr();

    // If we can get the terminal size, truncate to the last (h - 1) lines.
    // (-1) because tmux inserts an annoying newline which we cannot avoid.
    let truncated_lines = match term.size_checked() {
        Some((h, _w)) => {
            lines[(lines.len() - ((h - 1) as usize))..lines.len() - 1].into()
        },
        None => lines
    };

    let cwd = std::env::current_dir()?;
    let vcs_ = vcs::detect_vcs(&cwd)?;
    let revs: RevLocations = parser::parse_lines(truncated_lines.iter());

    if revs.is_empty() {
        return Ok(exitcode::OK);
    }

    let selected = match select(&term, &truncated_lines, &revs)? {
        Some(s) => s,
        None => process::exit(exitcode::OK),
    };

    if let Some(rev) = revs.get(selected) {
        vcs_.checkout(&rev.el.hash)?;
        println!(
            "Checking out revision {} with {} ...",
            &rev.el.hash,
            vcs_.name()
        );
        Ok(exitcode::OK)
    } else {
        bail!("Selected unavailable rev.")
    }
}
