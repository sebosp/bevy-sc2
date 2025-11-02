use super::*;
use nom::bytes::complete::*;
use nom::number::complete::*;
use nom_mpq::MPQ;
use nom_mpq::parser::peek_hex;
use s2protocol::dbg_peek_hex;
use tracing::instrument;

/// The MapInfo coordinates's purpose is to translate "cell"coordinates to "terrain" coordinates,
/// Showing the playable terrain.
/// The width and height defined in the MapInfo determine the width and height of the
/// coords::MapCellCoord.
#[derive(Default, Debug, Clone)]
pub struct MapInfo {
    pub file_version: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    // Maybe a mode, light Dark/Light?
    pub third_string: String,
    // Some name, "Zerus" in the test case
    pub fourth_string: String,
    pub cell_left: u32,
    pub cell_bottom: u32,
    pub cell_right: u32,
    pub cell_top: u32,
}

impl MapInfo {
    #[instrument(skip(mpq, file_contents))]
    pub fn from_mpq(
        file_name: &str,
        mpq: &MPQ,
        file_contents: &[u8],
    ) -> Result<Self, BevySC2MapError> {
        let (_, map_info_sector) = mpq.read_mpq_file_sector("MapInfo", false, file_contents)?;
        let (_, map_info) = Self::parse(&map_info_sector)?;
        Ok(map_info)
    }

    #[tracing::instrument(level = "info", skip(input), fields(input = peek_hex(input)))]
    pub fn parse(input: &[u8]) -> BevySC2MapResult<&[u8], Self> {
        let (tail, _) = dbg_peek_hex(tag(&b"IpaM"[..]), "read file magic, IpaM bytes")(input)?;

        let (mut tail, file_version_bytes) =
            dbg_peek_hex(take(4usize), "read file_version, 4 bytes")(tail)?;
        let (_, file_version) = u32(nom::number::Endianness::Little)(file_version_bytes)?;
        if file_version > 24 {
            // If file_version is more than 24 it seems to need 8 more bytes to read
            let (extra_tail, _) =
                dbg_peek_hex(take(8usize), "file_version >= 24 needs 8 more extra bytes")(tail)?;
            tail = extra_tail;
        }
        let (tail, cell_width_bytes) =
            dbg_peek_hex(take(4usize), "read map cell_width, 4 bytes")(tail)?;
        let (_, cell_width) = u32(nom::number::Endianness::Little)(cell_width_bytes)?;

        let (tail, cell_height_bytes) =
            dbg_peek_hex(take(4usize), "read map cell_height, 4 bytes")(tail)?;
        let (_, cell_height) = u32(nom::number::Endianness::Little)(cell_height_bytes)?;

        if cell_width > 256 || cell_height > 256 {
            tracing::warn!(
                "MapInfo cell_width({}) or cell_height({}) is larger than expected 256",
                cell_width,
                cell_height
            );
            return Err(BevySC2MapError::InvalidMapSize(cell_width.max(cell_height)));
        }
        let (tail, _unknown_bytes) =
            dbg_peek_hex(take(8usize), "read 8 unknown bytes after cell_height")(tail)?;
        let (tail, _) = dbg_peek_hex(
            take_while(|x| x == 0u8),
            "padding zeros fill before Strings after cell_height+unknown",
        )(tail)?;

        let (tail, _) = dbg_peek_hex(take_while(|x| x != 0u8), "walk past the first string")(tail)?;
        let (tail, _unknown_byte) = dbg_peek_hex(
            take(1usize),
            "advance past termination character first string",
        )(tail)?;

        let (tail, _) =
            dbg_peek_hex(take_while(|x| x != 0u8), "walk past the second string")(tail)?;
        let (tail, _unknown_byte) = dbg_peek_hex(
            take(1usize),
            "advance past termination character second string",
        )(tail)?;

        let (tail, _unknown_bytes) =
            dbg_peek_hex(take(5usize), "read 5 unknown bytes after second string")(tail)?;

        let (tail, string_bytes) =
            dbg_peek_hex(take_while(|x| x != 0u8), "collect third string")(tail)?;
        let third_string = String::from_utf8_lossy(string_bytes).to_string();
        let (tail, _unknown_byte) = dbg_peek_hex(
            take(1usize),
            "advance past termination character second string",
        )(tail)?;

        let (tail, string_bytes) =
            dbg_peek_hex(take_while(|x| x != 0u8), "collect fourth string")(tail)?;
        let fourth_string = String::from_utf8_lossy(string_bytes).to_string();
        let (tail, _unknown_byte) = dbg_peek_hex(
            take(1usize),
            "advance past termination character second string",
        )(tail)?;

        let (tail, cell_left_bytes) =
            dbg_peek_hex(take(4usize), "read map cell_left, 4 bytes")(tail)?;
        let (_, cell_left) = u32(nom::number::Endianness::Little)(cell_left_bytes)?;

        let (tail, cell_bottom_bytes) =
            dbg_peek_hex(take(4usize), "read map cell_bottom, 4 bytes")(tail)?;
        let (_, cell_bottom) = u32(nom::number::Endianness::Little)(cell_bottom_bytes)?;

        let (tail, cell_right_bytes) =
            dbg_peek_hex(take(4usize), "read map cell_right, 4 bytes")(tail)?;
        let (_, cell_right) = u32(nom::number::Endianness::Little)(cell_right_bytes)?;

        let (tail, cell_top_bytes) =
            dbg_peek_hex(take(4usize), "read map cell_top, 4 bytes")(tail)?;
        let (_, cell_top) = u32(nom::number::Endianness::Little)(cell_top_bytes)?;

        if cell_left >= cell_right {
            return Err(BevySC2MapError::InvalidCoordinateBounds(
                "MapInfoCellLeft".to_string(),
                cell_left,
                "MapInfoCellRight".to_string(),
                cell_right,
            ));
        }
        if cell_bottom >= cell_top {
            return Err(BevySC2MapError::InvalidCoordinateBounds(
                "MapInfoCellBottom".to_string(),
                cell_bottom,
                "MapInfoCellTop".to_string(),
                cell_top,
            ));
        }

        if cell_right > cell_width {
            return Err(BevySC2MapError::InvalidCoordinateBounds(
                "MapInfoCellRight".to_string(),
                cell_right,
                "MapInfoCellWidth".to_string(),
                cell_width,
            ));
        }

        if cell_top > cell_height {
            return Err(BevySC2MapError::InvalidCoordinateBounds(
                "MapInfoCellTop".to_string(),
                cell_top,
                "MapInfoCellHeight".to_string(),
                cell_height,
            ));
        }

        Ok((
            tail,
            Self {
                file_version,
                cell_width,
                cell_height,
                third_string,
                fourth_string,
                cell_left,
                cell_bottom,
                cell_right,
                cell_top,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_map_info() {
        // xxd -ps -c 1 < MapInfo|sed 's/^/0x/g;s/$/,/g'|xargs echo -n
        let cache_contents: Vec<u8> = vec![
            0x49, 0x70, 0x61, 0x4d, // 4 bytes IpaM Magic
            0x27, 0x00, 0x00, 0x00, // 4 bytes file_version, in this case more than 24.
            0xc3, 0x38, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, // Take 8 extra bytes for file_version > 24
            0xa8, 0x00, 0x00, 0x00, // 4 bytes width = 168
            0xa8, 0x00, 0x00, 0x00, // 4 bytes height = 168
            0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // Ignore 8 unknown bytes
            0x00, 0x00, // Pre-amble of strings, skip zeros.
            0x04, // First string
            0x00, // Advance termination character
            // Find terminator of second string, nothing is taken, would skip non-zero bytes
            0x00, // Advance termination character
            0x00, 0x00, 0x00, 0x00, // Skip 4 bytes
            0x00, // Advance an extra byte.
            0x44, 0x61, 0x72, 0x6b, // third string "Dark"
            0x00, // Advance termination character
            0x5a, 0x65, 0x72, 0x75, 0x73, // fourth string "Zerus"
            0x00, // Advance termination character
            0x0e, 0x00, 0x00, 0x00, // 6 bytes for "left"
            0x0e, 0x00, 0x00, 0x00, // 6 bytes for "bottom"
            0x9a, 0x00, 0x00, 0x00, // 6 bytes for "right"
            0x9a, 0x00, 0x00, 0x00, // 6 bytes for "top"
            // The rest of the data is unknown.
            0x00, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20,
            0x03, 0x00, 0x00, 0x58, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00,
            0xbd, 0xc6, 0x0a, 0x68, 0xa7, 0xc6, 0x0a, 0x68, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0f, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let (_, map_info) = MapInfo::parse(&cache_contents).unwrap();
        assert_eq!(map_info.cell_width, 168);
        assert_eq!(map_info.cell_height, 168);
        assert_eq!(map_info.third_string, "Dark".to_string());
        assert_eq!(map_info.fourth_string, "Zerus".to_string());
    }
}
