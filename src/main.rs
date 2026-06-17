use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game 2000 - Among Us D-Pad".into(),
                resolution: (360.0, 640.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (aplicar_gravidade, controle_joystick, mover_jogador).chain())
        .run();
}

#[derive(Component)]
struct Jogador {
    velocidade_x: f32,
    velocidade_y: f32,
    esta_no_chao: bool,
}

// Marcadores para o Joystick renderizado na tela
#[derive(Component)]
struct BaseJoystick;

#[derive(Component)]
struct ManeteJoystick;

#[derive(Component)]
struct BotaoPulo;

fn setup(mut commands: Commands) {
    // Câmera do jogo
    commands.spawn(Camera2dBundle::default());

    // 1. O nosso herói (Bloco Verde)
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

    // --- CONTROLES ESTILO AMONG US VIA SPRITES (SUPER ROBUSTO) ---

    // 2. Base do Joystick (Círculo/Quadrado de fundo no canto inferior esquerdo)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.2, 0.2, 0.2, 0.6), // Cinza escuro transparente irado
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            transform: Transform::from_xyz(-100.0, -220.0, 10.0), // Posicionado no canto inferior esquerdo
            ..default()
        },
        BaseJoystick,
    )).with_children(|parent| {
        // 3. O Manete (Bolinha branca do meio que mexe)
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 1.0, 1.0), // Brancão
                    custom_size: Some(Vec2::new(35.0, 35.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 1.0), // Centralizado na base
                ..default()
            },
            ManeteJoystick,
        ));
    });

    // 4. Botão de Pulo (No canto inferior direito)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.0, 0.6, 0.9), // Azul bonito
                custom_size: Some(Vec2::new(75.0, 75.0)),
                ..default()
            },
            transform: Transform::from_xyz(100.0, -220.0, 10.0), // Canto inferior direito
            ..default()
        },
        BotaoPulo,
    ));
}

fn aplicar_gravidade(mut query: Query<(&mut Transform, &mut Jogador)>) {
    let gravidade = -1000.0; 
    let chao_y = -100.0; // Chão acima dos botões para não tampar o herói

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

    // Se o usuário estiver clicando/tocando na tela
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(window) = windows.iter().next() {
            if let Some(pos_cursor) = window.cursor_position() {
                // Converte a coordenada do mouse para o espaço 2D do Bevy (com centro em 0,0)
                let pos_mouse_bevy = Vec2::new(
                    pos_cursor.x - window.width() / 2.0,
                    (window.height() / 2.0) - pos_cursor.y,
                );

                // 1. Verifica clique no Joystick
                if let Ok(transform_base) = q_base.get_single() {
                    let pos_base = transform_base.translation.truncate();
                    let distancia = pos_mouse_bevy.distance(pos_base);

                    // Se clicou dentro do raio do D-pad
                    if distancia < 60.0 {
                        let delta_x = pos_mouse_bevy.x - pos_base.x;
                        if delta_x > 10.0 {
                            direcao_x = 1.0;
                            offset_manete_x = 20.0; // Empurra bolinha pra direita
                        } else if delta_x < -10.0 {
                            direcao_x = -1.0;
                            offset_manete_x = -20.0; // Empurra bolinha pra esquerda
                        }
                    }
                }

                // 2. Verifica clique no Botão de Pulo (Detecta clique síncrono)
                if let Ok(transform_pulo) = q_pulo.get_single() {
                    let pos_pulo = transform_pulo.translation.truncate();
                    if pos_mouse_bevy.distance(pos_pulo) < 40.0 && mouse_input.just_pressed(MouseButton::Left) {
                        tentou_pular = true;
                    }
                }
            }
        }
    }

    // Suaviza e atualiza a posição do analógico visualmente (Feedback UX)
    if let Ok(mut transform_manete) = q_manete.get_single_mut() {
        transform_manete.translation.x = offset_manete_x;
    }

    // Fallback Teclado para testes rápidos no PC
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direcao_x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        direcao_x = 1.0;
    }
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::KeyW) {
        tentou_pular = true;
    }

    // Aplica os movimentos finais no Jogador
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