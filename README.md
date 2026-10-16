# Nixo Media Player

Media player multiplataforma de alta performance escrito em **Rust**, com interface gráfica 100% nativa renderizada na GPU (`wgpu` + `egui`) e decodificação de vídeo acelerada por hardware via `libmpv`.

## Arquitetura do Projeto

O repositório é estruturado como um **Cargo Workspace** modular:

```text
nixo/
├── Cargo.toml                 # Manifesto do Workspace
├── crates/
│   ├── nixo-core/             # Contratos desacoplados (trait MediaBackend, estados e comandos)
│   ├── nixo-engine/           # Motores de reprodução (libmpv dinâmico + fallback sintético GPU)
│   └── nixo-app/              # Aplicação nativa (janela wgpu/egui, controles OSD e atalhos)
└── scripts/
    └── setup-mpv.ps1          # Script para baixar a libmpv-2.dll automaticamente no Windows
```

## Como Compilar e Executar

### 1. Configurar o `libmpv` (Opcional no build, necessário para vídeos reais)

O Nixo carrega o `libmpv` dinamicamente em tempo de execução, portanto **não exige** bibliotecas `.lib` ou cabeçalhos C instalados para compilar com `cargo build`.

- **Windows (PowerShell)**:
  ```powershell
  ./scripts/setup-mpv.ps1
  ```
  *(Extrai a `libmpv-2.dll` oficial em `vendor/mpv/`, que é detectada automaticamente pelo Nixo).*
- **Linux**: Instale o pacote `libmpv2` / `mpv-libs` da sua distribuição.
- **macOS**: `brew install mpv`

### 2. Executar o Player

```bash
cargo run -p nixo-app
```

Ou passando um arquivo de vídeo diretamente via linha de comando:

```bash
cargo run -p nixo-app -- caminho/para/video.mkv
```

## Atalhos de Teclado

| Atalho | Ação |
| :--- | :--- |
| `Espaço` | Reproduzir / Pausar |
| `Seta Direita` / `Seta Esquerda` | Avançar / Retroceder 5 segundos |
| `Seta Cima` / `Seta Baixo` | Aumentar / Diminuir volume em 5% |
| `M` | Alternar Mudo (Mute) |
| `F` ou `F11` ou `Duplo Clique` | Alternar Tela Cheia (Fullscreen) |
| `Esc` | Sair da Tela Cheia |
| `Ctrl + O` | Abrir seletor de arquivos nativo |

## Licença

Distribuído sob a licença MIT.
