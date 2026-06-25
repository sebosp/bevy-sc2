use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::feathers::{
    FeathersPlugins,
    controls::*,
    dark_theme::create_dark_theme,
    theme::{ThemedText, UiTheme},
};
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
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
        app.add_systems(Startup, MapPlugin::scene.spawn());
        app.add_systems(Startup, setup_light_and_gizmo_control_text);
        app.add_systems(Startup, load_t3_height_map);
        app.add_systems(Update, load_cache_objects);
        app.add_systems(Update, load_t3_terrain);
        app.add_systems(Update, update_gizmo_config);
    }
}
impl MapPlugin {
    fn scene() -> impl SceneList {
        bsn_list![init_feathers()]
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
        .add_plugins(FeathersPlugins)
        .add_plugins(SkeinPlugin::default())
        .add_plugins(FreeCameraPlugin)
        .add_plugins(MapPlugin)
        .insert_resource(UiTheme(create_dark_theme()))
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
            bottom: px(12),
            left: px(12),
            ..default()
        },
        TextFont {
            font_size: bevy::prelude::FontSize::Px(11.),
            ..default()
        },
    ));
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

fn init_feathers() -> impl Scene {
    bsn! {
        Node {
            width: percent(10),
            height: percent(10),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
        }
        TabGroup
        Children[
            feather_column_1(),
        ]
    }
}
fn feather_column_1() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            padding: px(8),
            row_gap: px(8),
            width: percent(30),
            min_width: px(200),
        }
        Children [
            (
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    column_gap: px(8),
                }
                Children [
                    main_feather_menu(),
                ]
            )
        ]
    }
}

fn main_feather_menu() -> impl Scene {
    bsn! {
        (
            @FeathersMenu
            Children [
                (
                    @FeathersMenuButton {
                        @caption: bsn! { Text("View") ThemedText }
                    }
                    AccessibleLabel("View Menu")
                    Node {
                        flex_grow: 1.0,
                    }
                ),
                (
                    @FeathersMenuPopup
                    Children [
                        (
                            @FeathersCheckbox {
                                @caption: bsn! { Text("Enable ObjectDoodas") ThemedText }
                            }
                            on(|_: On<Activate>| {
                                info!("Enabling ObjectDoodas");
                            })
                        ),
                        (
                            @FeathersCheckbox {
                                @caption: bsn! { Text("Enable ObjectUnits") ThemedText }
                            }
                            on(|_: On<Activate>| {
                                info!("Enabling ObjectUnits");
                            })
                        ),
                        (
                            @FeathersCheckbox {
                                @caption: bsn! { Text("Enable ObjectPoints") ThemedText }
                            }
                            on(|_: On<Activate>| {
                                info!("Enabling ObjectPoints");
                            })
                        ),
                        @FeathersMenuDivider,
                        (
                            @FeathersMenuItem {
                                @caption: bsn! { Text("Second section") ThemedText }
                            }
                        )
                    ]
                )
            ]
        )
    }
}
