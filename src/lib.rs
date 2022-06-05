use bevy::prelude::PluginGroup;

pub mod camera;
pub mod cmd;
pub mod input;
pub mod iso;

pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(camera::CameraPlugin)
            .add(cmd::CommandPlugin)
            .add(iso::IsoPlugin)
            .add(input::InputPlugin);
    }
}
