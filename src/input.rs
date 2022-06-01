use bevy::{
    input::mouse::MouseButtonInput,
    math::{Vec2Swizzles, Vec3Swizzles},
    prelude::*,
};
use bevy_egui::{egui, EguiContext};
use bevy_mouse_tracking_plugin::{MousePosPlugin, MousePosWorld};

use crate::iso::{self, IsoCoord, PIXEL_TO_ISO};

#[derive(Default)]
pub struct InputState {
    tile_type: usize,
    layer: i32,
}

fn iso_pick_system(
    mouse: Res<MousePosWorld>,
    state: Res<InputState>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut query: Query<(&IsoCoord, &mut TextureAtlasSprite)>,
) {
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left {
            let offset = 16 * state.layer as usize;
            let layer = state.layer as f32;
            let pick_coord = (*PIXEL_TO_ISO * (mouse.xy() - layer * 16.0 * Vec2::Y)).floor();
            for (iso_coord, mut sprite) in query.iter_mut() {
                if iso_coord.0.x == pick_coord.x
                    && iso_coord.0.y == pick_coord.y
                    && iso_coord.1 == layer
                {
                    sprite.index = state.tile_type + offset;
                    break;
                }
            }
        }
    }
}

fn input_egui_system(mut state: ResMut<InputState>, mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("input").show(egui_context.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut state.tile_type, 0..=8));
        ui.add(egui::Slider::new(&mut state.layer, 0..=3));
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
