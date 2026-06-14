
use serde::{Deserialize, Serialize};
use bevy::color::palettes;
use bevy::camera_controller::free_camera::{FreeCamera};
use bevy::prelude::*;
use s2protocol::cache_handles::document_header::*;
use s2protocol::cache_handles::map::coords::*;
use s2protocol::cache_handles::map_info::*;
use s2protocol::cache_handles::t3_height_map::*;
use tracing::instrument;
use super::utils::*;
use super::error::*;
use bevy::log::tracing;
use chrono::DateTime;
use crate::cli::*;


#[derive(Component, Default, bevy::reflect::Reflect, Debug)]
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
) {
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
    let map_size = if let Some(ref val) = t3_height_map {
        val.width.max(val.height) as f32 * 0.1
    } else {
        1.
    };
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(map_size * 1., map_size * 0.75, map_size)
            .looking_at(Vec3::new(map_size / 2., 0.0, map_size / 2.), Vec3::Y),
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
        return;
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
            return;
        }
    };

    let playable_dimensions = MapDimension::from(map_info.cell_dim_playable());
    let map_dimension = MapDimension::new(t3_height_map.width as f32, t3_height_map.height as f32);
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

    // Cuboids for the cells.
    for (idx, cell_height) in t3_height_map.data.iter().enumerate() {
        let x = t3_height_map.width - (idx as i32) % t3_height_map.width;
        let y = t3_height_map.width - idx as i32 / t3_height_map.width;
        if x - 1 < map_info.cell_left || y - 1 < map_info.cell_bottom {
            continue;
        }
        let (x, y) = (x as f32, y as f32);
        let cell_color = compute_cell_color(
            *cell_height,
            x as f32,
            y as f32,
            playable_dimensions,
            map_dimension,
        );
        commands.spawn((
            TerrainCell {
                pos_x: x * 0.1,
                pos_y: 1.,
                pos_z: *cell_height as f32 * 0.1,
                scl_x: 0.1,
                scl_y: *cell_height as f32,
                scl_z: 0.1,
            },
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(
                0.1,
                *cell_height as f32 * 0.5,
                0.1,
            )))),
            MeshMaterial3d(materials.add(cell_color)),
            //SceneRoot(asset_server.load(GltfAssetLabel::Mesh(0).from_asset("swarmy-objects.gltf"))),
            Transform::from_xyz(y * 0.1, *cell_height as f32 * 0.25, x * 0.1),
        ));
    }
}


fn compute_cell_color(
    cell_height: u8,
    x: f32,
    y: f32,
    playable_dimensions: MapDimension,
    map_dimension: MapDimension,
) -> Color {
    let border_x = (map_dimension.x - playable_dimensions.x) / 2.;
    let border_y = (map_dimension.y - playable_dimensions.y) / 2.;
    if x < border_x || (map_dimension.x - x) < border_x {
        return Color::from(palettes::tailwind::SLATE_500);
    }
    if y < border_y || (map_dimension.y - y) < border_y {
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
