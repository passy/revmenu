// Enable clippy if our Cargo.toml file asked us to do so.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Enable as many useful Rust and Clippy warnings as we can stand.  We'd
// also enable `trivial_casts`, but we're waiting for
// https://github.com/rust-lang/rust/issues/23416.
#![warn(missing_copy_implementations,
        missing_debug_implementations,
        trivial_casts,
        trivial_numeric_casts,
        unsafe_code,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications)]
#![deny(bare_trait_objects,
        anonymous_parameters,
)]
#![cfg_attr(feature="clippy", warn(cast_possible_wrap))]
#![cfg_attr(feature="clippy", warn(cast_precision_loss))]
#![cfg_attr(feature="clippy", warn(mut_mut))]
// This allows us to use `unwrap` on `Option` values (because doing makes
// working with Regex matches much nicer) and when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test), feature="clippy"), warn(result_unwrap_used))]
#![cfg_attr(feature="clippy", warn(unseparated_literal_suffix))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]

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
use failure::{err_msg, Error};
use console::{Key, Term};
use types::RevLocation;

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
            process::exit(1)
        }
        Ok(r) => process::exit(r),
    }
}

fn select(term: &Term, lines: &[String], revs: &[RevLocation]) -> Result<Option<usize>, Error> {
    let mut selected = 0_usize;

    loop {
        for line in highlight::revs(lines, revs, &revs.get(selected)) {
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
    let reader: Box<dyn BufRead> = if file_val == "-" {
        Box::new(BufReader::new(stdin()))
    } else {
        let file = File::open(file_val)?;
        Box::new(BufReader::new(file))
    };

    let lines: Vec<String> = reader.lines().filter_map(|f| f.ok()).collect();
    let term = Term::stderr();

    // If we can get the terminal size, truncate to the last (h - 1) lines.
    let truncated_lines = match term.size_checked() {
        Some((h, _w)) => {
            let len = lines.len();
            let start = len - std::cmp::min(h as usize, len);
            let end = std::cmp::max(len - 1, start);
            lines[start..end].into()
        },
        None => lines
    };

    let cwd = std::env::current_dir()?;
    let vcs_ = vcs::detect_vcs(&cwd)?;
    let revs: Vec<RevLocation> = parser::parse_lines(truncated_lines.iter());

    if revs.is_empty() {
        return Ok(exitcode::OK);
    }

    let selected = match select(&term, &truncated_lines, &revs)? {
        Some(s) => s,
        None => return Ok(exitcode::OK),
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
