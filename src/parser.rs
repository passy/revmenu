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

named!(
    tokens<CompleteStr, Vec<RefLike>>,
    fold_many1!(token, vec![], |mut acc: Vec<RefLike>, item: CompleteStr| {
        match mk_reflike(item.0) {
            Some(r) => { acc.push(r); acc }
            None => acc,
        }
    })
);

pub fn parse_line(l: &str) -> Result<Vec<RefLike>, Error> {
    match tokens(CompleteStr(l)) {
        Ok((_remaining, value)) => Ok(value),
        Err(Err::Incomplete(needed)) => bail!("Incomplete, needed: {:?}", needed),
        Err(Err::Error(e)) | Err(Err::Failure(e)) => bail!("Parsing failure: {:?}", e),
    }
}

pub fn parse(ls: &str) -> Result<Vec<Located<RefLike>>, Error> {
    // Holy mutable son of satan, this needs a refactor.
    let mut tokens: Vec<Located<RefLike>> = vec![];
    let mut offset: usize = 0usize;
    let mut cls = CompleteStr(ls);

    loop {
        match token(cls) {
            Ok((remaining, value)) => {
                if let Some(v) = mk_reflike(value.0) {
                    tokens.push(Located { line: 0, col: offset, el: v });
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

#[cfg(test)]
mod tests {
    use nom::types::CompleteStr;

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
    fn test_tokens() {
        let result = Ok((
            CompleteStr(""),
            vec![
                super::RefLike {
                    hash: "deadbeef".to_string(),
                },
                super::RefLike {
                    hash: "aaabbbcccddd".to_string(),
                },
            ],
        ));

        assert_eq!(
            super::tokens(CompleteStr("deadbeef-525\naaabbbcccddd")),
            result
        );

        assert_eq!(
            super::tokens(CompleteStr("deadbeefx525zzzzaaaXXXaaabbbcccddd")),
            result
        );
    }

    #[test]
    fn test_full_parse() {
        assert_eq!(
            super::parse("deadbeef-525-hello-faceb00c").unwrap(),
            vec![super::Located { el: super::RefLike { hash: "deadbeef".to_string() }, col: 0, line: 0 },
                 super::Located { el: super::RefLike { hash: "faceb00c".to_string() }, col: 19, line: 0 }]
        );
    }
}
