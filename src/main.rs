use bevy::prelude::*;
use bevy_rapier2d::prelude::*; 

#[derive(Component)]
struct Player {
    saltos: u32,
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn main() {
    App::new()
        // Adiciona as configurações padrão do Bevy (janela, gráficos, etc)
        .add_plugins(DefaultPlugins) 
        
        // Se você estiver usando física (Rapier), o plugin dele fica aqui:
        // .add_plugins(RapierPhysicsPlugin::<NoUserData>::default()) 
        
        // O setup roda uma única vez no início (para criar o boneco e o chão)
        .add_systems(Startup, setup_game)
        
        // --- CORREÇÃO AQUI ---
        // Agora os nomes batem exatamente com as suas funções criadas abaixo!
        // Também adicionei a camera_seguidora para ela funcionar no jogo.
        .add_systems(Update, (mover_jogador, camera_seguidora, animate_player)) 
        
        // Roda o jogo de fato
        .run();
}

fn setup_game(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, 
) {
    commands.spawn(Camera2dBundle::default());

    // CHÃO GIGANTE
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.2, 0.2), 
                custom_size: Some(Vec2::new(3000.0, 50.0)), 
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(1500.0, 25.0),
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
    ));

    // Configuração da grade do seu spritesheet PNG
    let layout = TextureAtlasLayout::from_grid(UVec2::new(313, 313), 4, 4, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    // JOGADOR COM SPRITESHEET
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("meu_personagem_spritesheet.png"), 
            transform: Transform::from_scale(Vec3::splat(0.3)), 
            ..default()
        },
        TextureAtlas {
            layout: layout_handle,
            index: 0,
        },
        Player { saltos: 0 }, 
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), 
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::cuboid(40.0, 40.0), 
        LockedAxes::ROTATION_LOCKED,
        GravityScale(150.0), 
        Damping { linear_damping: 0.0, angular_damping: 0.0 },
        Restitution { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
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

fn camera_seguidora(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            let alvo_x = player_transform.translation.x;
            let alvo_y = player_transform.translation.y + 100.0;
            let velocidade_da_camera = 4.0; 

            camera_transform.translation.x += (alvo_x - camera_transform.translation.x) * velocidade_da_camera * time.delta_seconds();
            camera_transform.translation.y += (alvo_y - camera_transform.translation.y) * velocidade_da_camera * time.delta_seconds();
        }
    }
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas, &Velocity), With<Player>>,
) {
    for (mut timer, mut atlas, velocity) in &mut query {
        timer.0.tick(time.delta());
        
        if timer.0.just_finished() {
            // Se a velocidade no eixo X for maior que zero, ele está andando
            let is_moving = velocity.linvel.x.abs() > 0.1;

            if is_moving {
                // --- ANIMAÇÃO DE CORRIDA (Frames 4 a 7) ---
                if atlas.index < 4 || atlas.index > 7 {
                    atlas.index = 4;
                } else {
                    atlas.index = 4 + ((atlas.index - 4 + 1) % 4);
                }
            } else {
                // --- ANIMAÇÃO DE PARADO (Frames 0 a 3) ---
                if atlas.index > 3 {
                    atlas.index = 0;
                } else {
                    atlas.index = (atlas.index + 1) % 4;
                }
            }
        }
    }
}