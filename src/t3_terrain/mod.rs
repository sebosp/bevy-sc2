use crate::BevySC2MapError;
use crate::MAP_SCALE_FACTOR;
use crate::MapScene;
use bevy::gltf::GltfMaterial;
use bevy::prelude::*;
use nom::IResult;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;

/// A copy of the T3Terrain that impls Reflect, Resource.
#[derive(Resource, Default, Reflect, Debug)]
#[reflect(Resource, Default)]
pub struct T3TerrainResource {
    pub version: u32,
    pub ramp_list: Vec<RampResource>,
}

#[derive(Resource, Default, Reflect, Debug)]
#[reflect(Resource, Default)]
pub enum RampDirection {
    #[default]
    South,
    North,
    West,
    East,
    SouthWest,
    SouthEast,
    NorthWest,
    NorthEast,
}

impl TryFrom<u8> for RampDirection {
    type Error = BevySC2MapError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::South),
            1 => Ok(Self::North),
            2 => Ok(Self::West),
            3 => Ok(Self::East),
            4 => Ok(Self::SouthWest),
            5 => Ok(Self::SouthEast),
            6 => Ok(Self::NorthWest),
            7 => Ok(Self::NorthEast),
            _ => Err(BevySC2MapError::Other(
                "Unknown Ramp Direction.".to_string(),
            )),
        }
    }
}

/// A copy of the T3Terrain that impls Reflect, Resource.
#[derive(Resource, Default, Reflect, Debug)]
#[reflect(Resource, Default)]
pub struct RampResource {
    pub dir: RampDirection,
    /// Looks like cell layer/height
    pub hi: u8,
    pub lo: u8,
    // "u(-1.000000e+00, 0.000000e+00) r(0.000000e+00, 1.000000e+00) c=(1.420000e+02, 4.400000e+01) w=2.000000e+00 h=2.000000e+00"
    // Looks SVG-ish, maybe u=up r=right c=center w=width h=height ?
    pub left_lo: Vec2,
    pub left_hi: String,
    pub right_lo: String,
    pub right_hi: String,
    pub base: String,
    pub mid: String,
    pub cid: usize,
    pub left_lo_var: u32,
    pub left_hi_var: u32,
    pub right_lo_var: u32,
    pub right_hi_var: u32,
}

impl TryFrom<s2protocol::cache_handles::t3_terrain::T3Terrain> for T3TerrainResource {
    type Error = BevySC2MapError;
    fn try_from(
        input: s2protocol::cache_handles::t3_terrain::T3Terrain,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            version: input.version,
            ramp_list: input
                .height_map
                .ramp_list
                .inner
                .into_iter()
                .filter_map(|x| x.try_into().ok())
                .collect(),
        })
    }
}

impl TryFrom<s2protocol::cache_handles::t3_terrain::Ramp> for RampResource {
    type Error = BevySC2MapError;
    fn try_from(input: s2protocol::cache_handles::t3_terrain::Ramp) -> Result<Self, Self::Error> {
        let (_, (left_lo_str_x, left_lo_str_y)) = parse_c_x_y(&input.left_lo)?;
        let (_, left_lo) = c_x_y_to_vec2(left_lo_str_x, left_lo_str_y)?;
        Ok(Self {
            dir: input.dir.try_into()?,
            hi: input.hi,
            lo: input.lo,
            left_lo: left_lo,
            left_hi: input.left_hi,
            right_lo: input.right_lo,
            right_hi: input.right_hi,
            base: input.base,
            mid: input.mid,
            cid: input.cid,
            left_lo_var: input.left_lo_var,
            left_hi_var: input.left_hi_var,
            right_lo_var: input.right_lo_var,
            right_hi_var: input.right_hi_var,
        })
    }
}

/// The Ramp contains x,y inside c=(x,y)
/// u(0.000000e+00, -1.000000e+00) r(-1.000000e+00, 0.000000e+00) c=(4.800000e+01, 2.600000e+01) w=2.000000e+00 h=2.000000e+00
/// There are maybe 10 maybe 100 ramps per maps and it's only read once, maybe String is fine by now.
fn parse_c_x_y(s: &str) -> IResult<&str, (String, String)> {
    let (tail, _) = take_until("c=(")(s)?;
    let (tail, _) = tag("c=(")(tail)?;
    let (tail, x) = take_until(", ")(tail)?;
    let (tail, _) = tag(", ")(tail)?;
    let (tail, y) = take_until(") ")(tail)?;
    Ok((tail, (x.to_string(), y.to_string())))
}

fn c_x_y_to_vec2(x_str: String, y_str: String) -> Result<Vec2, BevySC2MapError> {
    Ok(Vec2::new(x_str.parse()?, y_str.parse()?))
}

/// Loads the t3 height map.
pub fn load_t3_terrain(
    mut _commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    map_scene: Res<MapScene>,
    gltf_assets: Res<Assets<Gltf>>,
    _gltf_materials: Res<Assets<GltfMaterial>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    mut loaded: Local<bool>,
) -> Result<(), BevyError> {
    // Only do this once
    if *loaded {
        return Ok(());
    }
    // Wait until the scene is loaded
    let Some(_gltf) = gltf_assets.get(&map_scene.0) else {
        return Ok(());
    };

    let source = "/home/seb/SC2Replays/swarmy/extract/a76deb95741e1d3d24527f0a303914824455bc9d68411fa143d23cc4edee9c27/a76deb95741e1d3d24527f0a303914824455bc9d68411fa143d23cc4edee9c27.s2ma".to_string();
    let (mpq, cache_contents) = s2protocol::read_mpq(&source)?;
    let t3_terrain_xml =
        s2protocol::cache_handles::t3_terrain::T3Terrain::from_mpq(&mpq, &cache_contents)?;
    let mineral_mesh = Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(
        MAP_SCALE_FACTOR,
        MAP_SCALE_FACTOR,
        MAP_SCALE_FACTOR,
    ))));
    let xel_naga_mesh = Mesh3d(meshes.add(Cylinder::new(
        10. * MAP_SCALE_FACTOR,
        10. * MAP_SCALE_FACTOR,
    )));
    let destructible_rock_ex1_diagonal_huge_blur_material_mesh = Mesh3d(meshes.add(Cylinder::new(
        10. * MAP_SCALE_FACTOR,
        10. * MAP_SCALE_FACTOR,
    )));
    /*
    // Id="1035" Position="6.0996,150.3146,0" Scale="1,1,1" Type="NoFlyZone" Name="No Fly Zone 011" Color="0,0,0,0" PathingRadiusSoft="5" PathingRadiusHard="4"
    for ramp in placed_objects.points {
        // These are objects in the map, decorations, animation references, etc.
        let unit_pos: Vec<f32> = object_point
            .position
            .split(",")
            .filter_map(|x| x.parse::<f32>().ok())
            .collect();
        if unit_pos.len() != 3 {
            tracing::error!(
                "Unexpected number of tokens for unit position typed: {}",
                object_point.kind
            );
            continue;
        }
        let x = unit_pos[0];
        let y = unit_pos[1];
        let z = unit_pos[2];
        let target_vec_pos = y as usize * t3_height_map_res.width as usize + x as usize;
        let cell_height = t3_height_map_res.data[target_vec_pos];

        let Some(unit_handle) = gltf.named_materials.get(object_point.kind.as_str()) else {
            tracing::warn!(
                "Unhandled Skein GLTF named_material ObjectPoint: {}",
                object_point.kind
            );
            continue;
        };
        let Some(unit_gltf_material) = gltf_materials.get(unit_handle.id()) else {
            tracing::warn!(
                "Unhandled Skein GLTF gltf_material ObjectPoint: {}",
                object_point.kind
            );
            continue;
        };
        let object_point_material = MeshMaterial3d(
            materials.add(standard_material_from_gltf_material(&unit_gltf_material)),
        );
        let pathing_radius_soft = object_point.pathing_radius_soft as f32;
        let _pathing_radius_hard = object_point.pathing_radius_hard as f32;
        let torus_mesh = Mesh3d(meshes.add(Torus::new(0.2, 0.25)));
        let cylinder_mesh = Mesh3d(meshes.add(Cylinder::new(
            pathing_radius_soft * NO_FLY_ZONE_RADIUS * MAP_SCALE_FACTOR,
            NO_FLY_ZONE_HEIGHT * MAP_SCALE_FACTOR,
        )));
        let cylinder_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * y,
            z + cell_height as f32 * CELL_HEIGHT_MULTIPLIER * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * x,
        );
        // TODO: We should use the alpha channel, dunno if we need glsl for that tho because it's
        // being set from blender and exported to gltf.
        match object_point.kind.as_str() {
            "NoFlyZone" => commands.spawn((
                NoFlyZoneMaterial,
                cylinder_mesh,
                object_point_material,
                cylinder_transform,
            )),
            "StartLoc" => commands.spawn((
                StartLocMaterial,
                torus_mesh,
                object_point_material,
                cylinder_transform,
            )),
            _ => commands.spawn((
                UnknownObjectPointMaterial,
                shadow_platform_ramp_mesh.clone(),
                cylinder_mesh,
                cylinder_transform,
            )),
        };
    }*/
    *loaded = true;
    Ok(())
}
