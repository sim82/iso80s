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
    for y in 0..4 {
        for x in 0..4 {
            let x = x as f32;
            let y = y as f32;
            let iso_coord = IsoCoord(Vec2::new(x, y));
            pos.push(iso_coord);
        }
    }
    pos.reverse();
    for iso_coord in pos {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: iso_state.tileset_atlas.clone(),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..default()
                },
                // transform: Transform::from_translation(iso_coord.into()),
                ..default()
            })
            .insert(iso_coord);
    }
}
