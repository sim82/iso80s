use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    math::{Vec2Swizzles, Vec3Swizzles},
    prelude::*,
};
use bevy_egui::{
    egui::{self, TextureId},
    EguiContext,
};
use bevy_mouse_tracking_plugin::{MousePosPlugin, MousePosWorld};

use crate::{
    cmd,
    iso::{self, IsoCoord, IsoState, PIXEL_TO_ISO},
};

#[derive(Component)]
pub struct CursorMarker;

#[derive(Default)]
pub struct InputState {
    pub tile_type: usize,
    pub layer: i32,

    texture: Option<egui::TextureId>,
}

#[allow(clippy::too_many_arguments)]
fn iso_pick_system(
    mouse: Res<MousePosWorld>,
    state: Res<InputState>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_query: Query<&mut IsoCoord, With<CursorMarker>>,
    mut command_events: EventWriter<(bool, cmd::Command)>,
) {
    let layer = state.layer as f32;
    let pick_coord = (*PIXEL_TO_ISO * (mouse.xy() - layer * 16.0 * Vec2::Y)).floor();
    if let Ok(mut iso_coord) = cursor_query.get_single_mut() {
        iso_coord.0 = pick_coord;
        iso_coord.1 = layer;
    }
    for event in mouse_button_input_events.iter() {
        if event.state == ElementState::Released && event.button == MouseButton::Left {
            let coord = IsoCoord(pick_coord, layer);
            command_events.send((
                true,
                cmd::Command::Single {
                    coord,
                    tile_type: state.tile_type,
                },
            ));
        }
    }
}

fn key_input_system(
    mut command_events: EventWriter<(bool, cmd::Command)>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Z) && input.pressed(KeyCode::LControl) {
        command_events.send((true, cmd::Command::Undo));
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
            .add_system(input_egui_system)
            .add_system(key_input_system);
    }
}
