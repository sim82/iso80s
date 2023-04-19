use std::collections::VecDeque;

use super::prelude::*;
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

#[derive(Default, Resource)]
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
        if let Command::Set { coords, tile_type } = command {
            let layer = coords.first().map_or(1, |c| c.1 as i32);
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
