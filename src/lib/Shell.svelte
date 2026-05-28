<script lang="ts">
  import { app, type Section } from "./state.svelte";
  import { stopInference } from "./api";

  const sections: { id: Section; label: string }[] = [
    { id: "chat", label: "chat" },
    { id: "discover", label: "discover" },
    { id: "models", label: "models" },
    { id: "settings", label: "settings" },
  ];

  let { children }: { children: any } = $props();

  async function eject() {
    await stopInference().catch(() => {});
    app.activeModel = null;
    app.loadedPort = null;
  }
</script>

<div class="shell">
  <header>
    <span class="brand">pico</span>
    <nav aria-label="Sections">
      {#each sections as s}
        <button
          class="tab"
          class:active={app.section === s.id}
          aria-current={app.section === s.id ? "page" : undefined}
          onclick={() => (app.section = s.id)}
        >
          {s.label}
        </button>
      {/each}
    </nav>
    <div class="model-slot">
      {#if app.activeModel}
        <div class="loaded" title={app.activeModel.path}>
          <span class="dot" aria-hidden="true"></span>
          <span class="name">{app.activeModel.file}</span>
        </div>
        <button class="eject" onclick={eject} aria-label="Unload model" title="Unload">
          ×
        </button>
      {:else if app.modelLoading}
        <span class="loading-pill">loading…</span>
      {:else}
        <button class="empty" onclick={() => (app.section = "models")}>
          no model
        </button>
      {/if}
    </div>
  </header>

  <main>
    {@render children()}
  </main>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    min-height: 0;
  }
  header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 12px 20px;
    border-bottom: 1px solid var(--line);
    gap: 24px;
    flex-shrink: 0;
  }
  .brand {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.02em;
  }
  nav {
    display: flex;
    gap: 2px;
    justify-content: center;
  }
  .tab {
    background: transparent;
    border: none;
    color: var(--fg-dim);
    font-size: 13px;
    padding: 6px 12px;
    border-radius: var(--r);
    letter-spacing: -0.005em;
    transition: color var(--t), background var(--t);
  }
  .tab:hover {
    color: var(--fg);
    background: var(--surface);
  }
  .tab.active {
    color: var(--fg);
    background: var(--surface);
  }
  .model-slot {
    display: flex;
    align-items: center;
    gap: 6px;
    justify-self: end;
    font-family: var(--mono);
    font-size: 12px;
    min-width: 0;
  }
  .loaded {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 10px 5px 8px;
    border: 1px solid var(--accent);
    background: var(--accent-bg);
    border-radius: var(--r);
    min-width: 0;
    max-width: 280px;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
  }
  .name {
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .eject {
    background: transparent;
    border: 1px solid transparent;
    color: var(--fg-mute);
    font-size: 16px;
    line-height: 1;
    padding: 4px 8px;
    border-radius: var(--r);
    transition: color var(--t), background var(--t);
  }
  .eject:hover {
    color: var(--fg);
    background: var(--surface);
  }
  .empty {
    background: transparent;
    border: 1px dashed var(--line-strong);
    color: var(--fg-mute);
    font-family: var(--mono);
    font-size: 11px;
    padding: 5px 12px;
    border-radius: var(--r);
    transition: color var(--t), border-color var(--t);
  }
  .empty:hover {
    color: var(--fg);
    border-color: var(--fg-mute);
  }
  .loading-pill {
    color: var(--fg-mute);
    padding: 5px 12px;
  }
  main {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
</style>
