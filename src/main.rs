use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game - Fase 4: Geração Infinita".into(),
                resolution: (360.0, 640.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Progresso {
            xp: 0,
            nivel: 1,
            pontuacao: 0,
        })
        // NOVO: Inicializa o timer que vai gerar novos itens a cada 2.5 segundos
        .insert_resource(TimerSpawn(Timer::from_seconds(2.5, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                aplicar_gravidade,
                controle_joystick,
                mover_jogador,
                animate_player,
                detectar_coleta,
                atualizar_hud,
                gerar_itens_procedurais, // Novo sistema de spawn infinito
            )
                .chain(),
        )
        .run();
}

// --- COMPONENTES & RECURSOS ---

#[derive(Resource)]
struct Progresso {
    xp: u32,
    nivel: u32,
    pontuacao: u32,
}

// NOVO: Timer global para controlar o fluxo de novos coletáveis
#[derive(Resource)]
struct TimerSpawn(Timer);

#[derive(Component)]
struct Coletavel {
    valor_xp: u32,
    valor_pontos: u32,
}

#[derive(Component)]
struct TextoHUD;

#[derive(Component)]
struct Jogador {
    velocidade_x: f32,
    velocidade_y: f32,
    esta_no_chao: bool,
}

#[derive(Component)]
struct AnimationTimer(Timer);

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

    let layout = TextureAtlasLayout::from_grid(UVec2::new(314, 370), 4, 4, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);
    let textura_personagem = asset_server.load("meu_personagem_spritesheet.png");

    // Spawn do Jogador
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

    // --- CENÁRIO FIXO ---
    // Chão
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

    // Plataforma Esquerda
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

    // Plataforma Direita
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

    // Inicializamos apenas 1 item inicial para o jogador ver logo de cara
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.9, 0.8, 0.1),
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -80.0, 1.5),
            ..default()
        },
        Coletavel { valor_xp: 25, valor_pontos: 100 },
    ));

    // HUD de Status
    commands.spawn((
        TextBundle::from_section(
            "Carregando status...",
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        }),
        TextoHUD,
    ));

    // CONTROLES VIRTUAIS
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

// --- NOVO: SISTEMA DE GERAÇÃO PROCEDURAL INFINITA ---
fn gerar_itens_procedurais(
    mut commands: Commands,
    time: Res<Time>,
    mut timer_spawn: ResMut<TimerSpawn>,
) {
    // Avança o timer baseado no delta time do frame
    timer_spawn.0.tick(time.delta());

    if timer_spawn.0.just_finished() {
        let segundos_corridos = time.elapsed_seconds();

        // Algoritmo matemático para gerar posições determinísticas pseudo-aleatórias
        // Usamos o seno e cosseno do tempo para criar variações caóticas estáveis de X e Y
        let pos_x = (segundos_corridos * 4.5).sin() * 140.0; 
        let pos_y = -50.0 + (segundos_corridos * 2.3).cos() * 130.0;

        // Variação de cor baseada no tempo (alterna entre Dourado/Amarelo e Ciano de alta recompensa)
        let eh_item_raro = (segundos_corridos as u32) % 3 == 0;
        let cor = if eh_item_raro { Color::srgb(0.2, 0.8, 1.0) } else { Color::srgb(0.9, 0.8, 0.1) };
        let xp = if eh_item_raro { 45 } else { 20 };
        let pontos = if eh_item_raro { 250 } else { 80 };

        // Realiza o spawn do novo item gerado dinamicamente
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: cor,
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_xyz(pos_x, pos_y, 1.5),
                ..default()
            },
            Coletavel { valor_xp: xp, valor_pontos: pontos },
        ));
    }
}

// --- SISTEMAS DE LOOP, FISICA E UI (Fase 3) ---

fn detectar_coleta(
    mut commands: Commands,
    query_jogador: Query<&Transform, With<Jogador>>,
    query_coletaveis: Query<(Entity, &Transform, &Coletavel)>,
    mut progresso: ResMut<Progresso>,
) {
    if let Ok(trans_jog) = query_jogador.get_single() {
        let jog_pos = trans_jog.translation;
        let jog_meia_largura = 30.0;
        let jog_meia_altura = 75.0;

        for (entidade_item, trans_item, coletavel) in query_coletaveis.iter() {
            let item_pos = trans_item.translation;
            let item_meia_tam = 9.0;

            if jog_pos.x + jog_meia_largura > item_pos.x - item_meia_tam
                && jog_pos.x - jog_meia_largura < item_pos.x + item_meia_tam
                && jog_pos.y + jog_meia_altura > item_pos.y - item_meia_tam
                && jog_pos.y - jog_meia_altura < item_pos.y + item_meia_tam
            {
                commands.entity(entidade_item).despawn();
                progresso.pontuacao += coletavel.valor_pontos;
                progresso.xp += coletavel.valor_xp;

                if progresso.xp >= 100 {
                    progresso.xp -= 100;
                    progresso.nivel += 1;
                    println!("🎉 LEVEL UP! Você alcançou o nível {}!", progresso.nivel);
                }
            }
        }
    }
}

fn atualizar_hud(progresso: Res<Progresso>, mut query_texto: Query<&mut Text, With<TextoHUD>>) {
    if let Ok(mut text) = query_texto.get_single_mut() {
        text.sections[0].value = format!(
            "NÍVEL: {}  |  XP: {}/100\nPONTOS: {}",
            progresso.nivel, progresso.xp, progresso.pontuacao
        );
    }
}

fn aplicar_gravidade(
    time: Res<Time>,
    mut query_jogador: Query<(&mut Transform, &mut Jogador), Without<Plataforma>>,
    query_plataformas: Query<(&Transform, &Plataforma)>,
) {
    let gravidade = -1200.0;
    let delta = time.delta_seconds();

    for (mut transform_jog, mut jogador) in query_jogador.iter_mut() {
        if !jogador.esta_no_chao {
            jogador.velocidade_y += gravidade * delta;
        }

        let proximo_y = transform_jog.translation.y + jogador.velocidade_y * delta;
        let jog_x = transform_jog.translation.x;
        let jogador_meia_largura = 35.0;
        let jogador_meia_altura = 92.0;

        let mut pousou = false;
        let mut y_corrigido = proximo_y;

        if jogador.velocidade_y <= 0.0 {
            for (transform_plat, plataforma) in query_plataformas.iter() {
                let plat_x = transform_plat.translation.x;
                let plat_y = transform_plat.translation.y;
                let plat_meia_l = plataforma.tamanho.x / 2.0;
                let plat_meia_a = plataforma.tamanho.y / 2.0;

                if jog_x + jogador_meia_largura > plat_x - plat_meia_l
                    && jog_x - jogador_meia_largura < plat_x + plat_meia_l
                {
                    let topo_plat = plat_y + plat_meia_a;
                    let pes_atuais = transform_jog.translation.y - jogador_meia_altura;
                    let pes_proximos = proximo_y - jogador_meia_altura;

                    if pes_atuais >= topo_plat - 12.0 && pes_proximos <= topo_plat {
                        pousou = true;
                        y_corrigido = topo_plat + jogador_meia_altura;
                        break;
                    }
                }
            }
        }

        if pousou {
            transform_jog.translation.y = y_corrigido;
            jogador.velocidade_y = 0.0;
            jogador.esta_no_chao = true;
        } else {
            transform_jog.translation.y = proximo_y;
            jogador.esta_no_chao = false;
        }
    }
}

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
                    if pos_mouse_bevy.distance(pos_base) < 120.0 {
                        let delta_x = pos_mouse_bevy.x - pos_base.x;
                        if delta_x > 15.0 { direcao_x = 1.0; offset_manete_x = 25.0; }
                        else if delta_x < -15.0 { direcao_x = -1.0; offset_manete_x = -25.0; }
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

    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) { direcao_x = -1.0; }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) { direcao_x = 1.0; }
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
        tentou_pular = true;
    }

    for mut jogador in query_jogador.iter_mut() {
        jogador.velocidade_x = direcao_x * 250.0;
        if tentou_pular && jogador.esta_no_chao {
            jogador.velocidade_y = 570.0;
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

        if jogador.velocidade_x > 0.1 { sprite.flip_x = false; }
        else if jogador.velocidade_x < -0.1 { sprite.flip_x = true; }
    }
}

fn animate_player(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut TextureAtlas, &Jogador)>) {
    for (mut timer, mut atlas, jogador) in query.iter_mut() {
        if jogador.velocidade_x.abs() > 0.1 {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                if atlas.index < 4 || atlas.index > 7 { atlas.index = 4; }
                else { atlas.index = 4 + ((atlas.index - 4 + 1) % 4); }
            }
        } else {
            atlas.index = 0;
        }
    }
}