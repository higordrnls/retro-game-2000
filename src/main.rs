use bevy::prelude::*;
use bevy_rapier2d::prelude::*; 

// 1. AQUI ESTÁ A DEFINIÇÃO QUE FALTAVA!
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin2d::default()) // Ativa a física
        .add_plugins(RapierDebugRenderPlugin::default()) // Mostra os contornos verdes de colisão (ajuda muito!)
        .add_systems(Startup, setup_game)
        .add_systems(Update, mover_jogador)
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // CHÃO: Precisamos de um lugar para o personagem pisar
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.2, 0.2), // Cinza escuro
                custom_size: Some(Vec2::new(600.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        },
        RigidBody::Fixed, // "Fixed" significa que a gravidade não afeta o chão
        Collider::cuboid(300.0, 25.0), // A caixa de colisão do chão
    ));

    // JOGADOR
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("meu_personagem.png"),
            transform: Transform::from_scale(Vec3::splat(0.5)), 
            ..default()
        },
        Player, // A tag que identifica ele
        RigidBody::Dynamic, // Corpo dinâmico (sofre gravidade)
        Velocity::default(), // Permite que a gente mude a velocidade dele
        Collider::cuboid(20.0, 20.0), // A caixa de colisão (ajuste o 20.0 se ficar grande ou pequena pro seu desenho)
        LockedAxes::ROTATION_LOCKED, // Impede que o personagem saia rolando igual uma bola
        GravityScale(3.0), // Deixa a gravidade um pouco mais rápida e responsiva
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

        // Pulo (Espaço, W ou Seta pra Cima)
        if teclas.just_pressed(KeyCode::Space) || teclas.just_pressed(KeyCode::KeyW) || teclas.just_pressed(KeyCode::ArrowUp) {
            vel.linvel.y = 500.0; // Aplica um impulso para cima
        }
    }
}