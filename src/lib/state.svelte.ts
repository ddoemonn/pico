import type { LocalModel } from "./api";

export type Section = "chat" | "discover" | "models" | "settings";

class AppState {
  section = $state<Section>("chat");
  activeModel = $state<LocalModel | null>(null);
  loadedPort = $state<number | null>(null);
  modelLoading = $state(false);
}

export const app = new AppState();
