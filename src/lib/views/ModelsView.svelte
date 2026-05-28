<script lang="ts">
  import { app } from "../state.svelte";
  import {
    formatBytes,
    listLocalModels,
    startInference,
    stopInference,
    type LocalModel,
  } from "../api";

  let local = $state<LocalModel[]>([]);
  let loading = $state(true);
  let busy = $state<string | null>(null);
  let error = $state<string | null>(null);

  refresh();

  async function refresh() {
    loading = true;
    try {
      local = await listLocalModels();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function load(m: LocalModel) {
    busy = m.path;
    error = null;
    app.modelLoading = true;
    try {
      await stopInference().catch(() => {});
      const port = await startInference(m.path, 8192);
      app.activeModel = m;
      app.loadedPort = port;
      app.section = "chat";
    } catch (e) {
      error = String(e);
    } finally {
      busy = null;
      app.modelLoading = false;
    }
  }
</script>

<section class="view">
  <div class="header">
    <h1>models</h1>
    <p class="sub">downloaded to <code>~/Library/Caches/pico/models</code></p>
  </div>

  {#if loading}
    <p class="hint">loading…</p>
  {:else if local.length === 0}
    <div class="empty">
      <p class="hint">no models yet</p>
      <button onclick={() => (app.section = "discover")}>browse discover</button>
    </div>
  {:else}
    <div class="list">
      {#each local as m}
        {@const active = app.activeModel?.path === m.path}
        <div class="row" class:active>
          <div class="info">
            <div class="file">{m.file}</div>
            <div class="meta">
              <span class="repo">{m.repo}</span>
              <span class="size">{formatBytes(m.size)}</span>
            </div>
          </div>
          {#if active}
            <span class="badge">loaded</span>
          {:else}
            <button
              disabled={!!busy}
              onclick={() => load(m)}
            >
              {busy === m.path ? "loading…" : "load"}
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if error}
    <p class="error">{error}</p>
  {/if}
</section>

<style>
  .view {
    flex: 1;
    overflow-y: auto;
    padding: 28px 24px;
    max-width: 760px;
    width: 100%;
    margin: 0 auto;
  }
  .header {
    margin-bottom: 20px;
  }
  h1 {
    font-size: 22px;
    margin: 0;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 12px;
    opacity: 0.5;
  }
  code {
    font-family: var(--mono);
    background: var(--surface);
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.92em;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 48px 0;
  }
  .empty button {
    padding: 8px 16px;
    font-family: inherit;
    font-size: 13px;
    background: transparent;
    border: 1px solid var(--line-strong);
    border-radius: 6px;
    cursor: pointer;
    color: inherit;
  }
  .empty button:hover {
    background: var(--surface);
  }
  .hint {
    font-family: var(--mono);
    font-size: 12px;
    opacity: 0.45;
    margin: 0;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid var(--line);
    border-radius: 8px;
  }
  .row.active {
    border-color: var(--accent);
    background: var(--accent-bg);
  }
  .info {
    flex: 1;
    min-width: 0;
  }
  .file {
    font-family: var(--mono);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    display: flex;
    gap: 12px;
    margin-top: 2px;
    font-family: var(--mono);
    font-size: 11px;
    opacity: 0.55;
  }
  button {
    padding: 6px 14px;
    font-family: inherit;
    font-size: 12px;
    background: transparent;
    border: 1px solid var(--line-strong);
    color: inherit;
    border-radius: 6px;
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    background: var(--surface);
  }
  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .badge {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--accent);
    padding: 4px 10px;
    border: 1px solid var(--accent);
    border-radius: 6px;
  }
  .error {
    color: #e55;
    font-size: 13px;
  }
</style>
