use bevy::prelude::*;

fn camera_setup_system(mut commands: Commands) {
    // commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn(Camera2dBundle::default());
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(camera_setup_system);
    }
}
