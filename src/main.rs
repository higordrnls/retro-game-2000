use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game 2000 - Mobile UI".into(),
                resolution: (360.0, 640.0).into(), // Proporção de smartphone
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_jogo, setup_ui))
        .add_systems(Update, (aplicar_gravidade, controle_jogador, mover_jogador).chain())
        .run();
}

// Componente para identificar o nosso herói
#[derive(Component)]
struct Jogador {
    velocidade_x: f32,
    velocidade_y: f32,
    esta_no_chao: bool,
}

// Componentes para identificar em qual botão virtual o jogador está clicando/tocando
#[derive(Component)]
struct BotaoEsquerda;

#[derive(Component)]
struct BotaoDireita;

#[derive(Component)]
struct BotaoPulo;

fn setup_jogo(mut commands: Commands) {
    // Inicializa a câmera 2D
    commands.spawn(Camera2dBundle::default());

    // Cria o bloco verde herói
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

// Sistema que desenha a interface mobile (os botões virtuais)
fn setup_ui(mut commands: Commands) {
    // Container principal na parte de baixo da tela
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(120.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Botão Esquerda
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(70.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ..default()
                },
                BotaoEsquerda,
            )).with_children(|p| {
                p.spawn(TextBundle::from_section("<", TextStyle { font_size: 30.0, color: Color::WHITE, ..default() }));
            });

            // Botão Pular (Fica no centro)
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.3, 0.3, 0.8)),
                    ..default()
                },
                BotaoPulo,
            )).with_children(|p| {
                p.spawn(TextBundle::from_section("PULO", TextStyle { font_size: 22.0, color: Color::WHITE, ..default() }));
            });

            // Botão Direita
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(70.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ..default()
                },
                BotaoDireita,
            )).with_children(|p| {
                p.spawn(TextBundle::from_section(">", TextStyle { font_size: 30.0, color: Color::WHITE, ..default() }));
            });
        });
}

fn aplicar_gravidade(mut query: Query<(&mut Transform, &mut Jogador)>) {
    let gravidade = -1000.0; 
    let chao_y = -100.0; // Subi um pouco o chão para não ficar atrás dos botões

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

// Sistema atualizado que escuta TANTO o teclado QUANTO os cliques nos botões da tela!
fn controle_jogador(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query_jogador: Query<&mut Jogador>,
    q_botoes_esq: Query<&Interaction, (With<Button>, With<BotaoEsquerda>)>,
    q_botoes_dir: Query<&Interaction, (With<Button>, With<BotaoDireita>)>,
    q_botoes_pulo: Query<&Interaction, (With<Button>, With<BotaoPulo>)>,
) {
    // Verifica se os botões virtuais estão sendo pressionados com o mouse/toque
    let toque_esquerda = q_botoes_esq.iter().any(|i| *i == Interaction::Pressed);
    let toque_direita = q_botoes_dir.iter().any(|i| *i == Interaction::Pressed);
    let toque_pulo = q_botoes_pulo.iter().any(|i| *i == Interaction::Pressed);

    for mut jogador in query_jogador.iter_mut() {
        let mut direcao_x = 0.0;
        
        // Anda para a esquerda se apertar 'A', Seta ou segurar o botão virtual "<"
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) || toque_esquerda {
            direcao_x -= 1.0;
        }
        // Anda para a direita se apertar 'D', Seta ou segurar o botão virtual ">"
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) || toque_direita {
            direcao_x += 1.0;
        }
        jogador.velocidade_x = direcao_x * 250.0;

        // Pula com Espaço, W, Seta Cima ou clicando no botão azul "PULO"
        let tentou_pular = keyboard_input.just_pressed(KeyCode::Space) 
            || keyboard_input.just_pressed(KeyCode::KeyW) 
            || keyboard_input.just_pressed(KeyCode::ArrowUp)
            || toque_pulo;

        if tentou_pular && jogador.esta_no_chao {
            jogador.velocidade_y = 500.0; 
            jogador.esta_no_chao = false; 
        }
    }
}

fn mover_jogador(mut query: Query<(&mut Transform, &Jogador)>) {
    for (mut transform, jogador) in query.iter_mut() {
        transform.translation.x += jogador.velocidade_x * 0.016;
        
        // Mantém o jogador dentro do limite visível da tela
        if transform.translation.x < -160.0 { transform.translation.x = -160.0; }
        if transform.translation.x > 160.0 { transform.translation.x = 160.0; }
    }
}