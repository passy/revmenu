use nom::{hex_digit, space, IResult};
use std::str::from_utf8;
use failure::Error;

#[derive(Debug)]
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
    line<Option<RefLike>>,
    do_parse!(many0!(space) >> c: opt!(reflike) >> take_while!(is_not_space) >> (c))
);

named!(
    entries<Vec<RefLike>>,
    fold_many1!(
        complete!(line),
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
