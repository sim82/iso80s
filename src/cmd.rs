use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    input::CursorMarker,
    iso::{IsoCoord, IsoState},
};

#[derive(Default)]
pub struct CommandState {
    undo_stack: Vec<Command>,
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Single { coord: IsoCoord, tile_type: usize },
    Despawn(IsoCoord),
    Undo,
}

fn apply_commands_system(
    mut commands: Commands,
    iso_state: Res<IsoState>,
    mut command_state: ResMut<CommandState>,
    mut command_events: EventReader<(bool, Command)>,
    mut query: Query<(Entity, &IsoCoord, &mut TextureAtlasSprite), Without<CursorMarker>>,
) {
    let mut command_queue: VecDeque<_> = command_events.iter().cloned().collect();

    while let Some((user_generated, command)) = command_queue.pop_back() {
        match command {
            Command::Single { coord, tile_type } => {
                let mut found = false;
                for (_, query_coord, mut sprite) in query.iter_mut() {
                    if *query_coord == coord {
                        info!("pick: {:?} {:?}", query_coord, sprite);
                        if user_generated {
                            command_state.undo_stack.push(Command::Single {
                                coord,
                                tile_type: sprite.index,
                            });
                        }
                        sprite.index = tile_type;
                        found = true;
                        break;
                    }
                }
                if !found {
                    info!("spawn");
                    commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: iso_state.tileset_atlas.clone(),
                            sprite: TextureAtlasSprite {
                                index: tile_type,
                                ..default()
                            },
                            // transform: Transform::from_translation(iso_coord.into()),
                            ..default()
                        })
                        .insert(coord);

                    if user_generated {
                        command_state.undo_stack.push(Command::Despawn(coord));
                    }
                }
            }
            Command::Despawn(coord) => {
                for (entity, query_coord, _) in query.iter() {
                    if *query_coord == coord {
                        info!("despawn {:?}", query_coord);
                        commands.entity(entity).despawn();
                        break;
                    }
                }
            }
            Command::Undo => {
                if let Some(undo_command) = command_state.undo_stack.pop() {
                    command_queue.push_back((false, undo_command));
                }
            }
        }
    }
    info!("undo stack: {:?}", command_state.undo_stack);
}

pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandState>()
            .add_event::<(bool, Command)>()
            .add_system(apply_commands_system);
    }
}
