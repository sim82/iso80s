use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec3Swizzles,
    prelude::*,
};
use bevy_egui::{
    egui::{self},
    EguiContext,
};
use bevy_mouse_tracking_plugin::{MousePosPlugin, MousePosWorld};

use crate::{
    cmd,
    iso::{IsoCoord, IsoState, PIXEL_TO_ISO},
};

#[derive(Component)]
pub struct CursorMarker;

#[derive(Default)]
pub struct InputState {
    pub tile_type: usize,
    pub layer: i32,

    texture: Option<egui::TextureId>,

    last_pos: IsoCoord,
}

#[allow(clippy::too_many_arguments)]
fn iso_pick_system(
    mouse: Res<MousePosWorld>,
    mut state: ResMut<InputState>,
    key_input: Res<Input<KeyCode>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_query: Query<(&mut IsoCoord, &mut TextureAtlasSprite), With<CursorMarker>>,
    mut command_events: EventWriter<cmd::Command>,
) {
    let layer = state.layer as f32;
    let pick_coord = (*PIXEL_TO_ISO * (mouse.xy() - layer * 16.0 * Vec2::Y)).floor();

    if pick_coord.x < 0.0 || pick_coord.y < 0.0 {
        return;
    }

    let coord = IsoCoord(pick_coord, layer);

    let line_mode = if key_input.pressed(KeyCode::LShift) {
        let d = coord.0 - state.last_pos.0;
        let dir = d.clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
        let len = d.length() as usize;
        info!("dir: {:?}", dir);
        if dir.x == 0.0 || dir.y == 0.0 {
            Some((dir, len))
        } else {
            None
        }
    } else {
        None
    };

    let cursor_index = if line_mode.is_some() { 33 } else { 32 };

    if let Ok((mut iso_coord, mut sprite)) = cursor_query.get_single_mut() {
        iso_coord.0 = pick_coord;
        iso_coord.1 = layer;
        sprite.index = cursor_index;
    }

    for event in mouse_button_input_events.iter() {
        if event.state == ElementState::Released && event.button == MouseButton::Left {
            if !key_input.pressed(KeyCode::LShift) {
                command_events.send(cmd::Command::Set {
                    coords: vec![coord],
                    tile_type: state.tile_type,
                });
            } else if let Some((dir, len)) = line_mode {
                let mut brush = state.last_pos.0;

                let coords = (0..len)
                    .map(|_| {
                        brush += dir;
                        IsoCoord(brush, layer)
                    })
                    .collect();
                command_events.send(cmd::Command::Set {
                    coords,
                    tile_type: state.tile_type,
                });
            }

            state.last_pos = coord;
        }
    }
}

fn key_input_system(mut command_events: EventWriter<cmd::Command>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Z) && input.pressed(KeyCode::LControl) {
        command_events.send(cmd::Command::Undo);
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
