# Typeless ⚡

A cross-platform desktop voice-input assistant built with **Rust + Tauri 2.x + React**.  
Hold a configurable global hotkey → speak → the transcript is injected into whatever app is focused.

---

## Features

| Feature | Detail |
|---|---|
| 🎙️ Global push-to-talk | Configurable hotkey (default `RightCtrl` on macOS, `F13` elsewhere) |
| 🌐 Multi-provider ASR | OpenAI Whisper (HTTP), WebSocket streaming, or built-in Mock |
| 🤖 LLM post-processing | Optional GPT-compatible refinement to fix ASR errors |
| ⌨️ Text injection | Clipboard-swap + simulated paste (Cmd+V / Ctrl+V) |
| 📖 Smart dictionary | Built-in tech vocab + user vocab + frequency history |
| 🎨 Floating overlay | Transparent frosted-glass capsule with live waveform bars |
| ⚙️ Settings UI | Clean dark-themed React settings page via system-tray |

---

## Prerequisites

| Requirement | Version |
|---|---|
| Rust | ≥ 1.77 |
| Node.js | ≥ 20 |
| Tauri CLI | v2 (`npm i -g @tauri-apps/cli`) |

### Linux additional deps
```bash
sudo apt-get install -y \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev patchelf \
  libasound2-dev libssl-dev pkg-config
```

---

## Quick Start

```bash
# Install frontend deps
npm install

# Run in development mode (hot-reload)
make dev
# or:
npm run dev &
cargo tauri dev
```

---

## Build for Release

```bash
make build
# or:
npm run build && cargo tauri build
```

Artifacts are placed in `src-tauri/target/release/bundle/`.

---

## Configuration

Settings are stored in `~/.config/typeless/settings.json`.  
Open via the system-tray icon → **Settings…**.

### Settings fields

```jsonc
{
  "language": "zh-CN",          // display language
  "hotkey": "RightCtrl",        // global hotkey key name
  "speechApi": {
    "provider": "mock",         // "mock" | "openai_http" | "websocket"
    "endpoint": "https://api.openai.com/v1/audio/transcriptions",
    "apiKey": "sk-...",
    "model": "whisper-1",
    "language": "zh"
  },
  "llm": {
    "enabled": false,
    "endpoint": "https://api.openai.com/v1/chat/completions",
    "apiKey": "sk-...",
    "model": "gpt-4o-mini"
  },
  "dictionary": {
    "techVocabEnabled": true,
    "userVocab": ["MyCustomWord"],
    "maxHistory": 100
  }
}
```

---

## Architecture

```
src-tauri/src/
├── main.rs          – Tauri app bootstrap, state init, hotkey setup
├── commands.rs      – All #[tauri::command] handlers
├── tray.rs          – System-tray menu
├── audio/           – cpal-based microphone capture + WAV export
├── recognition/     – ASR provider trait + OpenAI/WebSocket/Mock impls
├── llm/             – LLM post-processing (OpenAI chat completions)
├── hotkey/          – rdev global hotkey listener
├── injection/       – arboard + rdev clipboard-paste text injection
├── dictionary/      – Tech vocab + user vocab + frequency history
└── config/          – AppSettings serde load/save

src/
├── settings/        – React settings page (tabbed dark UI)
└── overlay/         – React floating overlay (waveform + transcript)
```

---

## Development

```bash
# Rust lint + tests
make lint
make test

# Format Rust code
cargo fmt --manifest-path src-tauri/Cargo.toml

# Frontend lint
npm run lint
```

---

## License

MIT
