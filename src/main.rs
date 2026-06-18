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
        .add_systems(Update, mover_jogador)
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // CHÃO
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.2, 0.2), 
                custom_size: Some(Vec2::new(600.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(300.0, 25.0),
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
    ));

    // JOGADOR (O seu dadinho)
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
        GravityScale(75.0), 
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Ccd::enabled(), 
    ));
}

fn mover_jogador(
    teclas: Res<ButtonInput<KeyCode>>, 
    mut query: Query<(&mut Velocity, &Transform, &mut Player)>
) {
    if let Ok((mut vel, transform, mut player)) = query.get_single_mut() {
        
        if vel.linvel.y.abs() < 0.1 && transform.translation.y <= -140.0 {
            player.saltos = 0;
        }

        let velocidad_corrida = 250.0;
        
        if teclas.pressed(KeyCode::KeyA) || teclas.pressed(KeyCode::ArrowLeft) { 
            vel.linvel.x = -velocidad_corrida; 
        } else if teclas.pressed(KeyCode::KeyD) || teclas.pressed(KeyCode::ArrowRight) { 
            vel.linvel.x = velocidad_corrida; 
        } else { 
            vel.linvel.x = 0.0;
        }

        if teclas.just_pressed(KeyCode::Space) || teclas.just_pressed(KeyCode::KeyW) || teclas.just_pressed(KeyCode::ArrowUp) {
            if player.saltos < 2 {
                // MODIFICADO: De 160.0 para 226.0 para atingir o dobro da altura física
                vel.linvel.y = 226.0; 
                player.saltos += 1;   
            }
        }
    }
}