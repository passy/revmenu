use nom::{hex_digit, space, IResult};
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
        IResult::Done(_, v) => Ok(v),
        IResult::Error(e) => Err(format_err!("{}", e)),
        IResult::Incomplete(i) => Err(format_err!("Not enough data: {:?}", i)),
    }
}

#[cfg(test)]
mod tests {
    use nom::IResult::Done;

    #[test]
    fn test_reflike() {
        assert_eq!(super::reflike(&b"deadbeef"[..]), Done(&b""[..], super::RefLike { hash: "deadbeef".to_string() }));
        assert_eq!(super::reflike(&b"deadberg"[..]), Done(&b"rg"[..], super::RefLike { hash: "deadbe".to_string() }));
    }

    #[test]
    fn test_hashes() {
        assert_eq!(super::hash(&b"deadbeef"[..]), Done(&b""[..], Some(super::RefLike { hash: "deadbeef".to_string() })));
        // Obviously not what we actually want.
        assert_eq!(super::hash(&b"hello deadbeef"[..]), Done(&b" deadbeef"[..], None));
    }
}