use bevy::prelude::*;

fn main() {
    App::new()
        // --- 1. CORREÇÃO DA TELA (Restaurado o tamanho de celular 9:16) ---
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game 2000 - Among Us D-Pad".into(),
                resolution: (360.0, 640.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        // Sistemas rodando em sequência organizada
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

// Marcadores para o Joystick renderizado na tela
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
    // Câmera do jogo
    commands.spawn(Camera2dBundle::default());

    // --- 3. CORREÇÃO DO CORTE (Largura vs Altura do Frame) ---
    let frame_largura = 313; 
    // SE ELE CONTINUAR CORTADO DA CINTURA PARA BAIXO, aumente este valor abaixo (ex: 400, 420) 
    // até que as pernas apareçam por completo e a grade se alinhe!
    let frame_altura = 313;  

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(frame_largura, frame_altura), 
        4, // 4 colunas
        4, // 4 linhas
        None, 
        None
    );
    let layout_handle = texture_atlas_layouts.add(layout);

    // O nosso herói agora usando o Spritesheet Animado e a sua struct Jogador original
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("meu_personagem_spritesheet.png"),
            // Ajuste o scale se ele ficar muito grande na tela de celular
            transform: Transform::from_xyz(0.0, 100.0, 0.0).with_scale(Vec3::splat(0.5)), 
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

    // --- CONTROLES ESTILO AMONG US VIA SPRITES VIA SEU CÓDIGO ORIGINAL ---

    // Base do Joystick (Círculo/Quadrado de fundo no canto inferior esquerdo)
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
        // O Manete (Bolinha branca do meio que mexe)
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

    // Botão de Pulo (No canto inferior direito)
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

                    if distancia < 60.0 {
                        let delta_x = pos_mouse_bevy.x - pos_base.x;
                        if delta_x > 10.0 {
                            direcao_x = 1.0;
                            offset_manete_x = 20.0;
                        } else if delta_x < -10.0 {
                            direcao_x = -1.0;
                            offset_manete_x = -20.0;
                        }
                    }
                }

                if let Ok(transform_pulo) = q_pulo.get_single() {
                    let pos_pulo = transform_pulo.translation.truncate();
                    if pos_mouse_bevy.distance(pos_pulo) < 40.0 && mouse_input.just_pressed(MouseButton::Left) {
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
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::KeyW) {
        tentou_pular = true;
    }

    for mut jogador in query_jogador.iter_mut() {
        jogador.velocidade_x = direcao_x * 250.0;

        if tentou_pular && jogador.esta_no_chao {
            jogador.velocidade_y = 500.0;
            jogador.esta_no_chao = false;
        }
    }
}

fn mover_jogador(mut query: Query<(&mut Transform, &mut Sprite, &Jogador)>) {
    for (mut transform, mut sprite, jogador) in query.iter_mut() {
        transform.translation.x += jogador.velocidade_x * 0.016;
        if transform.translation.x < -160.0 { transform.translation.x = -160.0; }
        if transform.translation.x > 160.0 { transform.translation.x = 160.0; }

        // Faz o boneco olhar para a direção certa de movimento baseado na sua velocidade_x estável
        if jogador.velocidade_x > 0.0 {
            sprite.flip_x = false;
        } else if jogador.velocidade_x < 0.0 {
            sprite.flip_x = true;
        }
    }
}

// --- 2. CORREÇÃO DA ANIMAÇÃO INFINITA ---
// Agora a animação escuta diretamente a sua struct estável de movimento (`jogador.velocidade_x`).
// Quando ninguém aperta nada, a velocidade zera na hora, forçando o estado parado (Frames 0 a 3).
fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas, &Jogador)>,
) {
    for (mut timer, mut atlas, jogador) in query.iter_mut() {
        timer.0.tick(time.delta());
        
        if timer.0.just_finished() {
            let is_moving = jogador.velocidade_x.abs() > 0.1;

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