use nom::{digit, hex_digit, rest, space, anychar, newline, IResult};
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
        space: many1!(space) >>
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
        (c)
    )
);

named!(
    entries<Vec<Option<RefLike>>>,
    do_parse!(
        entries: many1!(line) >> (entries)
    )
);

fn from_hash<'a>(input: &'a str) -> Result<&'a str, String> {
    println!("input: {}", input);
    if input.len() >= 6 {
        Ok(input)
    } else {
        Err(format!("Doesn't look like a hash"))
    }
}

pub fn parse(l: &[u8]) -> Result<Vec<Option<RefLike>>, Error> {
    match entries(l) {
        IResult::Done(_, v) => {
            Ok(v)
        }
        IResult::Error(e) =>
            Err(format_err!("{}", e)),
        IResult::Incomplete(_) => Err(err_msg("Not enough data")),
    }
}