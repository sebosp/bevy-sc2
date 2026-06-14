pub mod error;
pub use error::*;

pub use s2protocol::cache_handles::document_header::*;
pub use s2protocol::cache_handles::map::coords::*;
pub use s2protocol::cache_handles::map_info::*;
pub use s2protocol::cache_handles::t3_height_map::*;

pub mod t3_height_map;
pub use t3_height_map::*;
pub mod cache_objects;
pub use cache_objects::*;

pub mod cli;

pub mod utils;
pub use utils::*;

pub const MAP_SCALE_FACTOR: f32 = 0.1;
