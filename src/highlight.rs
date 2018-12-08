use crate::parser;
use crate::types::RevLocation;
use colored::Colorize;
use im::{catlist, CatList};
use itertools::Itertools;

pub fn revs(
    vlines: &[String],
    rls: &[RevLocation],
    selected: &Option<&parser::Located<parser::RefLike>>,
) -> CatList<String> {
    let grouped = rls.iter().group_by(|e| e.line);
    let mut igrouped = grouped.into_iter().peekable();
    let grouped_lines = vlines.iter().enumerate().map(|(vlno, vl)| {
        let matched = if let Some(&(lno, ref _ls)) = igrouped.peek() {
            lno == vlno
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

    grouped_lines.fold(catlist![], |acc, (original_line, rlocs)| {
        acc.snoc(line(original_line, rlocs, selected))
    })
}

fn line<'a, I>(str: &str, rls: I, selected: &Option<&RevLocation>) -> String
where
    I: IntoIterator<Item = &'a parser::Located<parser::RefLike>>,
{
    let (i, res) = rls.into_iter().fold((0_usize, catlist![]), |(i, acc), x| {
        let s = x.el.hash.len();
        let j = x.col + s;

        // TODO: Can we make this a closure of the highlighting method instead?
        let el = if &Some(x) == selected {
            x.el.hash.yellow().to_string()
        } else {
            x.el.hash.magenta().to_string()
        };

        (j, acc.snoc(str[i..x.col].to_string()).snoc(el))
    });

    format!("{}{}", res.iter().join(""), &str[i..])
}

#[cfg(test)]
mod tests {
    use crate::parser;

    fn mk_located(hash: &str, col: usize) -> parser::Located<parser::RefLike> {
        parser::Located {
            el: parser::RefLike {
                hash: hash.to_string(),
            },
            col: col,
            line: 0,
        }
    }

    #[test]
    fn test_highlight_line() {
        let testline = "deadbeef-525-hello-faceb00c";
        let revs = vec![mk_located("deadbeef", 0), mk_located("faceb00c", 19)];
        assert_eq!(
            super::line(&testline, &revs, &None),
            "\u{1b}[35mdeadbeef\u{1b}[0m-525-hello-\u{1b}[35mfaceb00c\u{1b}[0m"
        );
    }

    #[test]
    fn test_highlight_select_line() {
        let testline = "deadbeef-525-hello-faceb00c";
        let revs = vec![mk_located("deadbeef", 0), mk_located("faceb00c", 19)];
        assert_eq!(
            super::line(&testline, &revs, &revs.get(0)),
            "\u{1b}[33mdeadbeef\u{1b}[0m-525-hello-\u{1b}[35mfaceb00c\u{1b}[0m"
        );
    }

    #[test]
    fn test_highlight_nothing() {
        let testline = "deadbeef-525-hello-faceb00c";
        let revs = vec![];
        assert_eq!(super::line(&testline, &revs, &None), testline);
    }
}
