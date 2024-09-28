use nom::{
    bytes::complete::{tag, take, take_until}, combinator::map_res, error::make_error, number::complete::le_u16, sequence::terminated, IResult
};
use nom::number::complete::u8;

const NULL_PATTERN: &[u8] = &[0];
const DE_STRING_MARKER: &[u8] = &[0x60, 0x0A];

pub fn cstring(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, cstr) = map_res(
        terminated(take_until(NULL_PATTERN), tag(NULL_PATTERN)),
        |game_version_bytes| {
            core::str::from_utf8(game_version_bytes).map_err(|_| {
                nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Fail))
            })
        },
    )(input)?;

    Ok((input, cstr))
}

pub fn de_string(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, _) = tag(DE_STRING_MARKER)(input)?;
    let (input, length) = le_u16(input)?;
    let (input, value) = map_res(take(length), |game_version_bytes| {
        core::str::from_utf8(game_version_bytes).map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Fail))
        })
    })(input)?;
    Ok((input, value))
}

pub fn flag(input: &[u8]) -> IResult<&[u8], bool> {
    let (input, value) = u8(input)?;
    let flag = match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => {
            println!("Value: {value}");
            Err(nom::Err::Error(make_error(input, nom::error::ErrorKind::Fail)))
            //Ok(true)
        }
    }?;
    Ok((input, flag))
}
