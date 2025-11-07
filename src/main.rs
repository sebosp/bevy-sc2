use crate::map_info::MapInfo;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::tracing;
use bevy::prelude::*;

use bevy_sc2_map::*;
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MapPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // One of the files from the downloaded cache_handles, not all the handles will contain the
    // t3HeightMap or MapInfo, others seem to have just strings as information such as "SC2 Mod"
    let s2_mpq_cache: &str =
        "./assets/s2matest/300d0946f3f5bcd955b533e7acac0dd22445339b38a837efcca7ebe2d93badca.s2ma";
    let cache_contents = nom_mpq::parser::read_file(s2_mpq_cache);
    // based on sc2-map-analyzer/analyser/read.cpp
    let (_input, mpq) = nom_mpq::parser::parse(&cache_contents).unwrap();
    let map_info = MapInfo::from_mpq(s2_mpq_cache, &mpq, &cache_contents).unwrap();
    tracing::info!("Map Info: {map_info:?}");
    let t3_height_map =
        T3HeightMap::from_mpq(s2_mpq_cache, &mpq, &cache_contents, &map_info).unwrap();
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(
            *Dir3::Z,
            Vec2::new(t3_height_map.width as f32, t3_height_map.height as f32),
        ))),
        MeshMaterial3d(materials.add(Color::linear_rgba(0.88, 0.88, 0.88, 1.))),
        Transform::from_xyz(
            -t3_height_map.width as f32 / 2.,
            -t3_height_map.height as f32 / 2.,
            0.,
        ),
    ));
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    for (idx, cell_height) in t3_height_map.data.iter().enumerate() {
        let x = (idx as i32) % t3_height_map.width;
        let y = idx as i32 / t3_height_map.width;
        let color = if *cell_height == 0 {
            Color::srgb_u8(124, 144, 255)
        } else if *cell_height == 1 {
            Color::srgb_u8(124, 144, 124)
        } else if *cell_height == 2 {
            Color::srgb_u8(124, 255, 124)
        } else if *cell_height == 3 {
            Color::srgb_u8(124, 255, 255)
        } else {
            Color::srgb_u8(255, 125, 125)
        };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(x as f32, y as f32, *cell_height as f32),
        ));
    }
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(40.0, 80.0, 40.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-200., 200., 200.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}
