<script lang="ts">
  import { app } from "../state.svelte";
  import {
    formatBytes,
    listLocalModels,
    onLoadLine,
    startInference,
    stopInference,
    type LocalModel,
  } from "../api";

  let local = $state<LocalModel[]>([]);
  let loading = $state(true);
  let busy = $state<string | null>(null);
  let error = $state<string | null>(null);
  let stage = $state<string>("");
  let percent = $state<number | null>(null);

  onLoadLine((l) => {
    if (!busy) return;
    stage = l.line;
    if (l.percent !== null) percent = l.percent;
  });

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
    stage = "spawning…";
    percent = null;
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
      stage = "";
      percent = null;
      app.modelLoading = false;
    }
  }

  function isMmproj(file: string): boolean {
    return file.toLowerCase().startsWith("mmproj");
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
        {@const mmproj = isMmproj(m.file)}
        <div class="row" class:active>
          <div class="info">
            <div class="file">{m.file}</div>
            <div class="meta">
              <span class="repo">{m.repo}</span>
              <span class="size">{formatBytes(m.size)}</span>
              {#if mmproj}
                <span class="tag-warn">vision projector</span>
              {/if}
            </div>
            {#if busy === m.path}
              <div class="progress">
                {#if percent !== null}
                  <div class="bar"><div class="fill" style="width: {percent}%"></div></div>
                  <div class="stage">
                    <span class="pct">{percent}%</span>
                    <span class="line">{stage}</span>
                  </div>
                {:else}
                  <div class="stage">
                    <span class="line">{stage || "starting…"}</span>
                  </div>
                {/if}
              </div>
            {/if}
          </div>
          {#if active}
            <span class="badge">loaded</span>
          {:else if mmproj}
            <span class="badge-mute" title="vision projector, not a model">skip</span>
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
    padding: 24px 28px 48px;
    max-width: 820px;
    width: 100%;
    margin: 0 auto;
  }
  .header {
    margin-bottom: 24px;
  }
  h1 {
    font-size: 24px;
    margin: 0;
    font-weight: 600;
    letter-spacing: -0.015em;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--fg-mute);
  }
  code {
    font-family: var(--mono);
    background: var(--surface);
    padding: 1px 6px;
    border-radius: var(--r-sm);
    font-size: 0.92em;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 64px 0;
  }
  .empty button {
    padding: 9px 18px;
    font-size: 13px;
    background: transparent;
    border: 1px solid var(--line-strong);
    border-radius: var(--r);
    transition: border-color var(--t), background var(--t);
  }
  .empty button:hover {
    border-color: var(--fg-mute);
    background: var(--surface);
  }
  .hint {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg-mute);
    margin: 0;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    transition: border-color var(--t), background var(--t);
  }
  .row:hover {
    border-color: var(--line-strong);
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
    gap: 14px;
    margin-top: 4px;
    font-family: var(--mono);
    font-size: 11px;
    color: var(--fg-mute);
  }
  .row > button {
    padding: 7px 16px;
    font-size: 12px;
    background: transparent;
    border: 1px solid var(--line-strong);
    border-radius: var(--r);
    transition: border-color var(--t), background var(--t), color var(--t);
    flex-shrink: 0;
  }
  .row > button:hover:not(:disabled) {
    border-color: var(--fg);
    background: var(--surface-hover);
  }
  .row > button:disabled {
    opacity: 0.45;
  }
  .badge {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--accent);
    padding: 5px 12px;
    border: 1px solid var(--accent);
    border-radius: var(--r);
    flex-shrink: 0;
  }
  .error {
    color: var(--danger);
    background: var(--danger-bg);
    padding: 8px 12px;
    border-radius: var(--r);
    font-size: 12px;
    margin-top: 14px;
  }
  .tag-warn {
    color: var(--warn);
    background: var(--warn-bg);
    padding: 1px 8px;
    border-radius: var(--r-sm);
    font-size: 10px;
  }
  .badge-mute {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--fg-mute);
    padding: 5px 12px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--r);
    flex-shrink: 0;
  }
  .progress {
    margin-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .bar {
    height: 3px;
    background: var(--surface);
    border-radius: 2px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
    transition: width 200ms ease;
  }
  .stage {
    display: flex;
    gap: 10px;
    font-family: var(--mono);
    font-size: 10px;
    color: var(--fg-mute);
  }
  .pct {
    color: var(--accent);
    min-width: 36px;
  }
  .line {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
