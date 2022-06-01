use bevy::{diagnostic::DiagnosticsPlugin, input::system::exit_on_esc_system, prelude::*};
use clap::Parser;
use iso80s::iso::{IsoCoord, IsoPlugin, IsoState};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CmdlineArgs {
    #[clap(short, long)]
    pub debug_draw: bool,

    #[clap(short, long)]
    pub world_inspector: bool,
}

fn main() {
    let args = CmdlineArgs::parse();

    let mut app = App::new();
    // bevy plugins
    app.add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_system(exit_on_esc_system)
        .insert_resource(Msaa::default())
        .add_plugins(iso80s::DefaultPlugins)
        .add_startup_system(setup_system.after(iso80s::iso::iso_startup_system));

    // egui plugins
    #[cfg(feature = "inspector")]
    {
        if args.world_inspector {
            app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
        }
    }

    app.run();
}

fn setup_system(mut commands: Commands, iso_state: Res<IsoState>) {
    // for i in 0..8 {
    //     commands.spawn_bundle(SpriteSheetBundle {
    //         texture_atlas: iso_state.tileset_atlas.clone(),
    //         sprite: TextureAtlasSprite {
    //             index: i,
    //             ..default()
    //         },
    //         transform: Transform::from_translation(Vec3::new(i as f32 * 32.0, 0.0, 0.0)),
    //         ..default()
    //     });
    // }

    let mut pos = Vec::new();
    for layer in 0..2 {
        let mut pos_tmp = Vec::new();

        for y in 0..16 {
            for x in 0..16 {
                let layer = layer as f32;
                let x = x as f32;
                let y = y as f32;
                let iso_coord = IsoCoord(Vec2::new(x, y), layer);
                pos_tmp.push(iso_coord);
            }
        }
        pos_tmp.reverse();
        pos.append(&mut pos_tmp);
    }
    for iso_coord in pos {
        let index = if iso_coord.1 == 0.0 { 0 } else { 8 };
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: iso_state.tileset_atlas.clone(),
                sprite: TextureAtlasSprite { index, ..default() },
                // transform: Transform::from_translation(iso_coord.into()),
                ..default()
            })
            .insert(iso_coord);
    }
}
