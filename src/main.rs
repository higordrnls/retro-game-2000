use bevy::{prelude::*, window::WindowResolution};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Aqui definimos o tamanho (ex: 360x640 para uma proporção vertical)
                resolution: WindowResolution::new(360.0, 640.0),
                title: "Meu RPG Y2K".into(),
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
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

// 1. Definimos o componente do jogador
#[derive(Component)]
struct Player;

// 2. O Sistema que lê o teclado e move o Player
fn mover_jogador(
    teclas: Res<ButtonInput<KeyCode>>, 
    mut query: Query<&mut Transform, With<Player>>
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let velocidade = 2.0;
        if teclas.pressed(KeyCode::ArrowLeft) { transform.translation.x -= velocidade; }
        if teclas.pressed(KeyCode::ArrowRight) { transform.translation.x += velocidade; }
        // ... repita para cima/baixo
    }
}