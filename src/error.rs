//! Error handling of BevySC2MapError

use std::num::TryFromIntError;

use nom::error::ErrorKind;
use nom::error::ParseError;

use crate::MapTerrainCoord;

/// Holds the result of parsing progress and the possibly failures
pub type BevySC2MapResult<I, O> = Result<(I, O), BevySC2MapError>;

#[derive(thiserror::Error, Debug)]
pub enum BevySC2MapError {
    /// Unable to parse the MPQ file, could be corrupted or not a replay file
    #[error("MPQ Error")]
    MPQ(#[from] nom_mpq::MPQParserError),
    /// Unable to parse the byte aligned data types
    #[error("Nom ByteAligned Error {0}")]
    ByteAligned(String),
    /// Unable to parse a value that should have been an integer
    #[error("TryFromIntError")]
    ValueError(#[from] TryFromIntError),
    /// An I/O Error
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    /// Conversion to UTF-8 failed, from the `Vec<u8>` "name" fields in the proto fields
    #[error("Utf8 conversion error")]
    Utf8Error(#[from] std::str::Utf8Error),
    /// Map Size is bigger than max supported in game (I guess...)
    #[error("Expected max 256 for map size, got {0}")]
    InvalidMapSize(i32),
    // /The map coordinates bounds are invalid
    #[error("Expected coordinate {0} to be less than {1}")]
    InvalidCoordinateBounds(String, i32, String, i32),
    /// The MapInfo and t3HeightMay dimensions do not match
    #[error("T3 Height Map Terrain Dimensions {0:?} do not match Map Info Map Dimensions {1:?}")]
    T3HeightDimDoNotMatchMapInfoDim(MapTerrainCoord, MapTerrainCoord),
    /// Expected at least n bytes but got x bytes
    #[error("Expected at least {0} bytes, got {1} bytes")]
    T3HeightNotEnoughBytes(usize, usize),
    /// The height unit is out of bounds.
    #[error("Height unit out of bounds should be between 1 and 4, but got: {0}")]
    T3HeightUnitOutOfBounds(i32),
    /// Other error
    #[error("Other Error: {0}")]
    Other(String),
}

/// Conversion of errors from byte aligned parser
impl<I> From<nom::Err<nom::error::Error<I>>> for BevySC2MapError
where
    I: Clone + std::fmt::Debug,
{
    fn from(err: nom::Err<nom::error::Error<I>>) -> Self {
        match err {
            nom::Err::Incomplete(_) => {
                unreachable!("This library is compatible with only complete parsers, not streaming")
            }
            nom::Err::Error(e) => BevySC2MapError::ByteAligned(format!("{e:?}")),
            nom::Err::Failure(e) => BevySC2MapError::ByteAligned(format!("{e:?}")),
        }
    }
}

impl<I> ParseError<I> for BevySC2MapError
where
    I: Clone,
{
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        BevySC2MapError::ByteAligned(format!("{kind:?}"))
    }

    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}
