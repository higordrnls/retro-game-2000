use bevy::prelude::*;
use bevy_rapier2d::prelude::*; 

#[derive(Component)]
struct Player {
    saltos: u32,
}

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
        Player { saltos: 0 }, 
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::cuboid(25.0, 25.0),
        LockedAxes::ROTATION_LOCKED,
        // MODIFICADO: Dobramos o peso! De 20.0 foi para 40.0 (Cai muito rápido)
        GravityScale(40.0), 
    ));
}

fn mover_jogador(
    teclas: Res<ButtonInput<KeyCode>>, 
    mut query: Query<(&mut Velocity, &Transform, &mut Player)>
) {
    if let Ok((mut vel, transform, mut player)) = query.get_single_mut() {
        
        // Sistema de reset dos pulos ao tocar no chão
        if transform.translation.y <= -148.0 {
            player.saltos = 0;
        }

        let velocidad_corrida = 250.0;
        
        // Controles de direção (WASD + Setas)
        if teclas.pressed(KeyCode::KeyA) || teclas.pressed(KeyCode::ArrowLeft) { 
            vel.linvel.x = -velocidad_corrida; 
        } else if teclas.pressed(KeyCode::KeyD) || teclas.pressed(KeyCode::ArrowRight) { 
            vel.linvel.x = velocidad_corrida; 
        } else { 
            vel.linvel.x = 0.0;
        }

        // Pulo duplo com velocidade dobrada!
        if teclas.just_pressed(KeyCode::Space) || teclas.just_pressed(KeyCode::KeyW) || teclas.just_pressed(KeyCode::ArrowUp) {
            if player.saltos < 2 {
                // MODIFICADO: Força dobrada! De 260.0 foi para 520.0 (Sobe rápido)
                vel.linvel.y = 520.0; 
                player.saltos += 1;   
            }
        }
    }
}