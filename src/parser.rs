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

#[derive(Debug)]
struct TokenParser<'a> {
    input: CompleteStr<'a>,
    remaining: CompleteStr<'a>,
    offset: usize,
}

fn token_parser<'a>(ls: &'a str) -> TokenParser<'a> {
    let cs = CompleteStr(ls);
    TokenParser { input: cs, remaining: cs, offset: 0 }
}

impl<'a> Iterator for TokenParser<'a> {
    type Item = Result<(usize, CompleteStr<'a>), Error>;

    fn next(&mut self) -> Option<Result<(usize, CompleteStr<'a>), Error>> {
        match token(self.remaining) {
            Ok((remaining, value)) => {
                self.offset += self.input.offset(&remaining);

                if remaining.0.is_empty() {
                    None
                } else {
                    self.remaining = remaining;
                    Some(Ok((self.offset, value)))
                }
            },
            Err(Err::Incomplete(needed)) => Some(Err(format_err!("Incomplete, needed: {:?}", needed))),
            Err(Err::Error(e)) | Err(Err::Failure(e)) => Some(Err(format_err!("Parsing failure: {:?}", e))),
        }
    }
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

pub fn parse_(ls: &str) -> Result<Vec<Located<RefLike>>, Error> {
    token_parser(ls).fold(Ok(vec![]), |acc, res| {
        match acc {
            Err(_) => acc,
            Ok(mut tokens) => {
                match res {
                    Err(err) => Err(err),
                    Ok((offset, token)) => {
                        match mk_reflike(token.0) {
                            Some(reflike) => {
                                tokens.push(Located { el: reflike, line: 0, col: offset });
                                Ok(tokens)
                            },
                            None => Ok(tokens),
                        }
                    },
                }
            }
        }
    })
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
    fn test_full_parse() {
        assert_eq!(
            super::parse("deadbeef-525-hello-faceb00c").unwrap(),
            vec![super::Located { el: super::RefLike { hash: "deadbeef".to_string() }, col: 0, line: 0 },
                 super::Located { el: super::RefLike { hash: "faceb00c".to_string() }, col: 19, line: 0 }]
        );
    }
}
