use bevy::prelude::*;
use bevy::input::touch::TouchPhase; // Não esquece de importar isso lá em cima!

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>, // <-- Adicione isso
    mut game_state: ResMut<NextState<GameState>>,
) {
    let mut comando_de_start = false;

    if keyboard_input.just_pressed(KeyCode::Space) { comando_de_start = true; }
    if mouse_input.just_pressed(MouseButton::Left) { comando_de_start = true; }

    // Verifica se há qualquer toque na tela
    if touches.any_just_pressed() { // <-- Isso é o "pulo do gato"
        comando_de_start = true;
    }

    if comando_de_start {
        game_state.set(GameState::Playing);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Runner - Edição Floresta de Musgo".into(),
                resolution: (360.0, 640.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.06, 0.12))) // 1. Fundo azul-escuro da floresta à noite
        .init_state::<GameState>()
        .insert_resource(Progresso {
            xp: 0,
            nivel: 1,
            pontuacao: 0,
        })
        .insert_resource(EstadoMundo {
            proximo_spawn_x: 300.0,
        })
        .add_systems(Startup, setup_camera)
        
        // --- FLUXO DA TELA DE INÍCIO (MENU) ---
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, (atualizar_menu, piscar_texto_menu).run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), limpar_menu)
        
        // --- FLUXO DO JOGO ATIVO (PLAYING) ---
        .add_systems(OnEnter(GameState::Playing), setup_jogo)
        .add_systems(
            Update,
            (
                controle_joystick,       
                mover_jogador,           
                aplicar_gravidade,       
                gerenciar_morte,         
                seguir_camera,           
                gerar_mundo_procedural,  
                limpar_mundo_antigo,     
                detectar_coleta,         
                atualizar_hud,           
                animate_player,          
                animar_coletaveis, // 2. Novo sistema para dar vida aos cristais
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>() 
        // AQUI ESTÁ O SEGREDO:
        .add_systems(Update, input_handler) 
        .run();
}

// --- CONFIGURAÇÃO DE ESTADOS ---

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
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
struct ElementoMenu;

#[derive(Component)]
struct TextoPiscante {
    timer: Timer,
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

// --- TELA DE INÍCIO (MENU) ---

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        ElementoMenu,
    )).with_children(|parent| {
        parent.spawn(
            TextBundle::from_section(
                "RUST RUNNER",
                TextStyle {
                    font_size: 42.0,
                    color: Color::srgb(0.0, 0.9, 0.6), 
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(50.0)),
                ..default()
            })
        );

        parent.spawn((
            TextBundle::from_section(
                "PRESSIONE ESPAÇO\nPARA COMEÇAR",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextoPiscante {
                timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            },
        ));
    });
}

fn atualizar_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) || mouse_input.just_pressed(MouseButton::Left) {
        next_state.set(GameState::Playing);
    }
}

fn piscar_texto_menu(time: Res<Time>, mut query: Query<(&mut Visibility, &mut TextoPiscante)>) {
    for (mut visibility, mut pisca) in query.iter_mut() {
        pisca.timer.tick(time.delta());
        if pisca.timer.just_finished() {
            *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                _ => Visibility::Visible,
            };
        }
    }
}

fn limpar_menu(mut commands: Commands, query: Query<Entity, With<ElementoMenu>>) {
    for entidade in query.iter() {
        commands.entity(entidade).despawn_recursive();
    }
}

// --- JOGO ATIVO (PLAYING) ---

fn setup_jogo(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    query_camera: Query<Entity, With<Camera>>,
) {
    let camera_entity = query_camera.single();

    // Controles com paleta cinza-azulada semi-transparente combinando com o tema
    commands.entity(camera_entity).with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(0.12, 0.16, 0.22, 0.5),
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
                        color: Color::srgba(0.4, 0.46, 0.55, 0.8),
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
                    color: Color::srgba(0.15, 0.35, 0.65, 0.6),
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

    // Plataforma Inicial Estilizada (Rocha com Musgo acoplado)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.22, 0.16, 0.14), // Base de rocha escura
                custom_size: Some(Vec2::new(500.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -110.0, 1.0),
            ..default()
        },
        Plataforma { tamanho: Vec2::new(500.0, 20.0) },
    )).with_children(|parent| {
        // Musgo superior da plataforma inicial
        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.18, 0.52, 0.24), // Verde Musgo Vibrante
                custom_size: Some(Vec2::new(500.0, 4.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 10.0 - 2.0, 0.1),
            ..default()
        });
    });

    // HUD Nova: Caixa de texto imitando placa de pedra medieval/retro
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(3.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::srgb(0.14, 0.17, 0.2).into(), // Fundo pedra escura
        border_color: Color::srgb(0.32, 0.36, 0.4).into(),     // Contorno de pedra clara
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 15.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ),
            TextoHUD,
        ));
    });
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

fn gerenciar_morte(
    mut commands: Commands,
    mut query_jogador: Query<(&mut Transform, &mut Jogador)>,
    query_objetos: Query<Entity, Or<(With<Plataforma>, With<Coletavel>)>>,
    mut estado_mundo: ResMut<EstadoMundo>,
    mut progresso: ResMut<Progresso>,
) {
    if let Ok((mut trans_jog, mut jogador)) = query_jogador.get_single_mut() {
        if trans_jog.translation.y < -350.0 {
            // Remove tudo recursivamente para evitar sobras de musgos soltos
            for entidade in query_objetos.iter() {
                commands.entity(entidade).despawn_recursive();
            }

            trans_jog.translation = Vec3::new(0.0, 200.0, 2.0);
            jogador.velocidade_x = 0.0;
            jogador.velocidade_y = 0.0;
            jogador.esta_no_chao = false;
            estado_mundo.proximo_spawn_x = 300.0;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.22, 0.16, 0.14),
                        custom_size: Some(Vec2::new(500.0, 20.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, -110.0, 1.0),
                    ..default()
                },
                Plataforma { tamanho: Vec2::new(500.0, 20.0) },
            )).with_children(|parent| {
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.18, 0.52, 0.24),
                        custom_size: Some(Vec2::new(500.0, 4.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 10.0 - 2.0, 0.1),
                    ..default()
                });
            });

            progresso.pontuacao = 0;
            progresso.xp = 0;
            println!("💀 Game Over! Voltando ao início da rodada...");
        }
    }
}

fn seguir_camera(
    query_jogador: Query<&Transform, With<Jogador>>,
    mut query_camera: Query<&mut Transform, (With<Camera2d>, Without<Jogador>)>,
) {
    if let Ok(trans_jog) = query_jogador.get_single() {
        if let Ok(mut trans_cam) = query_camera.get_single_mut() {
            trans_cam.translation.x = trans_jog.translation.x;
        }
    }
}

fn gerar_mundo_procedural(
    mut commands: Commands,
    query_jogador: Query<&Transform, With<Jogador>>,
    mut estado_mundo: ResMut<EstadoMundo>,
) {
    if let Ok(trans_jog) = query_jogador.get_single() {
        while estado_mundo.proximo_spawn_x < trans_jog.translation.x + 1200.0 {
            let current_x = estado_mundo.proximo_spawn_x;

            let hash = (current_x * 0.05).sin().fract().abs();
            let y_plataforma = -110.0 + (hash * 120.0);
            let largura_plataforma = 110.0 + (hash * 70.0);

            // Gerando a Rocha Escura
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.25, 0.18, 0.15), // Pedra escura da floresta
                        custom_size: Some(Vec2::new(largura_plataforma, 15.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(current_x, y_plataforma, 1.0),
                    ..default()
                },
                Plataforma { tamanho: Vec2::new(largura_plataforma, 15.0) },
            )).with_children(|parent| {
                // Acoplando o Musgo Verde no topo da plataforma gerada
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.15, 0.55, 0.22), // Grama/Musgo verde brilhante
                        custom_size: Some(Vec2::new(largura_plataforma, 4.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 7.5 - 2.0, 0.1),
                    ..default()
                });
            });

            // Cristal Amarelo de Ouro Brilhante
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 0.84, 0.0), // Amarelo Dourado
                        custom_size: Some(Vec2::new(12.0, 12.0)),
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
        let limite_traseiro = trans_jog.translation.x - 600.0;
        for (entidade, transform) in query_objetos.iter() {
            if transform.translation.x < limite_traseiro {
                commands.entity(entidade).despawn_recursive(); // Despawn com os filhos inclusos
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

// Faz os cristais rodarem continuamente, gerando um efeito estético polido
fn animar_coletaveis(time: Res<Time>, mut query: Query<&mut Transform, With<Coletavel>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_z(2.0 * time.delta_seconds());
    }
}

fn atualizar_hud(progresso: Res<Progresso>, mut query_texto: Query<&mut Text, With<TextoHUD>>) {
    if let Ok(mut text) = query_texto.get_single_mut() {
        text.sections[0].value = format!(
            "🎖️ NÍVEL: {}  |  XP: {}/100\n💰 PONTOS: {}",
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