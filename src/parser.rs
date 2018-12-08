use failure::Error;
use nom::types::CompleteStr;
use nom::{hex_digit, is_hex_digit, Err, Offset};

#[derive(Debug, PartialEq, Eq)]
pub struct RefLike {
    pub hash: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Located<A> {
    pub el: A,
    pub col: usize,
    pub line: usize,
}

fn mk_reflike(hash: &str) -> Option<RefLike> {
    if hash.len() >= 6 {
        Some(RefLike {
            hash: hash.to_owned(),
        })
    } else {
        None
    }
}

fn is_hex_digit_char(c: char) -> bool {
    is_hex_digit(c as u8)
}

named!(
    terminator<CompleteStr, CompleteStr>,
    take_while!(|c| !is_hex_digit_char(c))
);

named!(
    token<CompleteStr, Option<CompleteStr>>,
    terminated!(opt!(hex_digit), alt!(eof!() | terminator))
);

pub fn parse_line(ls: &str, row: usize) -> Result<Vec<Located<RefLike>>, Error> {
    // Holy mutable son of satan, this needs a refactor.
    let mut tokens: Vec<Located<RefLike>> = vec![];
    let mut offset: usize = 0_usize;
    let mut cls = CompleteStr(ls);

    while !cls.0.is_empty() {
        match token(cls) {
            Ok((remaining, None)) => {
                offset += cls.offset(&remaining);
                cls = remaining;
            }
            Ok((remaining, Some(value))) => {
                if let Some(v) = mk_reflike(value.0) {
                    tokens.push(Located {
                        line: row,
                        col: offset,
                        el: v,
                    });
                }
                offset += cls.offset(&remaining);
                cls = remaining;
            }
            Err(Err::Incomplete(needed)) => {
                bail!("Incomplete, needed: {:?}", needed);
            }
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                bail!("Parsing failure: {:?}", e);
            }
        }
    }
    Ok(tokens)
}

#[allow(unused)]
pub fn parse_bufread<T>(reader: T) -> Vec<Located<RefLike>>
where
    T: ::std::io::BufRead,
{
    // Pretty sure that this is a terrible idea, but I don't feel like
    // rewriting my tests right now.
    let lines: Vec<_> = reader.lines().filter_map(|l| l.ok()).collect();
    parse_lines(lines.iter())
}

pub fn parse_lines<'a, I>(lines: I) -> Vec<Located<RefLike>>
where
    I: Iterator<Item = &'a String>,
{
    lines
        .enumerate()
        .filter_map(|(lineno, line)| parse_line(line, lineno).ok())
        .flat_map(|a| a)
        .collect()
}

#[cfg(test)]
mod tests {
    use nom::types::CompleteStr;
    fn mk_located(hash: &str, col: usize, line: usize) -> super::Located<super::RefLike> {
        super::Located {
            el: super::RefLike {
                hash: hash.to_string(),
            },
            col: col,
            line: line,
        }
    }

    #[test]
    fn test_token() {
        assert_eq!(
            super::token(CompleteStr("deadbeef")),
            Ok((CompleteStr(""), Some(CompleteStr("deadbeef"))))
        );

        assert_eq!(
            super::token(CompleteStr("deadbeef-faceb00c")),
            Ok((CompleteStr("faceb00c"), Some(CompleteStr("deadbeef"))))
        );
    }

    #[test]
    fn test_full_parse_line() {
        assert_eq!(
            super::parse_line("deadbeef-525-hello-faceb00c", 0).unwrap(),
            vec![mk_located("deadbeef", 0, 0), mk_located("faceb00c", 19, 0)]
        );
    }

    #[test]
    fn test_regression_split_line() {
        assert_eq!(
            super::parse_line("   Compiling dialoguer v0.1.0 (https://github.com/mitsuhiko/dialoguer?rev=5f28d3d74768b6ba532866ee3c83df9324f9df06#5f28d3d7)", 0).unwrap(),
            vec![mk_located("5f28d3d74768b6ba532866ee3c83df9324f9df06", 74, 0), mk_located("5f28d3d7", 115, 0)]
        );
    }

    #[test]
    fn test_full_parse_lines() {
        let str =
            "hello deadbeef\nlorem ipsum\r\ndolor 9d393a816701d3e74f268f3b6c3f6ff43f25e811 sup\n";
        let cursor = ::std::io::Cursor::new(str);

        assert_eq!(
            super::parse_bufread(cursor),
            vec![
                mk_located("deadbeef", 6, 0),
                mk_located("9d393a816701d3e74f268f3b6c3f6ff43f25e811", 6, 2),
            ]
        );
    }
}
