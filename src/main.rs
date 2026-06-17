use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // <-- O SEGREDO DO PIXEL ART NÍTIDO!
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 1. Spawn da Câmera (Obrigatório para ver algo)
    commands.spawn(Camera2dBundle::default());

    // 2. Spawn do Personagem (O herói/RPG)
    commands.spawn(SpriteBundle {
        // Aqui o Bevy vai buscar na sua pasta /assets/
        texture: asset_server.load("meu_personagem.png"), 
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}