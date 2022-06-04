use bevy::{
    input::mouse::MouseButtonInput,
    math::{Vec2Swizzles, Vec3Swizzles},
    prelude::*,
};
use bevy_egui::{
    egui::{self, TextureId},
    EguiContext,
};
use bevy_mouse_tracking_plugin::{MousePosPlugin, MousePosWorld};

use crate::iso::{self, IsoCoord, IsoState, PIXEL_TO_ISO};

#[derive(Component)]
pub struct CursorMarker;

#[derive(Default)]
pub struct InputState {
    pub tile_type: usize,
    pub layer: i32,

    texture: Option<egui::TextureId>,
}

fn iso_pick_system(
    mut commands: Commands,
    mouse: Res<MousePosWorld>,
    state: Res<InputState>,
    iso_state: Res<IsoState>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut query: Query<(&IsoCoord, &mut TextureAtlasSprite), Without<CursorMarker>>,
    mut cursor_query: Query<&mut IsoCoord, With<CursorMarker>>,
) {
    let layer = state.layer as f32;
    let pick_coord = (*PIXEL_TO_ISO * (mouse.xy() - layer * 16.0 * Vec2::Y)).floor();
    if let Ok(mut iso_coord) = cursor_query.get_single_mut() {
        iso_coord.0 = pick_coord;
        iso_coord.1 = layer;
    }
    'outer: for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left {
            let offset = 0; // 16 * state.layer as usize;

            for (iso_coord, mut sprite) in query.iter_mut() {
                if iso_coord.0.x == pick_coord.x
                    && iso_coord.0.y == pick_coord.y
                    && iso_coord.1 == layer
                {
                    info!("pick: {:?}", iso_coord);
                    sprite.index = state.tile_type + offset;
                    break 'outer;
                }
            }
            info!("spwan");
            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: iso_state.tileset_atlas.clone(),
                    sprite: TextureAtlasSprite {
                        index: state.tile_type,
                        ..default()
                    },
                    // transform: Transform::from_translation(iso_coord.into()),
                    ..default()
                })
                .insert(IsoCoord(pick_coord, layer));
        }
    }
}

fn input_egui_system(
    mut state: ResMut<InputState>,
    iso_state: Res<IsoState>,
    mut egui_context: ResMut<EguiContext>,
) {
    let texture = *state
        .texture
        .get_or_insert_with(|| egui_context.add_image(iso_state.tileset_image.clone()));

    egui::Window::new("input").show(egui_context.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut state.tile_type, 0..=24));
        ui.add(egui::Slider::new(&mut state.layer, 0..=3));

        let response = ui.add(egui::ImageButton::new(texture, (256.0, 256.0)));

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let pos = pos - response.rect.left_top();
                info!("pos: {:?}", pos);
                state.tile_type = (pos.y as usize / 32) * 8 + (pos.x as usize / 32);
            }
        }
        // if let Some(hover_pos) = response.hover_pos() {
        //     let pos = hover_pos - response.rect.left_top();
        //     info!("pos: {:?}", pos);

        //     if response.interact_pointer_pos() {
        //         info!("pos: {:?}", pos);
        //     }
        // }
    });
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_plugin(MousePosPlugin::SingleCamera)
            .add_system(iso_pick_system)
            .add_system(input_egui_system);
    }
}
