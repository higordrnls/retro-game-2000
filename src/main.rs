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
        
        // --- AQUI ESTÁ O QUE VOCÊ PROCURA! ---
        // O Update roda a cada frame por segundo (60 vezes por segundo!)
        // Passamos os sistemas dentro de parênteses (uma tupla) separados por vírgula
        .add_systems(Update, (player_movement, animate_player)) 
        
        // Roda o jogo de fato
        .run();
}

fn setup_game(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, 
) {
    // CORREÇÃO: Spawna apenas a câmera, sem o SpatialBundle duplicado!
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

    // 1. CORREÇÃO DO TAMANHO DA GRADE:
    // Dividimos 1254 por 4 colunas = 313 de largura.
    // Se ela tiver 4 linhas de animações, a altura também será 313.
    let layout = TextureAtlasLayout::from_grid(UVec2::new(313, 313), 4, 4, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    // JOGADOR COM SPRITESHEET
    commands.spawn((
        SpriteBundle {
            // CORREÇÃO AQUI: Mudamos para .png para bater com o seu arquivo real!
            texture: asset_server.load("meu_personagem_spritesheet.png"), 
            // Se o nome do seu arquivo for apenas "meu_personagem.png", mude aqui dentro também!
            
            transform: Transform::from_scale(Vec3::splat(0.3)), // Se ele ficar gigante, essa linha diminui o tamanho dele
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
        // 3. AJUSTE DA CAIXA DE COLISÃO:
        // Como o frame agora é maior (313px), aumentei o collider para o boneco não afundar no chão
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
            // Verifica se o jogador está se movendo no eixo X (esquerda/direita)
            let is_moving = velocity.linvel.x.abs() > 0.1;

            if is_moving {
                // --- ANIMAÇÃO DE CORRIDA (Frames 4 a 7) ---
                // Se o index atual não estiver no alcance de corrida, força a ir para o frame 4
                if atlas.index < 4 || atlas.index > 7 {
                    atlas.index = 4;
                } else {
                    // Avança o frame e faz o loop entre 4 e 7
                    atlas.index = 4 + ((atlas.index - 4 + 1) % 4);
                }
            } else {
                // --- ANIMAÇÃO DE PARADO (Frames 0 a 3) ---
                if atlas.index > 3 {
                    atlas.index = 0;
                } else {
                    // Avança o frame e faz o loop entre 0 e 3
                    atlas.index = (atlas.index + 1) % 4;
                }
            }
        }
    }
}