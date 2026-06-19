use bevy::prelude::*;
use bevy_rapier2d::prelude::*; 

#[derive(Component)]
struct Player {
    saltos: u32,
}

// NOVO: Componente para controlar o tempo de cada frame da animação
#[derive(Component)]
struct AnimationTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(360.0, 640.0),
                title: "Meu RPG Y2K".into(),
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest())) // Mantém o Pixel Art nítido
        
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default()) 
        
        .add_systems(Startup, setup_game)
        // ADICIONADO: 'animar_personagem' rodando junto no Update
        .add_systems(Update, (mover_jogador, camera_seguidora, animar_personagem)) 
        .run();
}

fn setup_game(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, // ADICIONADO: Para criar a grade de animação
) {
    commands.spawn((Camera2dBundle::default(), SpatialBundle::default()));

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

    // CONFIGURAÇÃO DA ANIMACAO:
    // Supondo que cada frame do seu boneco tem 64x64 pixels, dispostos em 4 colunas e 3 linhas.
    // (Ajuste o UVec2::new(64, 64) se o tamanho do frame for diferente, ex: 32x32)
    let layout = TextureAtlasLayout::from_grid(UVec2::new(64, 64), 4, 3, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    // JOGADOR COM SPRITESHEET
    commands.spawn((
        SpriteBundle {
            // ATENÇÃO: Mude aqui para o nome exato do arquivo da sua spritesheet nova!
            texture: asset_server.load("meu_personagem_spritesheet.png"), 
            sprite: Sprite {
                // Dizemos ao Bevy para usar o fatiamento em grade (Atlas) começando no frame 0
                atlas: Some(TextureAtlas {
                    layout: layout_handle,
                    index: 0,
                }),
                ..default()
            },
            ..default()
        },
        Player { saltos: 0 }, 
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // Troca de frame a cada 0.1 segundos
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::cuboid(25.0, 25.0),
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

// NOVO SISTEMA: Controla qual linha de animação rodar baseado no movimento físico
fn animar_personagem(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    if let Ok((vel, mut timer, mut sprite)) = query.get_single_mut() {
        // Atualiza o relógio interno da animação
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            if let Some(atlas) = &mut sprite.atlas {
                // Define qual o frame inicial e final baseado no estado físico:
                let (frame_inicial, frame_final) = if vel.linvel.y.abs() > 0.5 {
                    (8, 11) // Linha 3 (Frames de 8 a 11): Pulando/Caindo
                } else if vel.linvel.x.abs() > 0.1 {
                    (4, 7)  // Linha 2 (Frames de 4 a 7): Correndo
                } else {
                    (0, 3)  // Linha 1 (Frames de 0 a 3): Parado (Idle)
                };

                // Se o frame atual estiver fora da animação correta, reseta para o começo dela
                if atlas.index < frame_inicial || atlas.index >= frame_final {
                    atlas.index = frame_inicial;
                } else {
                    // Caso contrário, avança 1 frame
                    atlas.index += 1;
                }
            }
        }
    }
}