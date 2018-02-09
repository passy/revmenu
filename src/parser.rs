use nom::{digit, hex_digit, rest, space, anychar, IResult};
use std::str::from_utf8;
use failure::{err_msg, Error};

#[derive(Debug)]
pub struct RefLike {
    pub hash: String,
}

named!(
    reflike<RefLike>,
    do_parse!(
        hex: map_res!(hex_digit, from_utf8) >>
        hash: expr_opt!(maybe_hash(hex)) >>
        (
            RefLike {
                hash: hash.trim().to_owned(),
            }
        )
    )
);

named!(
    line<RefLike>,
    alt!(
        reflike | map_opt!(many0!(anychar), |_| None)
    )
);

fn maybe_hash<'a>(input: &'a str) -> Option<&'a str> {
    if input.len() >= 6 {
        Some(input)
    } else {
        None
    }
}

pub fn parse(l: &[u8]) -> Result<RefLike, Error> {
    match reflike(l) {
        IResult::Done(_, v) => {
            Ok(v)
        }
        IResult::Error(e) =>
            Err(format_err!("{}", e)),
        IResult::Incomplete(_) => Err(err_msg("Not enough data")),
    }
}