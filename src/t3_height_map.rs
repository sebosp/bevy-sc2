use super::*;
use nom::bytes::complete::*;
use nom::number::complete::*;
use nom_mpq::MPQ;
use nom_mpq::parser::peek_hex;
use s2protocol::dbg_peek_hex;
use tracing::instrument;

#[derive(Default, Debug, Clone, Copy)]
pub struct T3HeightMap {
    pub width: u32,
    pub height: u32,
}

impl T3HeightMap {
    #[instrument(skip(mpq, file_contents))]
    pub fn from_mpq(
        file_name: &str,
        mpq: &MPQ,
        file_contents: &[u8],
    ) -> Result<Self, BevySC2MapError> {
        let (_, t3_height_sector) = mpq.read_mpq_file_sector("t3HeightMap", false, file_contents)?;
        let (_, t3_height_map) = Self::parse(&t3_height_sector)?;
        Ok(t3_height_map)
    }

    #[tracing::instrument(level = "info", skip(input), fields(input = peek_hex(input)))]
    pub fn parse(input: &[u8]) -> BevySC2MapResult<&[u8], Self> {
        let (tail, _) = dbg_peek_hex(tag(&b"HMAP"[..]), "read file magic, HMAP bytes")(input)?;
        let (tail, _) = dbg_peek_hex(tag(&[0x65, 0x00, 0x00, 0x00][..]), "read file version, 4 bytes")(tail)?;

        let (tail, width_bytes) =
            dbg_peek_hex(take(4usize), "read map terrain width, 4 bytes")(tail)?;
        let (_, width) = u32(nom::number::Endianness::Little)(width_bytes)?;

        let (tail, height_bytes) =
            dbg_peek_hex(take(4usize), "read map terrain height, 4 bytes")(tail)?;
        let (_, height) = u32(nom::number::Endianness::Little)(height_bytes)?;

        let (tail, _unknown_bytes) = dbg_peek_hex(take(16usize), "read 16 unknown bytes")(tail)?;
        //tracing::info!("T3HeightMap ------ Next: {}", peek_hex(tail));
        Ok((
            tail,
            Self {
                width,
                height,
            },
        ))
    }
}
