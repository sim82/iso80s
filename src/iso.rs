use bevy::{math::Mat2, prelude::*};
use lazy_static::lazy_static;

const SIZE: f32 = 32.0;

lazy_static! {
    pub static ref ISO_TO_PIXEL: Mat2 = Mat2::from_cols(
        Vec2::new(0.5 * SIZE, 0.25 * SIZE),
        Vec2::new(-0.5 * SIZE, 0.25 * SIZE),
    );
    pub static ref PIXEL_TO_ISO: Mat2 = ISO_TO_PIXEL.inverse();
}

#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct IsoCoord(pub Vec2, pub f32);

impl From<&IsoCoord> for Vec3 {
    fn from(IsoCoord(v, layer): &IsoCoord) -> Self {
        // the z value calculation is a bit ad-hoc and only works for v.x + v.y <= 32.0.
        // but generally it works to a proper z order for rendering the tiles correctly
        (*ISO_TO_PIXEL * *v + layer * 16.0 * Vec2::Y).extend(32.0 - v.x - v.y + layer)
    }
}

fn iso_coord_update(mut query: Query<(&mut Transform, &IsoCoord), Changed<IsoCoord>>) {
    for (mut transform, iso_coord) in query.iter_mut() {
        transform.translation = iso_coord.into();
    }
}

#[derive(Default)]
pub struct IsoState {
    pub tileset_image: Handle<Image>,
    pub tileset_atlas: Handle<TextureAtlas>,
}

pub fn iso_startup_system(
    mut iso_state: ResMut<IsoState>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("iso_tiles_rgb.png");
    let atlas = TextureAtlas::from_grid(texture_handle.clone(), Vec2::splat(32.0), 8, 8);
    let atlas_handle = texture_atlases.add(atlas);
    iso_state.tileset_atlas = atlas_handle;
    iso_state.tileset_image = texture_handle;
}

pub struct IsoPlugin;
impl Plugin for IsoPlugin {
    fn build(&self, app: &mut App) {
        info!("init plugin");
        app.register_type::<IsoCoord>()
            .init_resource::<IsoState>()
            .add_startup_system(iso_startup_system)
            .add_system(iso_coord_update);
    }
}
