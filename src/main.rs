use bevy::prelude::*;

// 1. Criamos um Componente para identificar quem é o nosso Jogador
#[derive(Component)]
struct Player {
    speed: f32,
}

fn main() {
    App::new()
        // Configurações da janela padrão do Bevy
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game 2000 🕹️".into(),
                resolution: (600., 400.).into(), // Tamanho perfeito para mobile/retro
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // Adiciona as nossas funções de configuração e lógica
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

// 2. O "Setup" roda apenas UMA vez quando o jogo inicia
fn setup(mut commands: Commands) {
    // Precisamos de uma câmera 2D para conseguir enxergar o jogo
    commands.spawn(Camera2dBundle::default());

    // Mudando a cor do fundo para um bege/branquinho bem retrô e clarinho (#f4f0ea)
    commands.insert_resource(ClearColor(Color::rgb(0.95, 0.94, 0.92)));

    // Nascendo o nosso "Mario" (um bloco pixelado verde-menta anos 2000)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.7, 0.5), // Verde menta limpo
                custom_size: Some(Vec2::new(32.0, 32.0)), // Um bloco de 32x32 pixels
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0), // Começa no centro da tela
            ..default()
        },
        Player { speed: 200.0 }, // Adicionamos o nosso componente Player com velocidade de 200
    ));
}

// 3. O sistema de movimento roda a cada frame (lê o teclado)
fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    // Buscamos o Transform (posição) de quem tem o componente Player
    mut query: Query<(&mut Transform, &Player)>,
) {
    if let Ok((mut transform, player)) = query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        // Setas do teclado ou WASD
        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }

        // Move o boneco baseado no tempo decorrido para não travar
        transform.translation += direction.normalize_or_zero() * player.speed * time.delta_seconds();
    }
}