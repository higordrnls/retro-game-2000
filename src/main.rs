use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game - Versão Corrigida".into(),
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
        .insert_resource(EstadoMundo {
            proximo_spawn_x: 300.0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                controle_joystick,       // 1. Escuta seus comandos (Manual)
                mover_jogador,           // 2. Aplica os comandos de movimento
                aplicar_gravidade,       // 3. Calcula queda física
                gerenciar_morte,         // 4. NOVO: Reseta tudo pro início real se cair
                seguir_camera,           // 5. Câmera centralizada clássica
                gerar_mundo_procedural,  // 6. Cria o mundo MUITO à frente (Invisível)
                limpar_mundo_antigo,     // 7. Remove o que ficou muito para trás
                detectar_coleta,         // 8. Verifica itens coletados
                atualizar_hud,           // 9. Atualiza a interface
                animate_player,          // 10. Controla os sprites de corrida
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

#[derive(Resource)]
struct EstadoMundo {
    proximo_spawn_x: f32,
}

#[derive(Component)]
struct Coletavel;

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
    let camera_entity = commands.spawn(Camera2dBundle::default()).id();

    // Controles Virtuais fixados na Câmera
    commands.entity(camera_entity).with_children(|parent| {
        parent.spawn((
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
        )).with_children(|joystick_parent| {
            joystick_parent.spawn((
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

        parent.spawn((
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
    });

    let layout = TextureAtlasLayout::from_grid(UVec2::new(314, 370), 4, 4, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);
    let textura_personagem = asset_server.load("meu_personagem_spritesheet.png");

    // Spawn do Jogador inicial
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
            velocidade_x: 0.0, // Começa parado! Controle é 100% seu agora
            velocidade_y: 0.0,
            esta_no_chao: false,
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    // Plataforma Inicial do Ponto Zero
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.6, 0.3),
                custom_size: Some(Vec2::new(500.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -110.0, 1.0),
            ..default()
        },
        Plataforma { tamanho: Vec2::new(500.0, 20.0) },
    ));

    commands.spawn((
        TextBundle::from_section(
            "Modo Manual Ativo",
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
}

// --- CONTROLE 100% MANUAL E TRADICIONAL ---
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

    // Joystick Virtual
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

    // Teclado (Sobrepõe o joystick se pressionado)
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) { direcao_x = -1.0; }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) { direcao_x = 1.0; }
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
        tentou_pular = true;
    }

    for mut jogador in query_jogador.iter_mut() {
        // Velocidade controlada estritamente por onde você aperta
        jogador.velocidade_x = direcao_x * 240.0;

        if tentou_pular && jogador.esta_no_chao {
            jogador.velocidade_y = 550.0;
            jogador.esta_no_chao = false;
        }
    }
}

fn mover_jogador(time: Res<Time>, mut query: Query<(&mut Transform, &mut Sprite, &Jogador)>) {
    let delta = time.delta_seconds();
    for (mut transform, mut sprite, jogador) in query.iter_mut() {
        transform.translation.x += jogador.velocidade_x * delta;

        if jogador.velocidade_x > 0.1 { sprite.flip_x = false; } 
        else if jogador.velocidade_x < -0.1 { sprite.flip_x = true; }
    }
}

// --- NOVO: SISTEMA QUE CORRIGE O RESET E DELETA O LAG ---
fn gerenciar_morte(
    mut commands: Commands,
    mut query_jogador: Query<(&mut Transform, &mut Jogador)>,
    query_objetos: Query<Entity, Or<(With<Plataforma>, With<Coletavel>)>>,
    mut estado_mundo: ResMut<EstadoMundo>,
    mut progresso: ResMut<Progresso>,
) {
    if let Ok((mut trans_jog, mut jogador)) = query_jogador.get_single_mut() {
        // Se cair abaixo da linha da tela (-350)
        if trans_jog.translation.y < -350.0 {
            
            // 1. Limpa absolutamente todas as entidades antigas para matar o lag de acúmulo
            for entidade in query_objetos.iter() {
                commands.entity(entidade).despawn();
            }

            // 2. Teleporta o jogador de volta para o INÍCIO REAL do jogo
            trans_jog.translation = Vec3::new(0.0, 200.0, 2.0);
            jogador.velocidade_x = 0.0;
            jogador.velocidade_y = 0.0;
            jogador.esta_no_chao = false;

            // 3. Reseta a esteira de criação procedural para o começo
            estado_mundo.proximo_spawn_x = 300.0;

            // 4. Recria a plataforma inicial segura no ponto zero
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.2, 0.6, 0.3),
                        custom_size: Some(Vec2::new(500.0, 20.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, -110.0, 1.0),
                    ..default()
                },
                Plataforma { tamanho: Vec2::new(500.0, 20.0) },
            ));

            // Penalidade: Mantém seu nível de progresso, mas limpa os pontos obtidos naquela tentativa
            progresso.pontuacao = 0;
            progresso.xp = 0;
            println!("💀 Game Over! Todo o mapa foi resetado. Voltando ao início...");
        }
    }
}

// --- CÂMERA CENTRALIZADA CLÁSSICA ---
fn seguir_camera(
    query_jogador: Query<&Transform, With<Jogador>>,
    mut query_camera: Query<&mut Transform, (With<Camera2d>, Without<Jogador>)>,
) {
    if let Ok(trans_jog) = query_jogador.get_single() {
        if let Ok(mut trans_cam) = query_camera.get_single_mut() {
            // A câmera fica presa exatamente no centro do personagem
            trans_cam.translation.x = trans_jog.translation.x;
        }
    }
}

// --- GERAÇÃO PROCEDURAL SEAMLESS (INVISÍVEL) ---
fn gerar_mundo_procedural(
    mut commands: Commands,
    query_jogador: Query<&Transform, With<Jogador>>,
    mut estado_mundo: ResMut<EstadoMundo>,
) {
    if let Ok(trans_jog) = query_jogador.get_single() {
        // CORREÇÃO: Aumentado para 1200 pixels à frente do jogador.
        // Como a câmera só exibe até 180px à frente, os blocos nascem totalmente fora de vista.
        while estado_mundo.proximo_spawn_x < trans_jog.translation.x + 1200.0 {
            let current_x = estado_mundo.proximo_spawn_x;

            let hash = (current_x * 0.05).sin().fract().abs();
            let y_plataforma = -110.0 + (hash * 120.0);
            let largura_plataforma = 110.0 + (hash * 70.0);

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.5, 0.3, 0.1),
                        custom_size: Some(Vec2::new(largura_plataforma, 15.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(current_x, y_plataforma, 1.0),
                    ..default()
                },
                Plataforma { tamanho: Vec2::new(largura_plataforma, 15.0) },
            ));

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.9, 0.8, 0.1),
                        custom_size: Some(Vec2::new(14.0, 14.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(current_x, y_plataforma + 35.0, 1.5),
                    ..default()
                },
                Coletavel,
            ));

            let distancia_proximo_bloco = 170.0 + (hash * 80.0);
            estado_mundo.proximo_spawn_x += distancia_proximo_bloco;
        }
    }
}

fn limpar_mundo_antigo(
    mut commands: Commands,
    query_jogador: Query<&Transform, With<Jogador>>,
    query_objetos: Query<(Entity, &Transform), Or<(With<Plataforma>, With<Coletavel>)>>,
) {
    if let Ok(trans_jog) = query_jogador.get_single() {
        // Limpa o que ficou muito para trás (600px) para economizar processamento
        let limite_traseiro = trans_jog.translation.x - 600.0;
        for (entidade, transform) in query_objetos.iter() {
            if transform.translation.x < limite_traseiro {
                commands.entity(entidade).despawn();
            }
        }
    }
}

fn aplicar_gravidade(
    time: Res<Time>,
    mut query_jogador: Query<(&mut Transform, &mut Jogador), Without<Plataforma>>,
    query_plataformas: Query<(&Transform, &Plataforma)>,
) {
    let gravidade = -1300.0;
    let delta = time.delta_seconds();

    for (mut transform_jog, mut jogador) in query_jogador.iter_mut() {
        if !jogador.esta_no_chao {
            jogador.velocidade_y += gravidade * delta;
        }

        let proximo_y = transform_jog.translation.y + jogador.velocidade_y * delta;
        let jog_x = transform_jog.translation.x;
        let jogador_meia_largura = 25.0;
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

                    if pes_atuais >= topo_plat - 15.0 && pes_proximos <= topo_plat {
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

fn detectar_coleta(
    mut commands: Commands,
    query_jogador: Query<&Transform, With<Jogador>>,
    query_coletaveis: Query<(Entity, &Transform), With<Coletavel>>,
    mut progresso: ResMut<Progresso>,
) {
    if let Ok(trans_jog) = query_jogador.get_single() {
        let jog_pos = trans_jog.translation;
        let jog_meia_largura = 30.0;
        let jog_meia_altura = 75.0;

        for (entidade_item, trans_item) in query_coletaveis.iter() {
            let item_pos = trans_item.translation;
            let item_meia_tam = 10.0;

            if jog_pos.x + jog_meia_largura > item_pos.x - item_meia_tam
                && jog_pos.x - jog_meia_largura < item_pos.x + item_meia_tam
                && jog_pos.y + jog_meia_altura > item_pos.y - item_meia_tam
                && jog_pos.y - jog_meia_altura < item_pos.y + item_meia_tam
            {
                commands.entity(entidade_item).despawn();
                progresso.pontuacao += 100;
                progresso.xp += 20;

                if progresso.xp >= 100 {
                    progresso.xp -= 100;
                    progresso.nivel += 1;
                    println!("🎉 LEVEL UP! Você subiu para o nível {}!", progresso.nivel);
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