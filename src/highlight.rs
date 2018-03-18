use itertools::Itertools;
use colored::Colorize;
use types::RevLocations;
use im::List;
use parser;

pub fn revs<'a>(
    vlines: &Vec<String>,
    rls: &RevLocations,
    selected: Option<&parser::Located<parser::RefLike>>,
) -> List<String> {
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

    grouped_lines.fold(list![], |acc, (original_line, rlocs)| {
        acc.snoc(line(original_line, &rlocs, &selected))
    })
}

fn line(
    str: &str,
    rls: &Vec<&parser::Located<parser::RefLike>>,
    selected: &Option<&parser::Located<parser::RefLike>>,
) -> String {
    let (i, res) = rls.iter().fold((0usize, list![]), |(i, acc), &x| {
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
    use parser;

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
            super::line(&testline, &revs.iter().collect(), &None),
            "\u{1b}[35mdeadbeef\u{1b}[0m-525-hello-\u{1b}[35mfaceb00c\u{1b}[0m"
        );
    }

    #[test]
    fn test_highlight_select_line() {
        let testline = "deadbeef-525-hello-faceb00c";
        let revs = vec![mk_located("deadbeef", 0), mk_located("faceb00c", 19)];
        assert_eq!(
            super::line(&testline, &revs.iter().collect(), &revs.get(0)),
            "\u{1b}[33mdeadbeef\u{1b}[0m-525-hello-\u{1b}[35mfaceb00c\u{1b}[0m"
        );
    }

    #[test]
    fn test_highlight_nothing() {
        let testline = "deadbeef-525-hello-faceb00c";
        let revs = vec![];
        assert_eq!(
            super::line(&testline, &revs.iter().collect(), &None),
            testline
        );
    }
}
