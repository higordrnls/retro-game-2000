use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Retro Game 2000 - Mobile Preview".into(),
                resolution: (360.0, 640.0).into(), 
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (aplicar_gravidade, controle_jogador, mover_jogador).chain())
        .run();
}

#[derive(Component)]
struct Jogador {
    velocidade_x: f32,
    velocidade_y: f32,
    esta_no_chao: bool,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Nosso bloco verde herói
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.0, 0.8, 0.4), 
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 200.0, 0.0), 
            ..default()
        },
        Jogador {
            velocidade_x: 0.0,
            velocidade_y: 0.0,
            esta_no_chao: false,
        },
    ));
}

fn aplicar_gravidade(mut query: Query<(&mut Transform, &mut Jogador)>) {
    let gravidade = -1000.0; 
    let chao_y = -200.0;

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

fn controle_jogador(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Jogador>) {
    for mut jogador in query.iter_mut() {
        let mut direcao_x = 0.0;
        
        // Esquerda (A ou Seta Esquerda)
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direcao_x -= 1.0;
        }
        // Direita (D ou Seta Direita)
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direcao_x += 1.0;
        }
        jogador.velocidade_x = direcao_x * 250.0;

        // PULO COMPLETO: Espaço, W ou Seta para Cima (mapeando ArrowUp)
        let tentou_pular = keyboard_input.just_pressed(KeyCode::Space) 
            || keyboard_input.just_pressed(KeyCode::KeyW) 
            || keyboard_input.just_pressed(KeyCode::ArrowUp);

        if tentou_pular && jogador.esta_no_chao {
            jogador.velocidade_y = 500.0; 
            jogador.esta_no_chao = false; 
        }
    }
}

// O TREM VOLTOU! Função que tinha sumido:
fn mover_jogador(mut query: Query<(&mut Transform, &Jogador)>) {
    for (mut transform, jogador) in query.iter_mut() {
        transform.translation.x += jogador.velocidade_x * 0.016;
        
        // Limites da tela mobile
        if transform.translation.x < -160.0 { transform.translation.x = -160.0; }
        if transform.translation.x > 160.0 { transform.translation.x = 160.0; }
    }
}