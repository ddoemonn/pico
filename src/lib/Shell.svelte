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
    <nav>
      {#each sections as s}
        <button
          class="tab"
          class:active={app.section === s.id}
          onclick={() => (app.section = s.id)}
        >
          {s.label}
        </button>
      {/each}
    </nav>
    <div class="model-slot">
      {#if app.activeModel}
        <span class="dot"></span>
        <span class="name">{app.activeModel.file}</span>
        <button class="eject" onclick={eject} title="Unload">×</button>
      {:else if app.modelLoading}
        <span class="loading">loading…</span>
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
  }
  header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 14px 20px;
    border-bottom: 1px solid var(--line);
    gap: 24px;
  }
  .brand {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
  }
  nav {
    display: flex;
    gap: 4px;
    justify-content: center;
  }
  .tab {
    background: transparent;
    border: none;
    color: inherit;
    font-family: inherit;
    font-size: 13px;
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
    opacity: 0.55;
    letter-spacing: -0.005em;
  }
  .tab:hover {
    opacity: 0.85;
  }
  .tab.active {
    opacity: 1;
    background: var(--surface);
  }
  .model-slot {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-self: end;
    font-size: 12px;
    font-family: var(--mono);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }
  .name {
    opacity: 0.8;
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .eject {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    opacity: 0.5;
    padding: 0 4px;
  }
  .eject:hover {
    opacity: 1;
  }
  .empty {
    background: transparent;
    border: 1px dashed var(--line-strong);
    color: inherit;
    font-family: var(--mono);
    font-size: 11px;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
    opacity: 0.6;
  }
  .empty:hover {
    opacity: 1;
  }
  .loading {
    opacity: 0.5;
  }
  main {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
