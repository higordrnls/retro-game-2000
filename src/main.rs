use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game 2000 - Among Us Joystick".into(),
                resolution: (360.0, 640.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_jogo, setup_joystick))
        .add_systems(Update, (aplicar_gravidade, controle_joystick, mover_jogador).chain())
        .run();
}

#[derive(Component)]
struct Jogador {
    velocidade_x: f32,
    velocidade_y: f32,
    esta_no_chao: bool,
}

// Marcadores para a UI do Joystick
#[derive(Component)]
struct BaseJoystick;

#[derive(Component)]
struct ManeteJoystick;

#[derive(Component)]
struct BotaoPulo;

fn setup_jogo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Nosso querido bloco verde herói
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.0, 0.8, 0.4), 
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 100.0, 0.0), 
            ..default()
        },
        Jogador {
            velocidade_x: 0.0,
            velocidade_y: 0.0,
            esta_no_chao: false,
        },
    ));
}

fn setup_joystick(mut commands: Commands) {
    // Container da UI (ocupa a parte de baixo)
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(160.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            
            // --- BASE DO JOYSTICK (Estilo Among Us) ---
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Percent(50.0)), // Deixa a base Redonda!
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.6)), // Transparente estiloso
                    ..default()
                },
                BaseJoystick,
            )).with_children(|base| {
                // O Manete (a bolinha do meio que mexe)
                base.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            border_radius: BorderRadius::all(Val::Percent(50.0)), // Manete redondo!
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(1.0, 1.0, 1.0)), // Bolinha branca
                        ..default()
                    },
                    ManeteJoystick,
                ));
            });

            // --- BOTÃO DE PULO ULTRA CLEAN ---
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(80.0),
                        height: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Percent(50.0)), // Botão de pulo redondo!
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.0, 0.6, 0.9)),
                    ..default()
                },
                BotaoPulo,
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section("JUMP", TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }));
            });
        });
}

fn aplicar_gravidade(mut query: Query<(&mut Transform, &mut Jogador)>) {
    let gravidade = -1000.0; 
    let chao_y = -80.0;

    for (mut transform, mut jogador) in query.iter_mut() {
        if !jogador.esta_no_chao {
            jogador.velocidade_y += gravidade * 0.016;
        }
        transform.translation.y += jogador.velocidade_y * 0.016;

        if transform.translation.y <= chao_y {
            transform.translation.y = chao_y;
            jogador.velocidade_y = 0.0;
            jogador.esta_no_chao = true;
        }
    }
}

fn controle_joystick(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut query_jogador: Query<&mut Jogador>,
    q_base: Query<(&Interaction, &Node, &GlobalTransform), With<BaseJoystick>>,
    mut q_manete: Query<&mut Style, With<ManeteJoystick>>,
    q_pulo: Query<&Interaction, With<BotaoPulo>>,
) {
    let mut direcao_x = 0.0;
    let mut animar_manete_x = 0.0;

    // 1. Lógica do Toque/Clique na Base do Joystick
    if let Ok((interaction, node, global_transform)) = q_base.get_single() {
        if *interaction == Interaction::Pressed {
            if let Some(window) = windows.iter().next() {
                if let Some(pos_mouse) = window.cursor_position() {
                    // Descobre onde o clique aconteceu em relação ao centro do Joystick
                    let centro_joystick = global_transform.translation().truncate();
                    let pos_mouse_bevy = Vec2::new(pos_mouse.x, window.height() - pos_mouse.y);
                    let delta = pos_mouse_bevy - centro_joystick;

                    // Se clicou mais para a direita ou esquerda da base
                    if delta.x > 15.0 {
                        direcao_x = 1.0;
                        animar_manete_x = 20.0; // Desloca a bolinha branca para a direita
                    } else if delta.x < -15.0 {
                        direcao_x = -1.0;
                        animar_manete_x = -20.0; // Desloca a bolinha branca para a esquerda
                    }
                }
            }
        }
    }

    // Atualiza a posição visual do manete (feedback UX)
    if let Ok(mut estilo_manete) = q_manete.get_single_mut() {
        estilo_manete.margin = UiRect::left(Val::Px(animar_manete_x));
    }

    // 2. Fallback para Teclado (para facilitar testes no PC)
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direcao_x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        direcao_x = 1.0;
    }

    // 3. Aplica velocidade e Pulo
    let toque_pulo = q_pulo.iter().any(|i| *i == Interaction::Pressed);
    let tentou_pular = keyboard_input.just_pressed(KeyCode::Space) || toque_pulo;

    for mut jogador in query_jogador.iter_mut() {
        jogador.velocidade_x = direcao_x * 250.0;

        if tentou_pular && jogador.esta_no_chao {
            jogador.velocidade_y = 500.0; 
            jogador.esta_no_chao = false; 
        }
    }
}

fn mover_jogador(mut query: Query<(&mut Transform, &Jogador)>) {
    for (mut transform, jogador) in query.iter_mut() {
        transform.translation.x += jogador.velocidade_x * 0.016;
        if transform.translation.x < -160.0 { transform.translation.x = -160.0; }
        if transform.translation.x > 160.0 { transform.translation.x = 160.0; }
    }
}