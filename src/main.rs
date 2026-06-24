use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;
use clap::Parser;
use swarmy_bevy::MapScene;
use swarmy_bevy::t3_height_map::load_t3_height_map;
use swarmy_bevy::t3_terrain::load_t3_terrain;
use swarmy_bevy::utils::*;
use swarmy_bevy::{cli::*, load_cache_objects};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_light_and_gizmo_control_text);
        app.add_systems(Startup, load_t3_height_map);
        app.add_systems(Update, load_cache_objects);
        app.add_systems(Update, load_t3_terrain);
        app.add_systems(Update, update_gizmo_config);
    }
}

/// Show some text if there's a current action
#[derive(Default, Resource, Reflect)]
pub struct ActivityStage(String);

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
        .add_systems(PreStartup, load_gltf)
        .run();
}

fn setup_light_and_gizmo_control_text(mut commands: Commands) {
    // light
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
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
            B for AABB\n\
            M for lock/unlock mouse navigation",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        TextFont {
            font_size: bevy::prelude::FontSize::Px(11.),
            ..default()
        },
    ));
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "api"]
struct Character {
    name: String,
}

fn load_gltf(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the blender scene.
    /*commands.spawn((
        WorldAssetRoot(
            // SwarmyObjects handle
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("swarmy-objects.gltf")),
        ),
        Transform::from_xyz(10., -2.5, 10.).with_scale(Vec3 {
            x: 10.,
            y: 2.,
            z: 10.,
        }),
    ));*/
    let gltf = asset_server.load("swarmy-objects.gltf");
    commands.insert_resource(MapScene(gltf));
}
