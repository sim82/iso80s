use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    input::CursorMarker,
    iso::{IsoCoord, IsoState},
};

pub mod prelude {
    pub use super::Command;
}

pub mod predict;

#[derive(Default, Resource)]
pub struct CommandState {
    undo_stack: Vec<(usize, Command)>,
    transaction: usize,
}

#[derive(Clone, Debug)]
pub enum Command {
    Set {
        coords: Vec<IsoCoord>,
        tile_type: usize,
    },
    Despawn(IsoCoord),
    Undo,
}

fn apply_commands_system(
    mut commands: Commands,
    iso_state: Res<IsoState>,
    mut command_state: ResMut<CommandState>,
    mut command_events: EventReader<Command>,
    mut query: Query<(Entity, &IsoCoord, &mut TextureAtlasSprite), Without<CursorMarker>>,
) {
    let mut command_queue: VecDeque<_> = command_events
        .iter()
        .map(|command| (true, command.clone()))
        .collect();

    while let Some((user_generated, command)) = command_queue.pop_back() {
        match command {
            Command::Set { coords, tile_type } => {
                let transaction = command_state.transaction;

                for coord in coords {
                    let mut found = false;
                    for (_, query_coord, mut sprite) in query.iter_mut() {
                        if *query_coord == coord {
                            info!("pick: {:?} {:?}", query_coord, sprite);
                            if user_generated {
                                command_state.undo_stack.push((
                                    transaction,
                                    Command::Set {
                                        coords: vec![coord],
                                        tile_type: sprite.index,
                                    },
                                ));
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
                            command_state
                                .undo_stack
                                .push((transaction, Command::Despawn(coord)));
                        }
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
                let mut last_tx = None;
                while let Some((tx, undo_command)) = command_state.undo_stack.last() {
                    if last_tx.is_some() && last_tx != Some(*tx) {
                        break;
                    }
                    command_queue.push_back((false, undo_command.clone()));
                    last_tx = Some(*tx);
                    command_state.undo_stack.pop();
                }
            }
        }
    }
    command_state.transaction += 1;
    // info!("undo stack: {:?}", command_state.undo_stack);
}

pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandState>()
            .add_event::<Command>()
            .add_system(apply_commands_system);
    }
}
