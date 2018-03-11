#[macro_use]
extern crate clap;
extern crate colored;
extern crate console;
extern crate dialoguer;
extern crate exitcode;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate nom;

use std::io::{stderr, stdin, BufRead, BufReader, Write};
use std::ops::Rem;
use std::process;
use std::fs::File;
use std::iter::Iterator;
use std::process::exit;
use failure::{err_msg, Error};
use console::{Term, Key};
use types::RevLocations;
use itertools::Itertools;
use colored::Colorize;

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

fn highlight_revs<'a>(vlines: &Vec<String>, rls: &RevLocations, selected: Option<&parser::Located<parser::RefLike>>) -> Vec<String> {
    let grouped = rls.iter().group_by(|e| e.line);
    let mut igrouped = grouped.into_iter().peekable();
    let grouped_lines = vlines.iter().enumerate().map(|(vlno, vl)| {
        let matched = if let Some(&(lno, ref _ls)) = igrouped.peek() {
            if lno == vlno {
                true
            } else {
                false
            }
        } else {
            false
        };

        if matched {
            match igrouped.next() {
                None => (vl, vec![]),
                Some((_, group)) => (vl, group.collect()),
            }
        } else {
            (vl, vec![])
        }
    });

    // TODO: Another one for immutable.rs.
    grouped_lines.fold(vec![], |mut acc, (original_line, rlocs)| {
        acc.push(highlight_line(
            original_line,
            &rlocs,
            selected
        ));
        acc
    })
}

fn highlight_line(
    str: &str,
    // FIXME: This type is weird and I don't know why.
    rls: &Vec<&parser::Located<parser::RefLike>>,
    selected: Option<&parser::Located<parser::RefLike>>,
) -> String {
    let (i, res) = rls.iter().fold((0usize, vec![]), |(i, mut acc), &x| {
        let s = x.el.hash.len();
        let j = x.col + s;

        acc.push(str[i..x.col].to_string());
        // TODO: Can we make this a closure of the highlighting method instead?
        if Some(x) == selected {
            acc.push(x.el.hash.yellow().to_string());
        } else {
            acc.push(x.el.hash.magenta().to_string());
        }
        (j, acc)
    });

    format!("{}{}", res.join(""), &str[i..])
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

    if revs.len() == 0 {
        return Ok(exitcode::OK);
    }

    let mut selected = 0usize;
    let term = Term::stderr();
    loop {
        for line in highlight_revs(&lines, &revs, revs.get(selected)) {
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
            },
            Key::Escape => {
                process::exit(exitcode::UNAVAILABLE);
            },
            _ => {}
        }

        term.clear_last_lines(lines.len())?;
    }

    if let Some(rev) = revs.get(selected) {
        vcs_.checkout(&rev.el.hash)?;
        Ok(exitcode::OK)
    } else {
        bail!("Selected unavailable rev.")
    }
}
