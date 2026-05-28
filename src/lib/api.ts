import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type LlamaServerStatus = {
  installed: boolean;
  path: string | null;
  version: string | null;
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
