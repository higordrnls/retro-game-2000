use bevy::{prelude::*, window::WindowResolution};

// 1. Define o componente que marca quem é o jogador
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(360.0, 640.0), // Formato Widescreen Mobile
                title: "Meu RPG Y2K".into(),
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_game)
        .add_systems(Update, mover_jogador) // <--- IMPORTANTE: Registramos o movimento aqui!
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("meu_personagem.png"),
            // Transforma o tamanho: 0.5 é 50% do original
            transform: Transform::from_scale(Vec3::splat(0.5)), 
            ..default()
        },
        Player, // <-- O personagem agora tem a "tag" Player
    ));
}

fn mover_jogador(
    teclas: Res<ButtonInput<KeyCode>>, 
    mut query: Query<&mut Velocity, With<Player>>
) {
    if let Ok(mut vel) = query.get_single_mut() {
        let velocidade = 300.0;
        
        // Movimento Horizontal (WASD e Setas)
        if teclas.pressed(KeyCode::KeyA) || teclas.pressed(KeyCode::ArrowLeft) { vel.linvel.x = -velocidade; }
        else if teclas.pressed(KeyCode::KeyD) || teclas.pressed(KeyCode::ArrowRight) { vel.linvel.x = velocidade; }
        else { vel.linvel.x = 0.0; }

        // Pulo (Espaço)
        if teclas.just_pressed(KeyCode::Space) {
            vel.linvel.y = 500.0; // Aplica um impulso para cima
        }
    }
}