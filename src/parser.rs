use nom::{is_hex_digit, space, Err};
use nom::types::CompleteStr;
use failure::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct RefLike {
    pub hash: String,
}

fn is_hex_digit_char(c: char) -> bool {
    is_hex_digit(c as u8)
}

named!(
    reflike<CompleteStr, RefLike>,
    do_parse!(
        hex: map_res!(take_while!(is_hex_digit_char), from_hash) >> (RefLike {
            hash: hex.0.trim().to_owned(),
        })
    )
);

fn is_not_space(c: char) -> bool {
    c != ' ' && c != '\n' && c != '\t'
}

named!(
    hash<CompleteStr, Option<RefLike>>,
    do_parse!(many0!(space) >> c: opt!(reflike) >> take_while!(is_not_space) >> (c))
);

named!(
    entries<CompleteStr, Vec<RefLike>>,
    fold_many1!(
        hash,
        Vec::default(),
        |mut acc: Vec<RefLike>, i| match i {
            Some(l) => {
                acc.push(l);
                acc
            }
            None => acc,
        }
    )
);

fn from_hash(input: CompleteStr) -> Result<CompleteStr, String> {
    if input.0.len() >= 6 {
        Ok(input)
    } else {
        Err("Doesn't look like a hash".into())
    }
}

pub fn parse(l: &str) -> Result<Vec<RefLike>, Error> {
    match entries(CompleteStr(l)) {
        Ok((_remaining, value)) => { Ok(value) },
        Err(Err::Incomplete(needed)) => { bail!("Incomplete, needed: {:?}", needed) },
        Err(Err::Error(e)) | Err(Err::Failure(e)) => { bail!("Parsing failure: {:?}", e) },
    }
}

#[cfg(test)]
mod tests {
    use nom::types::CompleteStr;

    #[test]
    fn test_reflike() {
        assert_eq!(
            super::reflike(CompleteStr("deadbeef")),
            Ok((CompleteStr(""), super::RefLike { hash: "deadbeef".to_string() }))
        );
        assert_eq!(
            super::reflike(CompleteStr("deadberg")),
             Ok((CompleteStr("rg"), super::RefLike { hash: "deadbe".to_string() }))
        );
    }

    #[test]
    fn test_hashes() {
        assert_eq!(
            super::hash(CompleteStr("deadbeef")),
            Ok((CompleteStr(""), Some(super::RefLike { hash: "deadbeef".to_string() }))));
        // Obviously not what we actually want, but a good point of reference for future work.
        assert_eq!(
            super::hash(CompleteStr("hello deadbeef")),
            Ok((CompleteStr(" deadbeef"), None))
        );
    }
}