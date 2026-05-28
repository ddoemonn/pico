export type Theme = "auto" | "light" | "dark";

const KEY = "pico.theme";

function read(): Theme {
  if (typeof localStorage === "undefined") return "auto";
  const v = localStorage.getItem(KEY);
  return v === "light" || v === "dark" ? v : "auto";
}

function apply(t: Theme) {
  if (typeof document === "undefined") return;
  if (t === "auto") document.documentElement.removeAttribute("data-theme");
  else document.documentElement.setAttribute("data-theme", t);
}

class ThemeStore {
  current = $state<Theme>(read());

  set(next: Theme) {
    this.current = next;
    if (typeof localStorage !== "undefined") localStorage.setItem(KEY, next);
    apply(next);
  }

  init() {
    apply(this.current);
  }
}

export const theme = new ThemeStore();
