use bevy::prelude::*;
use bevy_rapier2d::prelude::*; 

#[derive(Component)]
struct Player {
    saltos: u32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(360.0, 640.0),
                title: "Meu RPG Y2K".into(),
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default()) 
        
        .add_systems(Startup, setup_game)
        .add_systems(Update, (mover_jogador, camera_seguidora)) // ADICIONADO: camera_seguidora rodando a cada frame
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // MODIFICADO: Adicionamos um componente na câmera para o sistema conseguir achar ela mais fácil
    commands.spawn((
        Camera2dBundle::default(),
        SpatialBundle::default(), // Ajuda o Bevy a rastrear o posicionamento
    ));

    // CHÃO MUITO MAIS LARGO (Aumentamos de 600 para 3000 de largura para você poder correr!)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.2, 0.2), 
                custom_size: Some(Vec2::new(3000.0, 50.0)), // Chão gigante para os lados
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(1500.0, 25.0), // Ajustado o colisor para metade da largura (regrade do Rapier)
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
    ));

    // JOGADOR
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
        GravityScale(150.0), 
        Damping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Ccd::enabled(), 
    ));
}

fn mover_jogador(
    teclas: Res<ButtonInput<KeyCode>>, 
    mut query: Query<(&mut Velocity, &Transform, &mut Player, &mut Sprite)>
) {
    if let Ok((mut vel, transform, mut player, mut sprite)) = query.get_single_mut() {
        
        if vel.linvel.y.abs() < 0.1 && transform.translation.y <= -140.0 {
            player.saltos = 0;
        }

        let velocidad_corrida = 250.0;
        
        if teclas.pressed(KeyCode::KeyA) || teclas.pressed(KeyCode::ArrowLeft) { 
            vel.linvel.x = -velocidad_corrida; 
            sprite.flip_x = true; 
        } else if teclas.pressed(KeyCode::KeyD) || teclas.pressed(KeyCode::ArrowRight) { 
            vel.linvel.x = velocidad_corrida; 
            sprite.flip_x = false; 
        } else { 
            vel.linvel.x = 0.0;
        }

        if teclas.just_pressed(KeyCode::Space) || teclas.just_pressed(KeyCode::KeyW) || teclas.just_pressed(KeyCode::ArrowUp) {
            if player.saltos < 2 {
                vel.linvel.y = 320.0; 
                player.saltos += 1;   
            }
        }
    }
}

// NOVO SISTEMA: Faz a câmera seguir o jogador de forma fofa e suave
fn camera_seguidora(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            
            // Onde a câmera quer chegar (na mesma posição X e Y do jogador)
            let alvo_x = player_transform.translation.x;
            let alvo_y = player_transform.translation.y + 100.0; // +100 para a câmera ficar um tiquinho mais alta e dar melhor visão do céu

            // Velocidade da suavização (quanto menor o número, mais "desliza")
            let velocidade_da_camera = 4.0; 

            // Interpolação Linear (Lerp) manual para mover a câmera só um pedacinho na direção do jogador a cada frame
            camera_transform.translation.x += (alvo_x - camera_transform.translation.x) * velocidade_da_camera * time.delta_seconds();
            camera_transform.translation.y += (alvo_y - camera_transform.translation.y) * velocidade_da_camera * time.delta_seconds();
        }
    }
}