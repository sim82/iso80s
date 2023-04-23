use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

pub mod camera;
pub mod cmd;
pub mod input;
pub mod iso;

pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(camera::CameraPlugin)
            .add(cmd::CommandPlugin)
            .add(cmd::predict::PredictPlugin)
            .add(iso::IsoPlugin)
            .add(input::InputPlugin)
    }
}
