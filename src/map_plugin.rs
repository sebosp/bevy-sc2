//! The main Map Bevy Plugin
use bevy::prelude::*;
use s2protocol::cache_handles::map_info::MapInfo;

use crate::cache_objects;
use crate::swarmy_feathers;
use crate::t3_height_map;
use crate::t3_terrain;
use crate::utils;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, MapPlugin::scene.spawn());
        app.add_systems(Startup, t3_height_map::load_t3_height_map);
        app.add_systems(Startup, cache_objects::doodas::load_object_doodas);
        app.add_systems(Update, cache_objects::load_cache_objects);
        app.add_systems(Update, t3_terrain::load_t3_terrain);
        app.add_systems(Update, utils::update_gizmo_config);
    }
}
impl MapPlugin {
    fn scene() -> impl SceneList {
        bsn_list![swarmy_feathers::init_feathers()]
    }
}

/// A resource mirror of s2protocol::cache_handles::map_info::MapInfo
#[derive(Resource, Default, Reflect, Debug, Default, Debug, Clone)]
pub struct MapInfoResource {
    pub file_version: i32,
    pub cell_width: usize,
    pub cell_height: usize,
    /// Mostly seen empty?
    pub first_string: String,
    /// Also empty?
    pub second_string: String,
    // Maybe a mode, light Dark/Light?
    pub third_string: String,
    // Some name, "Zerus" in the test case, maybe map maker?
    pub fourth_string: String,
    pub cell_left: usize,
    pub cell_bottom: usize,
    pub cell_right: usize,
    pub cell_top: usize,
}

impl From<MapInfo> for MapInfoResource {
    pub fn from(src: MapInfo) -> Self {
        Self {
            file_version,
            cell_width,
            cell_height,
            first_string,
            second_string,
            third_string,
            fourth_string,
            cell_left,
            cell_bottom,
            cell_right,
            cell_top,
        }
    }
}

/// A resource mirror of s2protocol::cache_handles::document_header::DocumentHeader
#[derive(Resource, Default, Reflect, Debug, Default, Debug, Clone)]
pub struct DocumentHeaderResource {
    pub maybe_dimension_x1: i32,
    pub maybe_dimension_y1: i32,
    // TODO: these are not epochs, probably more like units that are 1000th the value of the
    // maybe_dimension_x1
    pub some_epoch_1: i32,
    pub some_epoch_2: i32,
    pub mod_info: String,
    // Aka the map title.
    pub name: String,
    /// A long description of the map.
    pub description_long: String,
    /// A short description of the map, in the few files I've checked it's empty.
    pub description_short: String,
}
