# pico

Local AI in 5MB. A native client for llama.cpp.

![size comparison](assets/size.png)

```
macOS installer download size

LM Studio     ████████████████████████████  556 MB
Ollama        ████████                      160 MB
Jan                                          95 MB
pico          ▏                               2 MB
```

Sources: [lmstudio.ai/download](https://lmstudio.ai/download), [Ollama releases](https://github.com/ollama/ollama/releases/latest), [Jan releases](https://github.com/janhq/jan/releases/latest).

## Why this exists

Every local LLM desktop app is an Electron build that ships its own copy of llama.cpp. The result is hundreds of megabytes of duplicated runtime and a Chromium tax for what is, structurally, a chat window and a download button.

pico assumes you already have `llama-server` on your machine (or installs it with one click via Homebrew), then puts a small native window on top. No bundled inference engine. No telemetry. No account.

## Install

Download the `.app` from [releases](https://github.com/ddoemonn/pico/releases), drag it to Applications, and open.

On first launch, pico checks for `llama-server`. If missing, it offers to run `brew install llama.cpp` for you.

## Use

1. **Discover.** Search Hugging Face for GGUF models. Each file shows its quantization, size, and whether it fits in your RAM.
2. **Models.** Click load on a downloaded model. pico spawns `llama-server` in the background and shows live load progress.
3. **Chat.** Stream messages with temperature and top-p controls. TTFT and tokens/sec are visible at the bottom.

When you eject a model, the underlying server process is killed.

## Architecture

- Tauri 2 shell, Rust backend, Svelte 5 frontend
- Backend calls `llama-server`'s OpenAI-compatible HTTP API on a local port
- Models live in `~/Library/Caches/pico/models`
- Bundle is 3.3 MB because the inference engine is not bundled

## What's not in v0.1

No RAG, no MCP, no tool calling, no vision, no speculative decoding UI, no MLX backend yet. These are reasonable to want; they will arrive when they arrive. pico's scope is run a local model and chat with it, well.

Linux and Windows builds are planned for v0.2 via GitHub Actions.

## Build from source

```bash
git clone https://github.com/ddoemonn/pico
cd pico
pnpm install
pnpm tauri build
```

Requires Rust, Node 20+, pnpm.

## License

MIT.
