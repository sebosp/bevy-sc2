use std::fs::File;
use std::io::prelude::*;

use bevy::color::palettes;
use bevy::color::palettes::css::{GREEN, RED};
use bevy::prelude::*;
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

#[derive(Default, Resource, Reflect)]
pub struct SnapshotPath(String);

#[derive(Default, Resource, Reflect)]
pub struct CacheHandleIds(String);

/// Parse cli args.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
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
    let snapshot_path = args.snapshot_path;
    let cache_handle_id = args.cache_handle_ids;
    // store the name in a resource so we can access it in our systems

    App::new()
        .insert_resource(SnapshotPath(snapshot_path))
        .insert_resource(CacheHandleIds(cache_handle_id))
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
            Mouse drag\n\
            Mouse scroll for move speed",
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
) -> Result<(MapInfo, T3HeightMap), BevySC2MapError> {
    tracing::info!("Starting...");
    let cache_contents = read_mpq_file(&cache_handle_fname)?;
    // based on sc2-map-analyzer/analyser/read.cpp
    tracing::info!("MPQ file read, parsing...");
    let (_input, mpq) = nom_mpq::parser::parse(&cache_contents)?;
    for file in mpq.get_files(&cache_contents)? {
        tracing::info!("--- {:?}", file);
    }
    tracing::info!("Reading MapInfo from mpq...");
    let map_info = MapInfo::from_mpq(&mpq, &cache_contents)?;
    tracing::info!("Map Info: {map_info:?}");
    let t3_height_map = T3HeightMap::from_mpq(&mpq, &cache_contents, &map_info)?;
    Ok((map_info, t3_height_map))
}

/// set up a simple 3D scene
fn load_t3_height_map(
    mut commands: Commands,
    snapshot_path: Res<SnapshotPath>,
    cache_handle_ids: Res<CacheHandleIds>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut t3_height_map: Option<T3HeightMap> = None;
    let mut map_info: Option<MapInfo> = None;
    info!("Got input: {}", cache_handle_ids.0);
    for cache_handle_id in cache_handle_ids.0.split(",") {
        if cache_handle_id.is_empty() {
            continue;
        }
        let cache_handle_fname = format!("{}/{}.s2ma", snapshot_path.0, cache_handle_id);
        info!("Checking cache_handle_fname: {}", cache_handle_fname);
        if let Ok((map, height)) = try_get_t3_height_map_from_mpq(&cache_handle_fname) {
            map_info = Some(map);
            t3_height_map = Some(height);
        }
    }
    let map_info = if let Some(val) = map_info {
        val
    } else {
        commands.spawn((
            Text::new(format!(
                "Unable to find MapInfo input: {} on directory {}",
                snapshot_path.0, cache_handle_ids.0
            )),
            Node {
                position_type: PositionType::Absolute,
                top: px(12),
                left: px(500),
                ..default()
            },
            TextColor(Color::from(RED)),
            TextLayout::new_with_justify(Justify::Center),
        ));
        return;
    };
    let dim_playable = map_info.cell_dim_playable();
    commands.spawn((
        Text::new(format!(
            "{} - {}\nPlayable Dimensions: {} - {}",
            map_info.third_string, map_info.fourth_string, dim_playable.x, dim_playable.y,
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

    let t3_height_map = match t3_height_map {
        Some(val) => {
            debug!("Found t3_height_map.");
            val
        }
        None => {
            error!(
                "Unable to find cache handles from input: {} {}",
                snapshot_path.0, cache_handle_ids.0
            );
            commands.spawn((
                Text::new(format!(
                    "Unable to find cache handles from input: {} {}",
                    snapshot_path.0, cache_handle_ids.0
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
    let map_size = t3_height_map.width.max(t3_height_map.height) as f32 * 0.1;
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
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 4.0, 4.0),
    ));
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
