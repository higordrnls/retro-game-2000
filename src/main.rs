use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // <-- O SEGREDO DO PIXEL ART NÍTIDO!
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 1. Spawn da Câmera
    commands.spawn(Camera2dBundle::default());

    // 2. Spawn do SEU personagem
    commands.spawn(SpriteBundle {
        texture: asset_server.load("meu_personagem.png"), // Certifique-se de que o nome é idêntico!
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}