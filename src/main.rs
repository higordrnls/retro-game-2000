use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game - Fase 2: Plataformas".into(),
                resolution: (360.0, 640.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        // Mantemos a ordem lógica de execução dos sistemas
        .add_systems(Update, (aplicar_gravidade, controle_joystick, mover_jogador, animate_player).chain())
        .run();
}

#[derive(Component)]
struct Jogador {
    velocidade_x: f32,
    velocidade_y: f32,
    esta_no_chao: bool,
}

#[derive(Component)]
struct AnimationTimer(Timer);

// --- NOVO: Componente para identificar superfícies onde o jogador pode pisar ---
#[derive(Component)]
struct Plataforma {
    tamanho: Vec2,
}

#[derive(Component)]
struct BaseJoystick;

#[derive(Component)]
struct ManeteJoystick;

#[derive(Component)]
struct BotaoPulo;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Sua matemática exata e corrigida do Spritesheet
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(314, 370), 
        4,                    
        4,                    
        None,                 
        None,                 
    );
    let layout_handle = texture_atlas_layouts.add(layout);
    let textura_personagem = asset_server.load("meu_personagem_spritesheet.png");

    // Spawna o Jogador um pouco mais alto para testar a queda inicial
    commands.spawn((
        SpriteBundle {
            texture: textura_personagem,
            transform: Transform::from_xyz(0.0, 200.0, 2.0).with_scale(Vec3::splat(0.5)), 
            ..default()
        },
        TextureAtlas {
            layout: layout_handle,
            index: 0,
        },
        Jogador {
            velocidade_x: 0.0,
            velocidade_y: 0.0,
            esta_no_chao: false,
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    // --- CONSTRUÇÃO DO MUNDO (Fase 2) ---

    // 1. Chão Principal (Verde) - Posicionado logo acima do Joystick
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.6, 0.3),
                custom_size: Some(Vec2::new(360.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -110.0, 1.0),
            ..default()
        },
        Plataforma { tamanho: Vec2::new(360.0, 20.0) },
    ));

    // 2. Plataforma Flutuante Baixa (Esquerda)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 0.3, 0.1),
                custom_size: Some(Vec2::new(140.0, 15.0)),
                ..default()
            },
            transform: Transform::from_xyz(-70.0, -10.0, 1.0),
            ..default()
        },
        Plataforma { tamanho: Vec2::new(140.0, 15.0) },
    ));

    // 3. Plataforma Flutuante Alta (Direita)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 0.3, 0.1),
                custom_size: Some(Vec2::new(140.0, 15.0)),
                ..default()
            },
            transform: Transform::from_xyz(70.0, 90.0, 1.0),
            ..default()
        },
        Plataforma { tamanho: Vec2::new(140.0, 15.0) },
    ));


    // --- CONTROLES VIRTUAIS ---
    // Base do Joystick
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.2, 0.2, 0.2, 0.6),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            transform: Transform::from_xyz(-100.0, -220.0, 10.0),
            ..default()
        },
        BaseJoystick,
    )).with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(35.0, 35.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..default()
            },
            ManeteJoystick,
        ));
    });

    // Botão de Pulo
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.0, 0.6, 0.9),
                custom_size: Some(Vec2::new(75.0, 75.0)),
                ..default()
            },
            transform: Transform::from_xyz(100.0, -220.0, 10.0),
            ..default()
        },
        BotaoPulo,
    ));
}

// --- REFATORADO: Gravidade com checagem de colisão em múltiplas plataformas ---
fn aplicar_gravidade(
    time: Res<Time>, 
    mut query_jogador: Query<(&mut Transform, &mut Jogador), Without<Plataforma>>,
    query_plataformas: Query<(&Transform, &Plataforma)>,
) {
    let gravidade = -1200.0; 
    let delta = time.delta_seconds();

    for (mut transform_jog, mut jogador) in query_jogador.iter_mut() {
        // Se não estiver pisando em nada, a gravidade puxa para baixo
        if !jogador.esta_no_chao {
            jogador.velocidade_y += gravidade * delta;
        }

        // Calcula onde o boneco quer ir neste frame
        let proximo_y = transform_jog.translation.y + jogador.velocidade_y * delta;
        let jog_x = transform_jog.translation.x;

        // Caixa de colisão baseada no tamanho real do seu boneco escalado (0.5)
        let jogador_meia_largura = 35.0; 
        let jogador_meia_altura = 92.0; // Distância exata do centro do sprite até os pés

        let mut pousou = false;
        let mut y_corrigido = proximo_y;

        // Só tentamos pousar se o jogador estiver caindo ou parado verticalmente
        if jogador.velocidade_y <= 0.0 {
            for (transform_plat, plataforma) in query_plataformas.iter() {
                let plat_x = transform_plat.translation.x;
                let plat_y = transform_plat.translation.y;
                let plat_meia_l = plataforma.tamanho.x / 2.0;
                let plat_meia_a = plataforma.tamanho.y / 2.0;

                // 1. Checa se o jogador está alinhado horizontalmente com a plataforma
                if jog_x + jogador_meia_largura > plat_x - plat_meia_l 
                    && jog_x - jogador_meia_largura < plat_x + plat_meia_l 
                {
                    let topo_plat = plat_y + plat_meia_a;
                    let pes_atuais = transform_jog.translation.y - jogador_meia_altura;
                    let pes_proximos = proximo_y - ManhattanDistance_Y(jogador_meia_altura);

                    // 2. Checa se os pés atravessaram o topo da plataforma vindo de cima
                    if pes_atuais >= topo_plat - 12.0 && pes_proximos <= topo_plat {
                        pousou = true;
                        y_corrigido = topo_plat + jogador_meia_altura;
                        break; // Pousou na plataforma mais alta encontrada, pode parar o loop
                    }
                }
            }
        }

        if pousou {
            transform_jog.translation.y = y_corrigido;
            jogador.velocidade_y = 0.0;
            jogador.esta_no_chao = true;
        } else {
            // Se não colidiu com nada, aplica o movimento e define que ele está no ar (andou para fora da borda)
            transform_jog.translation.y = proximo_y;
            jogador.esta_no_chao = false;
        }
    }
}

// Helper interno apenas para legibilidade matemática da colisão vertical
#[allow(non_snake_case)]
fn ManhattanDistance_Y(val: f32) -> f32 { val }

fn controle_joystick(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut query_jogador: Query<&mut Jogador>,
    q_base: Query<&Transform, With<BaseJoystick>>,
    mut q_manete: Query<&mut Transform, (With<ManeteJoystick>, Without<BaseJoystick>)>,
    q_pulo: Query<&Transform, (With<BotaoPulo>, Without<BaseJoystick>, Without<ManeteJoystick>)>,
) {
    let mut direcao_x = 0.0;
    let mut offset_manete_x = 0.0;
    let mut tentou_pular = false;

    if mouse_input.pressed(MouseButton::Left) {
        if let Some(window) = windows.iter().next() {
            if let Some(pos_cursor) = window.cursor_position() {
                let pos_mouse_bevy = Vec2::new(
                    pos_cursor.x - window.width() / 2.0,
                    (window.height() / 2.0) - pos_cursor.y,
                );

                if let Ok(transform_base) = q_base.get_single() {
                    let pos_base = transform_base.translation.truncate();
                    let distancia = pos_mouse_bevy.distance(pos_base);

                    if distancia < 120.0 {
                        let delta_x = pos_mouse_bevy.x - pos_base.x;
                        if delta_x > 15.0 {
                            direcao_x = 1.0;
                            offset_manete_x = 25.0;
                        } else if delta_x < -15.0 {
                            direcao_x = -1.0;
                            offset_manete_x = -25.0;
                        }
                    }
                }

                if let Ok(transform_pulo) = q_pulo.get_single() {
                    let pos_pulo = transform_pulo.translation.truncate();
                    if pos_mouse_bevy.distance(pos_pulo) < 50.0 && mouse_input.just_pressed(MouseButton::Left) {
                        tentou_pular = true;
                    }
                }
            }
        }
    }

    if let Ok(mut transform_manete) = q_manete.get_single_mut() {
        transform_manete.translation.x = offset_manete_x;
    }

    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direcao_x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        direcao_x = 1.0;
    }
    if keyboard_input.just_pressed(KeyCode::Space) 
        || keyboard_input.just_pressed(KeyCode::KeyW) 
        || keyboard_input.just_pressed(KeyCode::ArrowUp) 
    {
        tentou_pular = true;
    }

    for mut jogador in query_jogador.iter_mut() {
        jogador.velocidade_x = direcao_x * 250.0;

        if tentou_pular && jogador.esta_no_chao {
            jogador.velocidade_y = 570.0; // Dei um pequeno boost no pulo para alcançar a plataforma mais alta com folga
            jogador.esta_no_chao = false;
        }
    }
}

fn mover_jogador(time: Res<Time>, mut query: Query<(&mut Transform, &mut Sprite, &Jogador)>) {
    let delta = time.delta_seconds();
    for (mut transform, mut sprite, jogador) in query.iter_mut() {
        transform.translation.x += jogador.velocidade_x * delta;
        
        if transform.translation.x < -160.0 { transform.translation.x = -160.0; }
        if transform.translation.x > 160.0 { transform.translation.x = 160.0; }

        if jogador.velocidade_x > 0.1 {
            sprite.flip_x = false;
        } else if jogador.velocidade_x < -0.1 {
            sprite.flip_x = true;
        }
    }
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas, &Jogador)>,
) {
    for (mut timer, mut atlas, jogador) in query.iter_mut() {
        let is_moving = jogador.velocidade_x.abs() > 0.1;

        if is_moving {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                if atlas.index < 4 || atlas.index > 7 {
                    atlas.index = 4;
                } else {
                    atlas.index = 4 + ((atlas.index - 4 + 1) % 4);
                }
            }
        } else {
            atlas.index = 0;
        }
    }
}