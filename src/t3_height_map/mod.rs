use super::error::*;
use super::utils::*;
use crate::MAP_SCALE_FACTOR;
use crate::cli::*;
use bevy::camera_controller::free_camera::FreeCamera;
use bevy::color::palettes;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::log::tracing;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use chrono::DateTime;
use s2protocol::cache_handles::document_header::*;
use s2protocol::cache_handles::map::coords::*;
use s2protocol::cache_handles::map_info::*;
use s2protocol::cache_handles::t3_height_map::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;

pub const CELL_HEIGHT_MULTIPLIER: f32 = 5.;

#[derive(Resource, Default, Reflect, Debug)]
#[reflect(Resource, Default)]
pub struct T3HeightMapRes {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}
#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
struct TerrainCell {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    scl_x: f32,
    scl_y: f32,
    scl_z: f32,
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MapDimension {
    x: f32,
    y: f32,
}

impl MapDimension {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<MapCellCoord> for MapDimension {
    fn from(src: MapCellCoord) -> Self {
        Self {
            x: src.x as f32,
            y: src.y as f32,
        }
    }
}

#[instrument]
fn try_get_t3_height_map_from_mpq(
    cache_handle_fname: &str,
) -> Result<(MapInfo, T3HeightMap, Option<DocumentHeader>), BevySC2MapError> {
    let mut document_header: Option<DocumentHeader> = None;
    let cache_contents = read_mpq_file(cache_handle_fname)?;
    // based on sc2-map-analyzer/analyser/read.cpp
    let (_input, mpq) = nom_mpq::parser::parse(&cache_contents)?;
    for (file, _file_size) in mpq.get_files(&cache_contents)? {
        if file == "DocumentHeader"
            && let Ok(docu_header) = DocumentHeader::from_mpq(&mpq, &cache_contents)
        {
            document_header = Some(docu_header);
        }
    }
    let map_info = MapInfo::from_mpq(&mpq, &cache_contents)?;
    tracing::info!("Map Info: {map_info:?}");
    let t3_height_map = T3HeightMap::from_mpq(&mpq, &cache_contents, &map_info)?;
    Ok((map_info, t3_height_map, document_header))
}

/// Loads the t3 height map.
pub fn load_t3_height_map(
    mut commands: Commands,
    cli_params: Res<CliParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result<(), BevyError> {
    let mut t3_height_map: Option<T3HeightMap> = None;
    let mut map_info: Option<MapInfo> = None;
    let mut document_header: Option<DocumentHeader> = None;
    for cache_handle_id in cli_params.ids.split(",") {
        if cache_handle_id.is_empty() {
            continue;
        }
        let cache_handle_fname = format!("{}/{}.s2ma", cli_params.path, cache_handle_id);
        info!("Checking cache_handle_fname: {}", cache_handle_fname);
        if let Ok((map, height, docu_header)) = try_get_t3_height_map_from_mpq(&cache_handle_fname)
        {
            // tracing::info!("t3_height_map: {:?}", height);
            tracing::info!("map_info: {:?}", map);
            tracing::info!("document_header: {:?}", document_header);
            map_info = Some(map);
            t3_height_map = Some(height);
            document_header = docu_header;
        }
    }
    let max_map_dim = if let Some(ref val) = t3_height_map {
        val.width.max(val.height)
    } else {
        tracing::info!("Unable to locate t3_height_map");
        return Ok(());
    };
    let map_size = max_map_dim as f32 * MAP_SCALE_FACTOR;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(map_size, map_size * 0.75, map_size)
            .looking_at(Vec3::new(map_size / 2., 0.0, map_size / 2.), Vec3::Y),
        Hdr,
        DepthPrepass,
        NormalPrepass,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        FreeCamera::default(),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    let map_info = if let Some(val) = map_info {
        val
    } else {
        commands.spawn((
            Text::new(format!(
                "Unable to find MapInfo embedded\n\
                in any of cache handles in directory\n\
                {}\n\
                Trigger Cache downloading by clicking on\n\
                Download Caches button in Swarmy app -> Scan tab.",
                cli_params.path
            )),
            Node {
                position_type: PositionType::Absolute,
                top: px(200),
                left: px(200),
                ..default()
            },
            TextColor(Color::from(palettes::css::RED)),
            TextLayout::new_with_justify(Justify::Right),
        ));
        tracing::error!("Unable to locate MapInfo the cache handles provided");
        return Ok(());
    };

    let t3_height_map = match t3_height_map {
        Some(val) => {
            debug!("Found t3_height_map.");
            val
        }
        None => {
            error!(
                "Unable to find t3HeightMap embedded in the cache handles directory: {}",
                cli_params.path,
            );
            commands.spawn((
                Text::new(format!(
                    "Unable to find cache handles from input: {} {}",
                    cli_params.path, cli_params.ids
                )),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(12),
                    left: px(12),
                    ..default()
                },
                TextColor(Color::from(palettes::css::RED)),
            ));
            tracing::error!("Unable to locate t3HeightMap the cache handles provided");
            return Ok(());
        }
    };

    let playable_dimensions = MapDimension::from(map_info.cell_dim_playable());
    commands.spawn((
        Text::new(format!(
            "{} - {}\n\
                MapInfo Dimensions: {} - {}\n\
                TerrainHeight Dimensions: {} - {}",
            map_info.third_string,
            map_info.fourth_string,
            playable_dimensions.x,
            playable_dimensions.y,
            t3_height_map.width,
            t3_height_map.height
        )),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
        TextColor(Color::from(palettes::css::GREEN)),
        TextLayout::new_with_justify(Justify::Right),
        TextFont {
            font_size: 14.,
            ..default()
        },
    ));
    tracing::info!("docu header: {:?}", document_header);
    if let Some(mut docu_header) = document_header {
        let maybe_dt_1 = match DateTime::from_timestamp_secs(docu_header.some_epoch_1 as i64) {
            Some(val) => val.to_string(),
            None => docu_header.some_epoch_1.to_string(),
        };
        let maybe_dt_2 = match DateTime::from_timestamp_secs(docu_header.some_epoch_2 as i64) {
            Some(val) => val.to_string(),
            None => docu_header.some_epoch_2.to_string(),
        };

        // Remove double new lines to save space in the UI.
        let line_len = 80usize;
        let mut desc_lines: Vec<String> = vec![];
        docu_header.description_long = docu_header.description_long.replace("<n/><n/>", "<n/>");
        let chunks = docu_header.description_long.split(" ");
        let mut curr_str = String::from("");
        for chunk in chunks {
            if curr_str.len() < line_len {
                curr_str.push_str(" ");
                curr_str.push_str(chunk);
            } else {
                desc_lines.push(curr_str.replace("<n/>", "\n"));
                curr_str = chunk.to_string();
            }
        }

        desc_lines.push(curr_str.replace("<n/>", "\n"));
        docu_header.description_long = desc_lines.join("\n");

        // Print DocumentHeader
        commands.spawn((
            Text::new(format!(
                "{} - {}\n\
                    {}\n\
                    Map Dates: {} - {}",
                docu_header.name,
                docu_header.mod_info,
                docu_header.description_long,
                maybe_dt_1,
                maybe_dt_2
            )),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(12),
                right: px(12),
                ..default()
            },
            TextColor(Color::from(palettes::css::GOLD)),
            TextLayout::new_with_justify(Justify::Right),
            TextFont {
                font_size: 14.,
                ..default()
            },
        ));
    }

    let mut cell_x_y_data: Vec<u8> = Vec::with_capacity(max_map_dim * max_map_dim);
    for _ in 0..(max_map_dim * max_map_dim) {
        cell_x_y_data.push(0);
    }

    // Cuboids for the cells.
    for (idx, cell_height) in t3_height_map.data.iter().enumerate() {
        let x = usize::try_from(
            (t3_height_map.width as i32 - idx as i32).abs() % t3_height_map.width as i32,
        )?;
        let y = usize::try_from(
            (t3_height_map.width as i32 - idx as i32).abs() / t3_height_map.width as i32,
        )?;
        let target_vec_pos = y as usize * t3_height_map.width as usize + x as usize;
        cell_x_y_data[target_vec_pos] = *cell_height;
        if x > map_info.cell_right + 15 || y > map_info.cell_top + 15 {
            continue;
        }
        let cell_color = compute_cell_color(*cell_height, x, y, &map_info);
        let (x, y) = (x as f32, y as f32);
        commands.spawn((
            TerrainCell {
                pos_x: x * MAP_SCALE_FACTOR,
                pos_y: 1.,
                pos_z: *cell_height as f32 * MAP_SCALE_FACTOR,
                scl_x: MAP_SCALE_FACTOR,
                scl_y: *cell_height as f32,
                scl_z: MAP_SCALE_FACTOR,
            },
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(
                MAP_SCALE_FACTOR,
                *cell_height as f32 * CELL_HEIGHT_MULTIPLIER * MAP_SCALE_FACTOR,
                MAP_SCALE_FACTOR,
            )))),
            MeshMaterial3d(materials.add(cell_color)),
            //SceneRoot(asset_server.load(GltfAssetLabel::Mesh(0).from_asset("swarmy-objects.gltf"))),
            Transform::from_xyz(
                y * MAP_SCALE_FACTOR,
                *cell_height as f32 * (CELL_HEIGHT_MULTIPLIER / 4.) * MAP_SCALE_FACTOR,
                x * MAP_SCALE_FACTOR,
            ),
        ));
    }
    let t3_height_map_res = T3HeightMapRes {
        data: cell_x_y_data,
        width: t3_height_map.width as usize,
        height: t3_height_map.height as usize,
    };
    commands.insert_resource(t3_height_map_res);
    Ok(())
}

fn compute_cell_color(cell_height: u8, x: usize, y: usize, map_info: &MapInfo) -> Color {
    if x > map_info.cell_right + 15
        || y > map_info.cell_top + 15
        || x + 10 < map_info.cell_left
        || y + 10 < map_info.cell_bottom
    {
        return Color::from(palettes::tailwind::SLATE_500);
    }
    let color = match cell_height {
        0 => palettes::css::BLACK,
        1 => palettes::css::LIGHT_BLUE,
        2 => palettes::tailwind::YELLOW_100,
        3 => palettes::css::DARK_GREEN,
        _ => palettes::css::DARK_RED,
    };
    Color::from(color)
}
