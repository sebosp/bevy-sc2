use std::fs::File;
use std::io::prelude::*;

use bevy::color::palettes;
use bevy::color::palettes::css::{GOLD, GREEN, RED};
use bevy::prelude::*;
use chrono::DateTime;
use clap::Parser;
use swarmy_bevy::*;

use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::log::tracing;
use tracing::instrument;

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
    map_title: String,
    snapshot_path: String,
    cache_handle_ids: String,
    document_header: DocumentHeader,
}

/// Parse cli args.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The name of the map.
    #[arg(short, long)]
    map_title: String,

    /// The path of the snapshot, a user should have already clicked on download caches.
    #[arg(short, long)]
    snapshot_path: String,
    /// A comma-separated list of caches to inspect for the map data.
    /// When a replay is loaded it contains multiple ids for different purposes.
    /// I assume there's only one t3HeightMap and only one MapInfo sector.
    #[arg(short, long)]
    cache_handle_ids: String,
}

fn main() {
    let args = Args::parse();
    // store the name in a resource so we can access it in our systems

    let snapshot_path = args.snapshot_path.trim_end_matches('/').to_string();
    App::new()
        .insert_resource(CliParams {
            map_title: args.map_title,
            snapshot_path,
            cache_handle_ids: args.cache_handle_ids,
            document_header: DocumentHeader::default(),
        })
        .init_gizmo_group::<MyRoundGizmos>()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(MapPlugin)
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
    map_title: &str,
    cache_handle_fname: &str,
) -> Result<(MapInfo, T3HeightMap, Option<DocumentHeader>), BevySC2MapError> {
    let mut document_header: Option<DocumentHeader> = None;
    let cache_contents = read_mpq_file(&cache_handle_fname)?;
    // based on sc2-map-analyzer/analyser/read.cpp
    let (_input, mpq) = nom_mpq::parser::parse(&cache_contents)?;
    for (file, _file_size) in mpq.get_files(&cache_contents)? {
        if file == "DocumentHeader" {
            if let Ok(docu_header) = DocumentHeader::from_mpq(&mpq, &cache_contents) {
                document_header = Some(docu_header);
            }
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
) {
    let mut t3_height_map: Option<T3HeightMap> = None;
    let mut map_info: Option<MapInfo> = None;
    let mut document_header: Option<DocumentHeader> = None;
    for cache_handle_id in cli_params.cache_handle_ids.split(",") {
        if cache_handle_id.is_empty() {
            continue;
        }
        let cache_handle_fname = format!("{}/{}.s2ma", cli_params.snapshot_path, cache_handle_id);
        info!("Checking cache_handle_fname: {}", cache_handle_fname);
        if let Ok((map, height, docu_header)) =
            try_get_t3_height_map_from_mpq(&cli_params.map_title, &cache_handle_fname)
        {
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
                cli_params.snapshot_path
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
                cli_params.snapshot_path,
            );
            commands.spawn((
                Text::new(format!(
                    "Unable to find cache handles from input: {} {}",
                    cli_params.snapshot_path, cli_params.cache_handle_ids
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

    let dim_playable = map_info.cell_dim_playable();
    commands.spawn((
        Text::new(format!(
            "{}\n\
                {} - {}\n\
                MapInfo Dimensions: {} - {}\n\
                TerrainHeight Dimensions: {} - {}",
            cli_params.map_title,
            map_info.third_string,
            map_info.fourth_string,
            dim_playable.x,
            dim_playable.y,
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
    if let Some(docu_header) = document_header {
        let maybe_dt_1 = match DateTime::from_timestamp_secs(docu_header.some_epoch_1 as i64) {
            Some(val) => val.to_string(),
            None => docu_header.some_epoch_1.to_string(),
        };
        let maybe_dt_2 = match DateTime::from_timestamp_secs(docu_header.some_epoch_2 as i64) {
            Some(val) => val.to_string(),
            None => docu_header.some_epoch_2.to_string(),
        };

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
                left: px(12),
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
        let color = if *cell_height == 0 {
            palettes::css::BLACK
        } else if *cell_height == 1 {
            palettes::css::LIGHT_BLUE
        } else if *cell_height == 2 {
            palettes::css::LIGHT_GOLDENROD_YELLOW
        } else if *cell_height == 3 {
            palettes::css::DARK_GREEN
        } else {
            palettes::css::DARK_RED
        };
        let color = Color::from(color);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(0.1, *cell_height as f32, 0.1)))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(y as f32 / 10., 1., x as f32 / 10.),
        ));
    }
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
