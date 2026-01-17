use bevy::color::palettes;
use bevy::prelude::*;
use bevy_sc2_map::*;

use bevy::log::tracing;
use bevy::time::common_conditions::once_after_delay;
use bevy_http_client::prelude::*;
use camera_controller::{CameraController, CameraControllerPlugin};

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);

        app.add_systems(
            Update,
            send_request.run_if(once_after_delay(std::time::Duration::from_secs(1))),
        );
        app.add_systems(Update, (handle_binary_response, handle_error));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HttpClientPlugin)
        .add_plugins(CameraControllerPlugin)
        .add_plugins(MapPlugin)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // example instructions
    commands.spawn((
        Text::new(
            "Controls \n\
            WASD/Mouse move\n\
            T/P\n\
            B/1\n\
            Up/Down",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            max_height: Val::Px(200.),
            max_width: Val::Px(300.),
            ..default()
        },
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(40.0, 80.0, 40.0),
    ));
}

// fn draw_example_collection(mut gizmos: Gizmos) {
//     gizmos.grid(
//         Isometry3d::new(Vec3::new(8., 0., 8.), Quat::from_rotation_x(PI / 2.)),
//         UVec2::splat(20),
//         Vec2::new(1., 1.),
//         // Light gray
//         LinearRgba::gray(0.65),
//     );
// }

// fn update_config(
//     mut config_store: ResMut<GizmoConfigStore>,
//     keyboard: Res<ButtonInput<KeyCode>>,
//     time: Res<Time>,
// ) {
//     if keyboard.just_pressed(KeyCode::KeyT) {
//         for (_, config, _) in config_store.iter_mut() {
//             config.depth_bias = if config.depth_bias == 0. { -1. } else { 0. };
//         }
//     }
//     if keyboard.just_pressed(KeyCode::KeyP) {
//         for (_, config, _) in config_store.iter_mut() {
//             // Toggle line perspective
//             config.line.perspective ^= true;
//             // Increase the line width when line perspective is on
//             config.line.width *= if config.line.perspective { 5. } else { 1. / 5. };
//         }
//     }

//     let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
//     if keyboard.pressed(KeyCode::ArrowRight) {
//         config.line.width += 5. * time.delta_secs();
//         config.line.width = config.line.width.clamp(0., 50.);
//     }
//     if keyboard.pressed(KeyCode::ArrowLeft) {
//         config.line.width -= 5. * time.delta_secs();
//         config.line.width = config.line.width.clamp(0., 50.);
//     }
//     if keyboard.just_pressed(KeyCode::Digit1) {
//         config.enabled ^= true;
//     }
//     if keyboard.just_pressed(KeyCode::KeyU) {
//         config.line.style = match config.line.style {
//             GizmoLineStyle::Solid => GizmoLineStyle::Dotted,
//             GizmoLineStyle::Dotted => GizmoLineStyle::Dashed {
//                 gap_scale: 3.0,
//                 line_scale: 5.0,
//             },
//             _ => GizmoLineStyle::Solid,
//         };
//     }
//     if keyboard.just_pressed(KeyCode::KeyJ) {
//         config.line.joints = match config.line.joints {
//             GizmoLineJoint::Bevel => GizmoLineJoint::Miter,
//             GizmoLineJoint::Miter => GizmoLineJoint::Round(4),
//             GizmoLineJoint::Round(_) => GizmoLineJoint::None,
//             GizmoLineJoint::None => GizmoLineJoint::Bevel,
//         };
//     }

//     let (my_config, _) = config_store.config_mut::<MyRoundGizmos>();
//     if keyboard.pressed(KeyCode::ArrowUp) {
//         my_config.line.width += 5. * time.delta_secs();
//         my_config.line.width = my_config.line.width.clamp(0., 50.);
//     }
//     if keyboard.pressed(KeyCode::ArrowDown) {
//         my_config.line.width -= 5. * time.delta_secs();
//         my_config.line.width = my_config.line.width.clamp(0., 50.);
//     }
//     if keyboard.just_pressed(KeyCode::Digit2) {
//         my_config.enabled ^= true;
//     }
//     if keyboard.just_pressed(KeyCode::KeyI) {
//         my_config.line.style = match my_config.line.style {
//             GizmoLineStyle::Solid => GizmoLineStyle::Dotted,
//             GizmoLineStyle::Dotted => GizmoLineStyle::Dashed {
//                 gap_scale: 3.0,
//                 line_scale: 5.0,
//             },
//             _ => GizmoLineStyle::Solid,
//         };
//     }
//     if keyboard.just_pressed(KeyCode::KeyK) {
//         my_config.line.joints = match my_config.line.joints {
//             GizmoLineJoint::Bevel => GizmoLineJoint::Miter,
//             GizmoLineJoint::Miter => GizmoLineJoint::Round(4),
//             GizmoLineJoint::Round(_) => GizmoLineJoint::None,
//             GizmoLineJoint::None => GizmoLineJoint::Bevel,
//         };
//     }

//     if keyboard.just_pressed(KeyCode::KeyB) {
//         // AABB gizmos are normally only drawn on entities with a ShowAabbGizmo component
//         // We can change this behavior in the configuration of AabbGizmoGroup
//         config_store.config_mut::<AabbGizmoConfigGroup>().1.draw_all ^= true;
//     }
// }

fn send_request(mut ev_request: MessageWriter<HttpRequest>) {
    tracing::info!("Requesting");
    // One of the files from the downloaded cache_handles, not all the handles will contain the
    // t3HeightMap or MapInfo, others seem to have just strings as information such as "SC2 Mod"
    let s2_mpq_cache: &str =
        "assets/s2matest/300d0946f3f5bcd955b533e7acac0dd22445339b38a837efcca7ebe2d93badca.s2ma";
    if let Ok(request) = HttpClient::new()
        .get(format!("https://sebosp.github.io/bevy-sc2/{s2_mpq_cache}"))
        .try_build()
    {
        ev_request.write(request);
    }
}

/// consume TypedResponse<IpInfo> events
fn handle_binary_response(
    mut events: ResMut<Messages<HttpResponse>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    tracing::info!("Handle response");
    for response in events.drain() {
        tracing::info!("response: {:?}", response);
        let cache_contents = &response.bytes;
        // based on sc2-map-analyzer/analyser/read.cpp
        let (_input, mpq) = nom_mpq::parser::parse(cache_contents).unwrap();
        let map_info = MapInfo::from_mpq(&mpq, cache_contents).unwrap();
        tracing::info!("Map Info: {map_info:?}");
        let t3_height_map = T3HeightMap::from_mpq(&mpq, cache_contents, &map_info).unwrap();
        let map_size = t3_height_map.width.max(t3_height_map.height) as f32 * 0.1;
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(map_size * 2., map_size * 1.5, map_size)
                .looking_at(Vec3::new(map_size / 2., 0.0, map_size / 2.), Vec3::Y),
            CameraController::default(),
        ));
        // cube
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));
        // light
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::new(
                *Dir3::Y,
                Vec2::new(
                    t3_height_map.width as f32 / 2. / 10.,
                    t3_height_map.height as f32 / 2. / 10.,
                ),
            ))),
            MeshMaterial3d(materials.add(Color::linear_rgba(0.88, 0.88, 0.88, 1.))),
            Transform::from_xyz(
                t3_height_map.width as f32 / 2. / 10.,
                0.,
                t3_height_map.height as f32 / 2. / 10.,
            ),
        ));
        // cube
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
}

fn handle_error(mut ev_error: MessageReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving S2MpqArchive: {}", error.err);
    }
}
