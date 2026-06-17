use super::t3_height_map::T3HeightMapRes;
use crate::CELL_HEIGHT_MULTIPLIER;
use crate::MAP_SCALE_FACTOR;
use crate::MapScene;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// In the observed map, for some reason the objects are placed 10 units from the origin...
pub const PLACED_OBJECT_OFFSET: f32 = 0.;

pub const MINERAL_FIELD_MULTIPLIER: f32 = 0.;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedObjects {
    #[serde(rename = "@Version")]
    pub version: u32,
    #[serde(rename = "ObjectPoint", default)]
    pub points: Vec<ObjectPoint>,
    #[serde(rename = "ObjectDoodad", default)]
    pub doodas: Vec<ObjectDoodad>,
    #[serde(rename = "ObjectUnit", default)]
    pub units: Vec<ObjectUnit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDoodad {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(default, rename = "@Variation")]
    pub variation: String,
    #[serde(rename = "@Position")]
    pub position: String,
    #[serde(default, rename = "@Rotation")]
    pub rotation: String,
    #[serde(rename = "@Scale")]
    pub scale: String,
    #[serde(rename = "@Type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPoint {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Position")]
    pub position: String,
    #[serde(rename = "@Scale")]
    pub scale: String,
    #[serde(rename = "@Type")]
    pub kind: String,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@Color")]
    pub color: String,
    #[serde(default, rename = "@PathingRadiusSoft")]
    pub pathing_radius_soft: u32,
    #[serde(default, rename = "@PathingRadiusHard")]
    pub pathing_radius_hard: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectUnit {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(default, rename = "@Variation")]
    pub variation: String,
    #[serde(rename = "@Position")]
    pub position: String,
    #[serde(rename = "@Scale")]
    pub scale: String,
    #[serde(rename = "@UnitType")]
    pub unit_kind: String,
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct RichMineralField750Material;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct RichMineralFieldMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct MineralField750Material;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct MineralFieldMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct VespeneGeyserMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct RichVespeneGeyserMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct SpacePlatformGeyserMaterial;

/// Loads the t3 height map.
pub fn load_cache_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    t3_height_map_res: Res<T3HeightMapRes>,
    map_scene: Res<MapScene>,
    gltf_assets: Res<Assets<Gltf>>,
    mut loaded: Local<bool>,
) -> Result<(), BevyError> {
    // Only do this once
    if *loaded {
        return Ok(());
    }
    // Wait until the scene is loaded
    let Some(gltf) = gltf_assets.get(&map_scene.0) else {
        return Ok(());
    };

    let path = "/home/seb/SC2Replays/swarmy/extract/a76deb95741e1d3d24527f0a303914824455bc9d68411fa143d23cc4edee9c27/Objects".to_string();
    if let Ok(files_content) = std::fs::read_to_string(&path) {
        match serde_xml_rs::from_str::<PlacedObjects>(&files_content) {
            Ok(val) => {
                for unit in val.units {
                    tracing::info!("{:?}", unit);
                    // ObjectUnit { id: "209", variation: "8", position: "97,102.5,0", scale: "1,1,1", unit_kind: "RichMineralField" }
                    let unit_pos: Vec<f32> = unit
                        .position
                        .split(",")
                        .filter_map(|x| x.parse::<f32>().ok())
                        .collect();
                    if unit_pos.len() != 3 {
                        tracing::error!(
                            "Unexpected number of tokens for unit position typed: {}",
                            unit.unit_kind
                        );
                        continue;
                    }
                    let Some(unit_material) = gltf.named_materials.get(unit.unit_kind.as_str())
                    else {
                        tracing::warn!("Unhandled ObjectUnit: {}", unit.unit_kind);
                        continue;
                    };
                    let x = unit_pos[0] + PLACED_OBJECT_OFFSET;
                    let y = unit_pos[1] + PLACED_OBJECT_OFFSET;
                    // x = t3_height_map.width - (idx as i32) % t3_height_map.width;
                    // y = t3_height_map.width - idx as i32 / t3_height_map.width;
                    let target_vec_pos = y as usize * t3_height_map_res.width as usize + x as usize;
                    let cell_height = t3_height_map_res.data[target_vec_pos];
                    // TODO: continue the t3_height_map_res here.
                    //let cell_height = t3_height_map_res.data[unit_pos[0] as usize * t3_height_map_res.width + unit_pos[1] as usize];
                    commands.spawn((
                        MineralField750Material,
                        Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(
                            MAP_SCALE_FACTOR,
                            MAP_SCALE_FACTOR,
                            MAP_SCALE_FACTOR,
                        )))),
                        //MeshMaterial3d(materials.add(unit_color)),
                        MeshMaterial3d(unit_material.clone()),
                        //SceneRoot(asset_server.load(GltfAssetLabel::Mesh(0).from_asset("swarmy-objects.gltf"))),
                        // TODO: The camera coordinate space is right-handed X-right, Y-up, Z-back.
                        // This is probably not the way to deal with the camera coords...
                        Transform::from_xyz(
                            MAP_SCALE_FACTOR * y,
                            cell_height as f32 * CELL_HEIGHT_MULTIPLIER * MAP_SCALE_FACTOR,
                            MAP_SCALE_FACTOR * x,
                        ),
                    ));
                }
                //val,
            }
            Err(err) => {
                tracing::error!("Failed to parse XML file {:?}: {}", path, err);
            }
        }
    }
    *loaded = true;
    Ok(())
}
