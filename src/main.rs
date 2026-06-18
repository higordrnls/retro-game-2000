use bevy::prelude::*;
use bevy_rapier2d::prelude::*; 

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        // Configuração da janela mobile + pixel art nítida
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(360.0, 640.0),
                title: "Meu RPG Y2K".into(),
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        
        // Ativando os plugins de física do Rapier
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default()) // Desenha linhas para vermos as colisões
        
        // Sistemas do jogo
        .add_systems(Startup, setup_game)
        .add_systems(Update, mover_jogador)
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 1. Câmera
    commands.spawn(Camera2dBundle::default());

    // 2. CHÃO
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
        RigidBody::Fixed,
        Collider::cuboid(300.0, 25.0),
    ));

    // 3. JOGADOR (O teu dadinho)
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("meu_personagem.png"),
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..default()
        },
        Player,
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::cuboid(25.0, 25.0),
        LockedAxes::ROTATION_LOCKED,
        // AJUSTADO: Aumentámos o peso para 20.0 para ele cair muito mais rápido!
        GravityScale(20.0), 
    ));
}

fn mover_jogador(
    teclas: Res<ButtonInput<KeyCode>>, 
    mut query: Query<&mut Velocity, With<Player>>
) {
    if let Ok(mut vel) = query.get_single_mut() {
        let velocidad_corrida = 250.0;
        
        // Controles de direção (WASD + Setas)
        if teclas.pressed(KeyCode::KeyA) || teclas.pressed(KeyCode::ArrowLeft) { 
            vel.linvel.x = -velocidad_corrida; 
        } else if teclas.pressed(KeyCode::KeyD) || teclas.pressed(KeyCode::ArrowRight) { 
            vel.linvel.x = velocidad_corrida; 
        } else { 
            vel.linvel.x = 0.0;
        }

        // AJUSTADO: Diminuímos a força vertical de 350.0 para apenas 130.0!
        if teclas.just_pressed(KeyCode::Space) || teclas.just_pressed(KeyCode::KeyW) || teclas.just_pressed(KeyCode::ArrowUp) {
            vel.linvel.y = 130.0; 
        }
    }
}