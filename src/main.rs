use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_skein::SkeinPlugin;
use clap::Parser;
use swarmy_bevy::cli::*;
use swarmy_bevy::t3_height_map::load_t3_height_map;
use swarmy_bevy::utils::*;

#[derive(Default, bevy::reflect::Reflect, GizmoConfigGroup)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Startup, load_t3_height_map);
        app.add_systems(Update, update_gizmo_config);
    }
}

/// Show some text if there's a current action
#[derive(Default, Resource, bevy::reflect::Reflect)]
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

#[derive(Component, Default, bevy::reflect::Reflect, Debug)]
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
