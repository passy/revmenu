use nom::{hex_digit, space, Err};
use std::str::from_utf8;
use failure::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct RefLike {
    pub hash: String,
}

named!(
    reflike<RefLike>,
    do_parse!(
        hex: map_res!(map_res!(hex_digit, from_utf8), from_hash) >> (RefLike {
            hash: hex.trim().to_owned(),
        })
    )
);

fn is_not_space(c: u8) -> bool {
    c != b' ' && c != b'\n' && c != b'\t'
}

named!(
    hash<Option<RefLike>>,
    do_parse!(many0!(space) >> c: opt!(reflike) >> take_while!(is_not_space) >> (c))
);

named!(
    entries<Vec<RefLike>>,
    fold_many1!(
        complete!(hash),
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

fn from_hash(input: &str) -> Result<&str, String> {
    if input.len() >= 6 {
        Ok(input)
    } else {
        Err("Doesn't look like a hash".into())
    }
}

pub fn parse(l: &[u8]) -> Result<Vec<RefLike>, Error> {
    match entries(l) {
        Ok((_remaining, value)) => { Ok(value) },
        Err(Err::Incomplete(needed)) => { bail!("Incomplete, needed: {:?}", needed) },
        Err(Err::Error(e)) | Err(Err::Failure(e)) => { bail!("Parsing failure: {:?}", e) },
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_reflike() {
        assert_eq!(super::reflike(&b"deadbeef"[..]), Ok((&b""[..], super::RefLike { hash: "deadbeef".to_string() })));
        assert_eq!(super::reflike(&b"deadberg"[..]), Ok((&b"rg"[..], super::RefLike { hash: "deadbe".to_string() })));
    }

    #[test]
    fn test_hashes() {
        assert_eq!(super::hash(&b"deadbeef"[..]), Ok((&b""[..], Some(super::RefLike { hash: "deadbeef".to_string() }))));
        // Obviously not what we actually want.
        // assert_eq!(super::hash(&b"hello deadbeef"[..]), Ok((&b" deadbeef"[..], None)));
    }
}