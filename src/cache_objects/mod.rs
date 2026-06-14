use bevy::color::palettes;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::cli::*;

// In the observed map, for some reason the objects are placed 10 units from the origin...
pub const PLACED_OBJECT_OFFSET: f32 = 10.;

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

/// Loads the t3 height map.
pub fn load_cache_objects(
    mut commands: Commands,
    cli_params: Res<CliParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    let path = "/home/seb/SC2Replays/swarmy/extract/a76deb95741e1d3d24527f0a303914824455bc9d68411fa143d23cc4edee9c27/Objects".to_string();

    if let Ok(files_content) = std::fs::read_to_string(&path) {
        match serde_xml_rs::from_str::<PlacedObjects>(&files_content) {
            Ok(val) => {
                tracing::info!("{:?}", val);
                for unit in val.units {
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
                    let unit_color: StandardMaterial = match unit.unit_kind.as_ref() {
                        "RichMineralField750" => {
                            let mut unit_col =
                                StandardMaterial::from(Color::from(palettes::tailwind::ORANGE_600));
                            unit_col.metallic = 1.0;
                            unit_col
                        }
                        "RichMineralField" => {
                            let mut unit_col =
                                StandardMaterial::from(Color::from(palettes::tailwind::YELLOW_500));
                            unit_col.metallic = 1.0;
                            unit_col
                        }
                        "MineralField750" => {
                            StandardMaterial::from(Color::from(palettes::tailwind::BLUE_600))
                        }
                        "MineralField" => {
                            StandardMaterial::from(Color::from(palettes::tailwind::CYAN_400))
                        }
                        "RichVespeneGeyser" => {
                            StandardMaterial::from(Color::from(palettes::tailwind::GREEN_500))
                        }
                        "VespeneGeyser" => {
                            StandardMaterial::from(Color::from(palettes::tailwind::VIOLET_600))
                        }
                        "SpacePlatformGeyser" => {
                            StandardMaterial::from(Color::from(palettes::tailwind::ROSE_600))
                        }
                        _ => StandardMaterial::from(Color::from(palettes::tailwind::NEUTRAL_500)),
                    };
                    let x = unit_pos[0] + PLACED_OBJECT_OFFSET;
                    let y = unit_pos[1] + PLACED_OBJECT_OFFSET;
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(0.1, 8., 0.1)))),
                        MeshMaterial3d(materials.add(unit_color)),
                        //SceneRoot(asset_server.load(GltfAssetLabel::Mesh(0).from_asset("swarmy-objects.gltf"))),
                        // TODO: The camera coordinate space is right-handed X-right, Y-up, Z-back.
                        // This is probably not the way to deal with the camera coords...
                        Transform::from_xyz(0.1 * y, 0., 0.1 * x),
                    ));
                }
                //val,
            }
            Err(err) => {
                tracing::error!("Failed to parse XML file {:?}: {}", path, err);
            }
        }
    }
}
