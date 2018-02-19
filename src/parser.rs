use nom::{is_hex_digit, Err, hex_digit};
use nom::types::CompleteStr;
use failure::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct RefLike {
    pub hash: String,
}

fn mk_reflike(hash: &str) -> Option<RefLike> {
    if hash.len() >= 6 {
        Some(RefLike { hash: hash.to_owned() })
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
    terminated!(hex_digit, terminator)
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

pub fn parse(l: &str) -> Result<Vec<RefLike>, Error> {
    match tokens(CompleteStr(l)) {
        Ok((_remaining, value)) => { Ok(value) },
        Err(Err::Incomplete(needed)) => { bail!("Incomplete, needed: {:?}", needed) },
        Err(Err::Error(e)) | Err(Err::Failure(e)) => { bail!("Parsing failure: {:?}", e) },
    }
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
        let result = Ok((CompleteStr(""), vec![super::RefLike { hash: "deadbeef".to_string() }, super::RefLike { hash: "aaabbbcccddd".to_string() }]));

        assert_eq!(
            super::tokens(CompleteStr("deadbeef-525\naaabbbcccddd")),
            result
        );

        assert_eq!(
            super::tokens(CompleteStr("deadbeefx525zzzzaaaXXXaaabbbcccddd")),
            result
        );
    }
}