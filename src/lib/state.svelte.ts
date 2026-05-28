import type { LocalModel, SystemInfo } from "./api";
import { systemInfo } from "./api";

export type Section = "chat" | "discover" | "models" | "settings";

class AppState {
  section = $state<Section>("chat");
  activeModel = $state<LocalModel | null>(null);
  loadedPort = $state<number | null>(null);
  modelLoading = $state(false);
  system = $state<SystemInfo | null>(null);

  async init() {
    try {
      this.system = await systemInfo();
    } catch {}
  }
}

export const app = new AppState();
app.init();

export type FitVerdict = "fits" | "tight" | "too-big";

export function fitVerdict(fileBytes: number, ramGb: number | undefined): FitVerdict {
  if (!ramGb) return "fits";
  const ramBytes = ramGb * 1024 ** 3;
  const headroom = 6 * 1024 ** 3;
  const overhead = 1.5 * 1024 ** 3;
  const usable = ramBytes - headroom;
  if (fileBytes + overhead <= usable) return "fits";
  if (fileBytes + overhead <= ramBytes - 3 * 1024 ** 3) return "tight";
  return "too-big";
}
