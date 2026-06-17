use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // <-- O SEGREDO DO PIXEL ART NÍTIDO!
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(mut commands: Commands) {
    // 1. Spawn da Câmera
    commands.spawn(Camera2dBundle::default());

    // 2. Spawn de um quadrado colorido (Sem precisar de imagem por enquanto)
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.0, 0.6, 0.9), // Azulzinho estilo Y2K
            custom_size: Some(Vec2::new(100.0, 100.0)), // Tamanho do quadrado
            ..default()
        },
        ..default()
    });
}