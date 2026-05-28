import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type LlamaServerStatus = {
  installed: boolean;
  path: string | null;
  version: string | null;
};

export type HfModel = {
  id: string;
  downloads: number;
  likes: number;
};

export type HfFile = {
  path: string;
  size: number;
};

export type DownloadProgress = {
  repo: string;
  file: string;
  downloaded: number;
  total: number;
};

export type LocalModel = {
  repo: string;
  file: string;
  path: string;
  size: number;
};

export function detectLlamaServer(): Promise<LlamaServerStatus> {
  return invoke<LlamaServerStatus>("detect_llama_server");
}

export function installLlamaCpp(): Promise<void> {
  return invoke<void>("install_llama_cpp");
}

export function onInstallOutput(
  cb: (line: string, stream: "stdout" | "stderr") => void,
): Promise<UnlistenFn[]> {
  return Promise.all([
    listen<string>("install:stdout", (e) => cb(e.payload, "stdout")),
    listen<string>("install:stderr", (e) => cb(e.payload, "stderr")),
  ]);
}

export function searchHfModels(query: string): Promise<HfModel[]> {
  return invoke<HfModel[]>("search_hf_models", { query });
}

export function listHfFiles(repo: string): Promise<HfFile[]> {
  return invoke<HfFile[]>("list_hf_files", { repo });
}

export function downloadModel(repo: string, file: string): Promise<string> {
  return invoke<string>("download_model", { repo, file });
}

export function listLocalModels(): Promise<LocalModel[]> {
  return invoke<LocalModel[]>("list_local_models");
}

export function onDownloadProgress(
  cb: (p: DownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download:progress", (e) => cb(e.payload));
}

export type ChatMessage = {
  role: "system" | "user" | "assistant";
  content: string;
};

export type ChatMetrics = {
  ttft_ms: number;
  tokens: number;
  tok_per_s: number;
};

export function startInference(modelPath: string, ctxSize: number): Promise<number> {
  return invoke<number>("start_inference", { modelPath, ctxSize });
}

export function stopInference(): Promise<void> {
  return invoke<void>("stop_inference");
}

export function chatStream(
  messages: ChatMessage[],
  temperature: number,
  topP: number,
): Promise<ChatMetrics> {
  return invoke<ChatMetrics>("chat_stream", {
    messages,
    temperature,
    topP,
  });
}

export function onChatToken(cb: (delta: string) => void): Promise<UnlistenFn> {
  return listen<{ delta: string }>("chat:token", (e) => cb(e.payload.delta));
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 ** 2) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 ** 3) return `${(n / 1024 ** 2).toFixed(1)} MB`;
  return `${(n / 1024 ** 3).toFixed(2)} GB`;
}
