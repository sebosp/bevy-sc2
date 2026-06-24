use super::t3_height_map::T3HeightMapResource;
use crate::CELL_HEIGHT_MULTIPLIER;
use crate::MAP_SCALE_FACTOR;
use crate::MapScene;
use crate::standard_material_from_gltf_material;
use bevy::gltf::GltfMaterial;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// A few Object Units.
pub const XEL_NAGA_TOWER_HEIGHT: f32 = 25.0;
pub const XEL_NAGA_TOWER_RADIUS: f32 = 0.5;
pub const DESTRUCTIBLE_ROCKS_HEIGHT: f32 = 5.0;
pub const DESTRUCTIBLE_ROCKS_RADIUS: f32 = 5.0;

// Testing a Dooda.
pub const SHADOW_PLATFORM_RAMP_SIZE: f32 = 1.0;

// Testing Object Points.
pub const NO_FLY_ZONE_HEIGHT: f32 = 25.0;
pub const NO_FLY_ZONE_RADIUS: f32 = 0.5;

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

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct XelNagaTowerMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct DestructibleRockEx1DiagonalHugeBLURMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct UnknownUnit;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct ShadowPlatformRampMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct UnknownDoodaMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct NoFlyZoneMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct UnknownObjectPointMaterial;

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
pub struct StartLocMaterial;

/// Loads the t3 height map.
pub fn load_cache_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    t3_height_map_res: Res<T3HeightMapResource>,
    map_scene: Res<MapScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_materials: Res<Assets<GltfMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // TODO: Move this to s2protocol
    let path = "/home/seb/SC2Replays/swarmy/extract/a76deb95741e1d3d24527f0a303914824455bc9d68411fa143d23cc4edee9c27/Objects".to_string();
    let files_content = std::fs::read_to_string(&path)?;
    let placed_objects = serde_xml_rs::from_str::<PlacedObjects>(&files_content)?;
    let mineral_mesh = Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(
        MAP_SCALE_FACTOR,
        MAP_SCALE_FACTOR,
        MAP_SCALE_FACTOR,
    ))));
    let xel_naga_mesh = Mesh3d(meshes.add(Cylinder::new(
        XEL_NAGA_TOWER_RADIUS * MAP_SCALE_FACTOR,
        XEL_NAGA_TOWER_HEIGHT * MAP_SCALE_FACTOR,
    )));
    let destructible_rock_ex1_diagonal_huge_blur_material_mesh = Mesh3d(meshes.add(Cylinder::new(
        DESTRUCTIBLE_ROCKS_RADIUS * MAP_SCALE_FACTOR,
        DESTRUCTIBLE_ROCKS_HEIGHT * MAP_SCALE_FACTOR,
    )));
    for unit in placed_objects.units {
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
        let Some(unit_handle) = gltf.named_materials.get(unit.unit_kind.as_str()) else {
            tracing::warn!(
                "Unhandled Skein GLTF named_material Unit: {}",
                unit.unit_kind
            );
            continue;
        };
        let Some(unit_gltf_material) = gltf_materials.get(unit_handle.id()) else {
            tracing::warn!(
                "Unhandled Skein GLTF gltf_material Unit: {}",
                unit.unit_kind
            );
            continue;
        };
        let unit_material = MeshMaterial3d(
            materials.add(standard_material_from_gltf_material(&unit_gltf_material)),
        );
        let x = unit_pos[0];
        let y = unit_pos[1];
        let target_vec_pos = y as usize * t3_height_map_res.width as usize + x as usize;
        let cell_height = t3_height_map_res.data[target_vec_pos];
        let mineral_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * y,
            cell_height as f32 * CELL_HEIGHT_MULTIPLIER * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * x,
        );
        let xel_naga_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * y,
            XEL_NAGA_TOWER_HEIGHT * 0.5 * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * x,
        );
        let destructible_rock_ex1_diagonal_huge_blur_material_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * y,
            DESTRUCTIBLE_ROCKS_HEIGHT * 0.5 * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * x,
        );
        match unit.unit_kind.as_ref() {
            "RichMineralField750" => commands.spawn((
                RichMineralField750Material,
                mineral_mesh.clone(),
                unit_material,
                mineral_transform,
            )),
            "RichMineralField" => commands.spawn((
                RichMineralFieldMaterial,
                mineral_mesh.clone(),
                unit_material,
                mineral_transform,
            )),
            "MineralField750" => commands.spawn((
                MineralField750Material,
                mineral_mesh.clone(),
                unit_material,
                mineral_transform,
            )),
            "MineralField" => commands.spawn((
                MineralFieldMaterial,
                mineral_mesh.clone(),
                unit_material,
                mineral_transform,
            )),
            "RichVespeneGeyser" => commands.spawn((
                RichVespeneGeyserMaterial,
                mineral_mesh.clone(),
                unit_material,
                mineral_transform,
            )),
            "VespeneGeyser" => commands.spawn((
                VespeneGeyserMaterial,
                mineral_mesh.clone(),
                unit_material,
                mineral_transform,
            )),
            "SpacePlatformGeyser" => commands.spawn((
                SpacePlatformGeyserMaterial,
                mineral_mesh.clone(),
                unit_material,
                mineral_transform,
            )),
            "XelNagaTower" => commands.spawn((
                XelNagaTowerMaterial,
                xel_naga_mesh.clone(),
                unit_material,
                xel_naga_transform,
            )),
            "DestructibleRockEx1DiagonalHugeBLUR" => commands.spawn((
                DestructibleRockEx1DiagonalHugeBLURMaterial,
                destructible_rock_ex1_diagonal_huge_blur_material_mesh.clone(),
                unit_material,
                destructible_rock_ex1_diagonal_huge_blur_material_transform,
            )),
            _ => {
                tracing::warn!("Unhandled unit_kind: {}", unit.unit_kind);
                commands.spawn((
                    RichMineralFieldMaterial,
                    mineral_mesh.clone(),
                    unit_material,
                    mineral_transform,
                ))
            }
        };
    }
    let shadow_platform_ramp_mesh = Mesh3d(meshes.add(Cuboid::new(
        SHADOW_PLATFORM_RAMP_SIZE * MAP_SCALE_FACTOR,
        SHADOW_PLATFORM_RAMP_SIZE * MAP_SCALE_FACTOR,
        SHADOW_PLATFORM_RAMP_SIZE * MAP_SCALE_FACTOR,
    )));
    // Id="1231" Position="118.1433,8.0437,6.9763" Scale="1,1,1" Type="Shadow_Platform_Ramp"
    for dooda in placed_objects.doodas {
        // These are objects in the map, decorations, animation references, etc.
        let unit_pos: Vec<f32> = dooda
            .position
            .split(",")
            .filter_map(|x| x.parse::<f32>().ok())
            .collect();
        if unit_pos.len() != 3 {
            tracing::error!(
                "Unexpected number of tokens for unit position typed: {}",
                dooda.kind
            );
            continue;
        }
        let x = unit_pos[0];
        let y = unit_pos[1];

        let Some(unit_handle) = gltf.named_materials.get(dooda.kind.as_str()) else {
            tracing::warn!("Unhandled Skein GLTF named_material Dooda: {}", dooda.kind);
            continue;
        };
        let Some(unit_gltf_material) = gltf_materials.get(unit_handle.id()) else {
            tracing::warn!("Unhandled Skein GLTF gltf_material Dooda: {}", dooda.kind);
            continue;
        };
        let dooda_material = MeshMaterial3d(
            materials.add(standard_material_from_gltf_material(&unit_gltf_material)),
        );

        let shadow_platform_ramp_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * y,
            SHADOW_PLATFORM_RAMP_SIZE * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * x,
        );
        match dooda.kind.as_str() {
            "Shadow_Platform_Ramp" => commands.spawn((
                ShadowPlatformRampMaterial,
                shadow_platform_ramp_mesh.clone(),
                dooda_material,
                shadow_platform_ramp_transform,
            )),
            _ => commands.spawn((
                UnknownDoodaMaterial,
                shadow_platform_ramp_mesh.clone(),
                dooda_material,
                shadow_platform_ramp_transform,
            )),
        };
    }
    // Id="1035" Position="6.0996,150.3146,0" Scale="1,1,1" Type="NoFlyZone" Name="No Fly Zone 011" Color="0,0,0,0" PathingRadiusSoft="5" PathingRadiusHard="4"
    for object_point in placed_objects.points {
        // These are objects in the map, decorations, animation references, etc.
        let unit_pos: Vec<f32> = object_point
            .position
            .split(",")
            .filter_map(|x| x.parse::<f32>().ok())
            .collect();
        if unit_pos.len() != 3 {
            tracing::error!(
                "Unexpected number of tokens for unit position typed: {}",
                object_point.kind
            );
            continue;
        }
        let x = unit_pos[0];
        let y = unit_pos[1];
        let z = unit_pos[2];
        let target_vec_pos = y as usize * t3_height_map_res.width as usize + x as usize;
        let cell_height = t3_height_map_res.data[target_vec_pos];

        let Some(unit_handle) = gltf.named_materials.get(object_point.kind.as_str()) else {
            tracing::warn!(
                "Unhandled Skein GLTF named_material ObjectPoint: {}",
                object_point.kind
            );
            continue;
        };
        let Some(unit_gltf_material) = gltf_materials.get(unit_handle.id()) else {
            tracing::warn!(
                "Unhandled Skein GLTF gltf_material ObjectPoint: {}",
                object_point.kind
            );
            continue;
        };
        let object_point_material = MeshMaterial3d(
            materials.add(standard_material_from_gltf_material(&unit_gltf_material)),
        );
        let pathing_radius_soft = object_point.pathing_radius_soft as f32;
        let _pathing_radius_hard = object_point.pathing_radius_hard as f32;
        let torus_mesh = Mesh3d(meshes.add(Torus::new(0.2, 0.25)));
        let cylinder_mesh = Mesh3d(meshes.add(Cylinder::new(
            pathing_radius_soft * NO_FLY_ZONE_RADIUS * MAP_SCALE_FACTOR,
            NO_FLY_ZONE_HEIGHT * MAP_SCALE_FACTOR,
        )));
        let cylinder_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * y,
            z + cell_height as f32 * CELL_HEIGHT_MULTIPLIER * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * x,
        );
        // TODO: We should use the alpha channel, dunno if we need glsl for that tho because it's
        // being set from blender and exported to gltf.
        match object_point.kind.as_str() {
            "NoFlyZone" => commands.spawn((
                NoFlyZoneMaterial,
                cylinder_mesh,
                object_point_material,
                cylinder_transform,
            )),
            "StartLoc" => commands.spawn((
                StartLocMaterial,
                torus_mesh,
                object_point_material,
                cylinder_transform,
            )),
            _ => commands.spawn((
                UnknownObjectPointMaterial,
                shadow_platform_ramp_mesh.clone(),
                cylinder_mesh,
                cylinder_transform,
            )),
        };
    }
    *loaded = true;
    Ok(())
}
