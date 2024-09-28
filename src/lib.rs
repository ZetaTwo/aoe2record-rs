use flate2::bufread::ZlibDecoder;
use flate2::Decompress;
use nom::{
    bytes::complete::take,
    combinator::{cond, map_res, peek},
    error::make_error,
    number::complete::{le_f32, le_u32},
    IResult,
};
use std::io::{self, prelude::*};

mod de;
mod primitives;

fn compute_save_version(old_save_version: f32, new_save_version: Option<u32>) -> f32 {
    match new_save_version {
        None => old_save_version,
        Some(new_save_version) => {
            if new_save_version == 37 {
                37.0f32
            } else {
                // round(new_version / (1 << 16), 2)
                (new_save_version as f32) / ((1 << 16) as f32)
            }
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct CompressedHeader {
    pub game_version: String,
    pub old_save_version: f32,
    pub new_save_version: Option<u32>,
    pub save_version: f32,
    pub hd: Option<u8>, // TODO: change to Option<HdHeader>
    pub de: Option<de::DeHeader>,
    /*
     ai,
    replay,
    map_info,
    initial,
    achievements,
    scenario,
    lobby,
     */
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Subheader {
    pub chapter_address: Option<u32>,
    pub compressed_header: CompressedHeader,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct MgzReplay {
    pub header_length: u32,
    pub subheader: Subheader,
    // log_version: Option<u32>
    // version: ...
}

fn mgz_compressed_header(input: &[u8]) -> IResult<&[u8], CompressedHeader> {
    let (input, game_version) = primitives::cstring(input)?;
    let (input, old_save_version) = le_f32(input)?;
    let (input, new_save_version) = cond(old_save_version == -1.0f32, le_u32)(input)?;
    let save_version = compute_save_version(old_save_version, new_save_version);
    // Python: "version"/Computed(lambda ctx: get_version(ctx.game_version, ctx.save_version, None)),
    // Python: "hd"/If(lambda ctx: ctx.version == Version.HD and ctx.save_version > 12.34, hd),
    // Python: "de"/If(lambda ctx: ctx.version == Version.DE, de),
    let (input, de) = de::mgz_de(input, save_version)?;

    let compressed_header = CompressedHeader {
        game_version: game_version.to_string(),
        old_save_version,
        new_save_version,
        save_version,
        hd: None,
        de: Some(de),
    };

    Ok((input, compressed_header))
}

fn mgz_decompress_header(compressed: &[u8]) -> Result<Vec<u8>, io::Error> {
    let deflated_data = {
        let mut deflater = ZlibDecoder::new_with_decompress(
            compressed,
            Decompress::new_with_window_bits(false, 15),
        );
        let mut deflated_data = Vec::new();
        let deflated = deflater.read_to_end(&mut deflated_data);
        deflated.map(|_| deflated_data)
    }?;
    Ok(deflated_data)
}

fn mgz_subheader(input: &[u8], header_length: u32) -> IResult<&[u8], Subheader> {
    let (input, check) = peek(le_u32)(input)?;
    let (input, chapter_address) = cond(check < 100000000, le_u32)(input)?;
    let header_read_len = header_length - 4 - (if check < 100000000 { 4 } else { 0 });
    let (input, compressed_header) = map_res(take(header_read_len), mgz_decompress_header)(input)?;
    // TODO: better error mapping
    let (input_inner, compressed_header) = mgz_compressed_header(&compressed_header)
        .map_err(|err| nom::Err::Error(make_error(input, nom::error::ErrorKind::Fail)))?;

    let subheader = Subheader {
        chapter_address,
        compressed_header,
    };

    Ok((input, subheader))
}

pub fn mgz_header(input: &[u8]) -> IResult<&[u8], MgzReplay> {
    let (input, header_length) = le_u32(input)?;
    let (input, subheader) = mgz_subheader(input, header_length)?;
    let replay = MgzReplay {
        header_length,
        subheader,
    };

    Ok((input, replay))
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::*;

    #[test]
    fn parse_62_0() {
        let replay_bytes = {
            let mut file =
                File::open("test/recs/de-62.0.aoe2record").expect("Failed to open test recording");
            let mut bytes = Vec::<u8>::new();
            file.read_to_end(&mut bytes).map(|_| bytes)
        }
        .expect("Failed to read test recording");

        let (_, header) = mgz_header(&replay_bytes).expect("Failed to parse test recording");

        assert_eq!(header.subheader.compressed_header.game_version, "VER 9.4");
        assert_eq!(header.subheader.compressed_header.save_version, 62.0f32);

        let de_header = header.subheader.compressed_header.de.unwrap();

        assert_eq!(de_header.fog_of_war, 6);
        assert_eq!(de_header.players.len(), 2);
        assert_eq!(de_header.players[0].name, "_LHD_xiaohai");
        assert_eq!(de_header.players[1].name, "Mars_zZ");
    }

    #[test]
    fn parse_12_87() {
        let replay_bytes = {
            let mut file = File::open("test/recs/de-12.97-6byte-tile.aoe2record")
                .expect("Failed to open test recording");
            let mut bytes = Vec::<u8>::new();
            file.read_to_end(&mut bytes).map(|_| bytes)
        }
        .expect("Failed to read test recording");

        let (_, header) = mgz_header(&replay_bytes).expect("Failed to parse test recording");

        assert_eq!(header.subheader.compressed_header.game_version, "VER 9.4");
        assert_eq!(header.subheader.compressed_header.save_version, 12.97f32);

        let de_header = header.subheader.compressed_header.de.unwrap();

        println!("Player 0: {0:?}", de_header.players[0]);

        assert_eq!(de_header.fog_of_war, 1);
        assert_eq!(de_header.players.len(), 8);
        assert_eq!(de_header.players[0].name, "378906");
        assert_eq!(de_header.players[1].name, "312663");
        assert_eq!(de_header.players[2].name, "0");
        assert_eq!(de_header.players[3].name, "0");
        assert_eq!(de_header.players[4].name, "0");
        assert_eq!(de_header.players[5].name, "0");
        assert_eq!(de_header.players[6].name, "0");
        assert_eq!(de_header.players[7].name, "0");
    }
}
