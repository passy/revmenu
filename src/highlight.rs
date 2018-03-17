use itertools::Itertools;
use colored::Colorize;
use types::RevLocations;
use parser;

pub fn revs<'a>(
    vlines: &Vec<String>,
    rls: &RevLocations,
    selected: Option<&parser::Located<parser::RefLike>>,
) -> Vec<String> {
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
        acc.push(line(original_line, &rlocs, &selected));
        acc
    })
}

fn line(
    str: &str,
    rls: &Vec<&parser::Located<parser::RefLike>>,
    selected: &Option<&parser::Located<parser::RefLike>>,
) -> String {
    let (i, res) = rls.iter().fold((0usize, vec![]), |(i, mut acc), &x| {
        let s = x.el.hash.len();
        let j = x.col + s;

        acc.push(str[i..x.col].to_string());
        // TODO: Can we make this a closure of the highlighting method instead?
        let el = if &Some(x) == selected {
            x.el.hash.yellow().to_string()
        } else {
            x.el.hash.magenta().to_string()
        };

        // TODO: Use immutable.rs. This is gross.
        acc.push(el);
        (j, acc)
    });

    format!("{}{}", res.join(""), &str[i..])
}