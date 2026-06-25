# 🎮 Rust Runner: O Desafio Procedural

Jogo de plataforma 2D feito inteiramente em **Rust** com **Bevy 0.14**. O foco do projeto é combinar geração procedural, sistema de estados, HUD, controles móveis e publicação web via **GitHub Pages**.

## Visão Geral

O jogo foi pensado para rodar tanto localmente quanto no navegador. A versão web usa compilação para **WASM** e foi ajustada para funcionar em desktop e também em dispositivos móveis, incluindo toque no iPhone/Safari.

## Recursos

- Mundo infinito com geração procedural.
- Sistema de XP, nível e pontuação.
- Menu inicial com transição para a gameplay.
- Controles por teclado, mouse e toque.
- HUD com pontuação e nível em tempo real.
- Publicação web compatível com GitHub Pages.

## Tecnologias

- Rust 2021
- Bevy 0.14
- WebAssembly para build web
- Trunk para empacotar e publicar a versão web

## Como rodar localmente

Certifique-se de ter o Rust instalado.

```powershell
cargo run --release
```

## Como gerar a versão web

Instale o Trunk se ainda não tiver:

```powershell
cargo install trunk
```

Depois gere os arquivos web:

```powershell
trunk build --release --public-url ./
```

Isso gera os arquivos finais da versão web para publicação no GitHub Pages.

## Como publicar no GitHub Pages

1. Rode o build web com Trunk.
2. Copie os arquivos gerados para a pasta publicada do site, se necessário.
3. Suba o conteúdo atualizado para o repositório do GitHub Pages.

Se o jogo for aberto no iPhone Safari, use uma aba privada na primeira validação para evitar cache antigo.

## Controles

- `A` / `D` ou setas: mover o personagem.
- `Espaço`, `W` ou seta para cima: pular.
- Mouse ou toque: iniciar o jogo no menu.
- Joystick e botão na tela: controles móveis.

## Observações

O projeto está em Rust do começo ao fim. Os arquivos JavaScript e WASM da versão web são apenas artefatos de build, não a lógica principal do jogo.

---

Feito com Rust e Bevy.