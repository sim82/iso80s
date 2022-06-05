use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    input::CursorMarker,
    iso::{IsoCoord, IsoState},
};

pub mod prelude {
    pub use super::Command;
}

pub mod predict {
    use std::collections::VecDeque;

    use super::prelude::*;
    use crate::iso::prelude::*;
    use bevy::{prelude::*, utils::HashMap};
    use itertools::Itertools;

    // basic idea: train Markov'esque model on the tile_type and layer of each added tile to predict the adjustments
    // the designer will likely do next. (i.e. add a ramp in higher layer after lauying down a path in layer 0).
    //

    // TODO: there must be some common name for the type of data a HHM is trained on
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
    struct ModelBasekey {
        layer_change: i32,
        tile_type: usize,
    }
    const DEPTH: usize = 2;
    type ModelKey = (ModelBasekey, ModelBasekey);
    type ModelKey2 = (ModelBasekey, ModelBasekey, ModelBasekey);

    #[derive(Default)]
    pub struct PredictState {
        history: VecDeque<ModelBasekey>,
        model: HashMap<ModelKey, HashMap<ModelBasekey, usize>>,
        last_layer: i32,
    }

    fn training_system(
        mut predict_state: ResMut<PredictState>,
        mut command_events: EventReader<Command>,
    ) {
        for command in command_events.iter() {
            if let Command::Single {
                coord: IsoCoord(_, layer),
                tile_type,
            } = command
            {
                let layer = *layer as i32;
                let layer_change = layer - predict_state.last_layer;
                predict_state.history.push_back(ModelBasekey {
                    layer_change,
                    tile_type: *tile_type,
                });
                predict_state.last_layer = layer;
            }
        }
        if predict_state.history.len() > DEPTH {
            let old_history = predict_state.history.clone();

            for key2 in old_history.iter().cloned().tuple_windows::<ModelKey2>() {
                let key = (key2.0, key2.1);
                match predict_state.model.entry(key) {
                    bevy::utils::hashbrown::hash_map::Entry::Occupied(mut e) => {
                        match e.get_mut().entry(key2.2) {
                            bevy::utils::hashbrown::hash_map::Entry::Occupied(mut e) => {
                                *e.get_mut() += 1;
                            }
                            bevy::utils::hashbrown::hash_map::Entry::Vacant(e) => {
                                e.insert(1);
                            }
                        }
                    }
                    bevy::utils::hashbrown::hash_map::Entry::Vacant(e) => {
                        let mut m = HashMap::new();
                        m.insert(key2.2, 1);
                        e.insert(m);
                    }
                }
                info!("train: {:?}", key2);
                info!("model: {:?}", predict_state.model);
            }
        }
        while predict_state.history.len() > DEPTH {
            predict_state.history.pop_front();
        }

        // if !command_events.is_empty() {
        for key in predict_state
            .history
            .iter()
            .cloned()
            .tuple_windows::<ModelKey>()
        {
            if let Some(model_map) = predict_state.model.get(&key) {
                let mut prediction = model_map
                    .iter()
                    .sorted_by_key(|(_, count)| **count)
                    .collect::<Vec<_>>();

                prediction.reverse();
                // info!("prediction: {:?} -> {:?}", key, prediction);
            }
        }
        // }
    }

    pub struct PredictPlugin;

    impl Plugin for PredictPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<PredictState>()
                .add_system(training_system);
        }
    }
}

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
    mut command_events: EventReader<Command>,
    mut query: Query<(Entity, &IsoCoord, &mut TextureAtlasSprite), Without<CursorMarker>>,
) {
    let mut command_queue: VecDeque<_> = command_events
        .iter()
        .map(|command| (true, *command))
        .collect();

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
