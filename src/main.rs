use bevy::prelude::*;
use bevy_rapier2d::prelude::*; // <--- ISSO É O QUE FALTA!

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin2d::default()) // <--- ATIVA A FÍSICA
        .add_plugins(RapierDebugRenderPlugin::default()) // Opcional: mostra as caixas de colisão
        .add_systems(Startup, setup_game)
        .add_systems(Update, mover_jogador)
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