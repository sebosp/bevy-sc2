use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::color::palettes;
use bevy::color::palettes::css::{GOLD, GREEN, RED};
use bevy::log::tracing;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_skein::SkeinPlugin;
use chrono::DateTime;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use tracing::instrument;

use swarmy_bevy::*;

pub const MAP_SCALE_FACTOR: f32 = 10.;

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Startup, load_t3_height_map);
        app.add_systems(Update, update_config);
    }
}

/// Show some text if there's a current action
#[derive(Default, Resource, Reflect)]
pub struct ActivityStage(String);

#[derive(Default, Resource, Reflect)]
pub struct CliParams {
    path: String,
    ids: String,
}

/// Parse cli args.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path where the caches have been downloaded to.
    /// A user should have already clicked on download caches.
    /// This is set in the Config section of Swarmy.
    #[arg(short, long)]
    path: String,
    /// A comma-separated list of caches to inspect for the map data.
    /// When a replay is loaded it contains multiple ids for different purposes.
    /// I assume there's only one t3HeightMap and only one MapInfo sector.
    #[arg(short, long)]
    ids: String,
}

fn main() {
    let args = Args::parse();
    // store the name in a resource so we can access it in our systems

    let path = args.path.trim_end_matches('/').to_string();
    App::new()
        .insert_resource(CliParams {
            path,
            ids: args.ids,
        })
        .init_gizmo_group::<MyRoundGizmos>()
        .add_plugins(DefaultPlugins)
        .add_plugins(SkeinPlugin::default())
        .add_plugins(FreeCameraPlugin)
        .add_plugins(MapPlugin)
        .add_observer(
            // log the component from the gltf spawn
            |ready: On<SceneInstanceReady>,
             children: Query<&Children>,
             characters: Query<&Character>| {
                for entity in children.iter_descendants(ready.entity) {
                    let Ok(character) = characters.get(entity) else {
                        continue;
                    };
                    info!(?character);
                }
            },
        )
        .add_systems(Startup, startup)
        .run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // example instructions
    commands.spawn((
        Text::new(
            "Controls:\n\
            W/A/S/D to move (Shift for speed)\n\
            Mouse drag (scroll for speed)\n\
            B for AABB",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        TextFont {
            font_size: 11.,
            ..default()
        },
    ));
}

/// Attempts to read the s2ma file.
#[instrument]
pub fn read_mpq_file(path: &str) -> Result<Vec<u8>, BevySC2MapError> {
    tracing::info!("Opening file.");
    let mut f = File::open(path)?;
    tracing::info!("Reading into buffer.");
    let mut buffer: Vec<u8> = vec![];
    // read the whole file
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
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

/// set up a simple 3D scene
fn load_t3_height_map(
    mut commands: Commands,
    cli_params: Res<CliParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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
                Download Caches button in swarmy app -> Scan tab.",
                cli_params.path
            )),
            Node {
                position_type: PositionType::Absolute,
                top: px(200),
                left: px(200),
                ..default()
            },
            TextColor(Color::from(RED)),
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
                TextColor(Color::from(RED)),
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
        TextColor(Color::from(GREEN)),
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
            TextColor(Color::from(GOLD)),
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
                pos_x: y / 10.,
                pos_y: 1.,
                pos_z: x / 10.,
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
            Transform::from_xyz(y / 10., *cell_height as f32 * 0.25, x / 10.),
        ));
    }

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
                    let (x, y) = playable_dimensions_to_t3_map_dimensions(
                        (unit_pos[0], unit_pos[1]),
                        map_dimension,
                        playable_dimensions,
                    );
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(0.1, 2., 0.1)))),
                        MeshMaterial3d(materials.add(unit_color)),
                        //SceneRoot(asset_server.load(GltfAssetLabel::Mesh(0).from_asset("swarmy-objects.gltf"))),
                        // TODO: The camera coordinate space is right-handed X-right, Y-up, Z-back.
                        // This is probably not the way to deal with the camera coords...
                        Transform::from_xyz(0.1 * x, 1., 0.1 * y),
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

/// Transforms PlayableCoordinates into T3HeightMap Dimensions
fn playable_dimensions_to_t3_map_dimensions(
    (x, y): (f32, f32),
    t3_map_size: MapDimension,
    playable_dimensions: MapDimension,
) -> (f32, f32) {
    (
        t3_map_size.x * (x / playable_dimensions.x),
        t3_map_size.y * (y / playable_dimensions.y),
    )
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
struct TerrainCell {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    scl_x: f32,
    scl_y: f32,
    scl_z: f32,
}

fn update_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
    real_time: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        for (_, config, _) in config_store.iter_mut() {
            config.depth_bias = if config.depth_bias == 0. { -1. } else { 0. };
        }
    }
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    if keyboard.just_pressed(KeyCode::KeyU) {
        config.line.style = match config.line.style {
            GizmoLineStyle::Solid => GizmoLineStyle::Dotted,
            GizmoLineStyle::Dotted => GizmoLineStyle::Dashed {
                gap_scale: 3.0,
                line_scale: 5.0,
            },
            _ => GizmoLineStyle::Solid,
        };
    }
    if keyboard.just_pressed(KeyCode::KeyJ) {
        config.line.joints = match config.line.joints {
            GizmoLineJoint::Bevel => GizmoLineJoint::Miter,
            GizmoLineJoint::Miter => GizmoLineJoint::Round(4),
            GizmoLineJoint::Round(_) => GizmoLineJoint::None,
            GizmoLineJoint::None => GizmoLineJoint::Bevel,
        };
    }

    let (my_config, _) = config_store.config_mut::<MyRoundGizmos>();
    if keyboard.pressed(KeyCode::ArrowUp) {
        my_config.line.width += 5. * real_time.delta_secs();
        my_config.line.width = my_config.line.width.clamp(0., 50.);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        my_config.line.width -= 5. * real_time.delta_secs();
        my_config.line.width = my_config.line.width.clamp(0., 50.);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        my_config.enabled ^= true;
    }
    if keyboard.just_pressed(KeyCode::KeyI) {
        my_config.line.style = match my_config.line.style {
            GizmoLineStyle::Solid => GizmoLineStyle::Dotted,
            GizmoLineStyle::Dotted => GizmoLineStyle::Dashed {
                gap_scale: 3.0,
                line_scale: 5.0,
            },
            _ => GizmoLineStyle::Solid,
        };
    }
    if keyboard.just_pressed(KeyCode::KeyK) {
        my_config.line.joints = match my_config.line.joints {
            GizmoLineJoint::Bevel => GizmoLineJoint::Miter,
            GizmoLineJoint::Miter => GizmoLineJoint::Round(4),
            GizmoLineJoint::Round(_) => GizmoLineJoint::None,
            GizmoLineJoint::None => GizmoLineJoint::Bevel,
        };
    }

    if keyboard.just_pressed(KeyCode::KeyB) {
        // AABB gizmos are normally only drawn on entities with a ShowAabbGizmo component
        // We can change this behavior in the configuration of AabbGizmoGroup
        config_store.config_mut::<AabbGizmoConfigGroup>().1.draw_all ^= true;
    }
    if keyboard.just_pressed(KeyCode::Space) {
        if virtual_time.is_paused() {
            virtual_time.unpause();
        } else {
            virtual_time.pause();
        }
    }
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
struct Character {
    name: String,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneRoot(asset_server.load(
        // Change this to your exported gltf file
        GltfAssetLabel::Scene(0).from_asset("swarmy-objects.gltf"),
    )));
}
