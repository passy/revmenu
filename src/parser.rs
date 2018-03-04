use nom::{hex_digit, is_hex_digit, Err, Offset};
use nom::types::CompleteStr;
use failure::Error;

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
    token<CompleteStr, CompleteStr>,
    terminated!(hex_digit, alt!(eof!() | terminator))
);

pub fn parse_line(ls: &str, row: usize) -> Result<Vec<Located<RefLike>>, Error> {
    // Holy mutable son of satan, this needs a refactor.
    let mut tokens: Vec<Located<RefLike>> = vec![];
    let mut offset: usize = 0usize;
    let mut cls = CompleteStr(ls);

    loop {
        match token(cls) {
            Ok((remaining, value)) => {
                if let Some(v) = mk_reflike(value.0) {
                    tokens.push(Located { line: row, col: offset, el: v });
                }
                offset += cls.offset(&remaining);

                if remaining.0.is_empty() {
                    break;
                } else {
                    cls = remaining;
                }
            },
            Err(Err::Incomplete(needed)) => bail!("Incomplete, needed: {:?}", needed),
            Err(Err::Error(e)) | Err(Err::Failure(e)) => bail!("Parsing failure: {:?}", e),
        }
    }
    Ok(tokens)
}

pub fn parse_bufread<T>(reader: T) -> Vec<Located<RefLike>>
    where T: ::std::io::BufRead {
    reader
        .lines()
        .enumerate()
        .filter_map(|(lineno, line)| {
            line.map_err(|e| e.into())
                .and_then(|l| parse_line(&l, lineno))
                .ok()
        })
        .flat_map(|a| a)
        .collect()
}

#[cfg(test)]
mod tests {
    use nom::types::CompleteStr;
    use std::io::BufRead;

    fn mk_located(hash: &str, col: usize, line: usize) -> super::Located<super::RefLike> {
        super::Located { el: super::RefLike { hash: hash.to_string() }, col: col, line: line }
    }

    #[test]
    fn test_token() {
        assert_eq!(
            super::token(CompleteStr("deadbeef")),
            Ok((CompleteStr(""), CompleteStr("deadbeef")))
        );

        assert_eq!(
            super::token(CompleteStr("deadbeef-faceb00c")),
            Ok((CompleteStr("faceb00c"), CompleteStr("deadbeef")))
        );
    }

    #[test]
    fn test_full_parse_line() {
        assert_eq!(
            super::parse_line("deadbeef-525-hello-faceb00c", 0).unwrap(),
            vec![mk_located("deadbeef", 0, 0),
                 mk_located("faceb00c", 19, 0)]);
    }
}
