use nom::{hex_digit, space, IResult};
use std::str::from_utf8;
use failure::{err_msg, Error};

#[derive(Debug)]
pub struct RefLike {
    pub hash: String,
}

named!(
    reflike<RefLike>,
    do_parse!(
        hex: map_res!(map_res!(hex_digit, from_utf8), from_hash) >>
        (
            RefLike {
                hash: hex.trim().to_owned(),
            }
        )
    )
);

named!(
    line<Option<RefLike>>,
    do_parse!(
        s0: many0!(space) >>
        c: opt!(reflike) >>
        s1: take_until_and_consume!(" ") >>
        (c)
    )
);

named!(
    entries<Vec<RefLike>>,
    fold_many1!(line, Vec::default(), |mut acc: Vec<RefLike>, i| {
        match i {
            Some(l) => { acc.push(l); acc },
            None => acc
        }
    })
);

fn from_hash(input: &str) -> Result<&str, String> {
    println!("input: {}", input);
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
        IResult::Incomplete(_) => Err(err_msg("Not enough data")),
    }
}
