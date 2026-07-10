use crate::BevySC2MapError;
use crate::MAP_SCALE_FACTOR;
use crate::swarmy_feathers::DisplayInfoOnClick;
use crate::swarmy_feathers::update_info_on_click;
use crate::t3_height_map::CELL_HEIGHT_MULTIPLIER;
use crate::t3_height_map::T3HeightMapResource;
use bevy::color::palettes;
use bevy::prelude::*;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;

pub const RAMP_SIZE: f32 = 1.0;
pub const RAMP_HEIGHT: f32 = 1.0;

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
    pub left_hi: Vec2,
    pub right_lo: Vec2,
    pub right_hi: Vec2,
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
        let left_lo = parse_c_x_y(&input.left_lo)?;
        let left_hi = parse_c_x_y(&input.left_hi)?;
        let right_lo = parse_c_x_y(&input.right_lo)?;
        let right_hi = parse_c_x_y(&input.right_hi)?;
        Ok(Self {
            dir: input.dir.try_into()?,
            hi: input.hi,
            lo: input.lo,
            left_lo: left_lo,
            left_hi: left_hi,
            right_lo: right_lo,
            right_hi: right_hi,
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
fn parse_c_x_y(s: &str) -> Result<Vec2, BevySC2MapError> {
    let (tail, _) = take_until("c=(")(s)?;
    let (tail, _) = tag("c=(")(tail)?;
    let (tail, x) = take_until(", ")(tail)?;
    let (tail, _) = tag(", ")(tail)?;
    let (_, y) = take_until(") ")(tail)?;
    Ok(c_x_y_to_vec2(x, y)?)
}

fn c_x_y_to_vec2(x_str: &str, y_str: &str) -> Result<Vec2, BevySC2MapError> {
    Ok(Vec2::new(x_str.parse()?, y_str.parse()?))
}

/// Loads the t3 terrain with ramps
pub fn load_t3_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    t3_height_map_res: Res<T3HeightMapResource>,
    t3_terrain: ResMut<T3TerrainResource>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (ramp_ith, ramp) in t3_terrain.ramp_list.iter().enumerate() {
        let left_lo_x = ramp.left_lo.x;
        let left_lo_y = ramp.left_lo.y;
        let right_lo_x = ramp.right_lo.x;
        let right_lo_y = ramp.right_lo.y;

        // Find the height for the ramp points.
        // I guess these two should be the same since they are "lo"?
        let left_lo_target_vec_pos =
            left_lo_y as usize * t3_height_map_res.width as usize + left_lo_x as usize;
        let right_lo_target_vec_pos =
            left_lo_y as usize * t3_height_map_res.width as usize + left_lo_x as usize;

        let left_lo_cell_height = t3_height_map_res.data[left_lo_target_vec_pos];
        let right_lo_cell_height = t3_height_map_res.data[right_lo_target_vec_pos];
        let left_lo_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * left_lo_y,
            (left_lo_cell_height as f32 - 0.35) * CELL_HEIGHT_MULTIPLIER * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * left_lo_x,
        );
        let right_lo_transform = Transform::from_xyz(
            MAP_SCALE_FACTOR * right_lo_y,
            (right_lo_cell_height as f32 - 0.35) * CELL_HEIGHT_MULTIPLIER * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR * right_lo_x,
        );
        let ramp_mesh = Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(
            MAP_SCALE_FACTOR,
            RAMP_HEIGHT * MAP_SCALE_FACTOR,
            MAP_SCALE_FACTOR,
        ))));
        commands
            .spawn((
                DisplayInfoOnClick,
                Name(format!("left_lo: {}", ramp_ith).into()),
                left_lo_transform,
                ramp_mesh.clone(),
                MeshMaterial3d(materials.add(Color::from(palettes::tailwind::RED_600))),
            ))
            .observe(update_info_on_click);
        commands
            .spawn((
                DisplayInfoOnClick,
                Name(format!("left_lo: {}", ramp_ith).into()),
                right_lo_transform,
                ramp_mesh,
                MeshMaterial3d(materials.add(Color::from(palettes::tailwind::RED_600))),
            ))
            .observe(update_info_on_click);
    }
}
