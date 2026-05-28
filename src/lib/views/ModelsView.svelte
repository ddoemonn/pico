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
  import {
    groupShards,
    localFileName,
    localFileSize,
    shardBaseName,
    shardInfo,
    type Group,
  } from "../shard";
  import { downloads } from "../downloads.svelte";

  let local = $state<LocalModel[]>([]);
  let groups = $state<Group<LocalModel>[]>([]);
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

  $effect(() => {
    downloads.completedAt;
    refresh();
  });

  async function refresh() {
    loading = true;
    try {
      local = await listLocalModels();
      groups = groupShards(local, localFileName, localFileSize);
      groups.sort((a, b) => b.totalSize - a.totalSize);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function groupStatus(g: Group<LocalModel>): {
    complete: boolean;
    have: number;
    total: number;
  } {
    const info = shardInfo(g.rep.file);
    if (!info) return { complete: true, have: 1, total: 1 };
    return { complete: g.members.length >= info.total, have: g.members.length, total: info.total };
  }

  async function loadGroup(g: Group<LocalModel>) {
    const m = g.rep;
    const status = groupStatus(g);
    if (!status.complete) {
      error = `incomplete: ${status.have} of ${status.total} parts present`;
      return;
    }
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

</script>

<section class="view">
  <div class="header">
    <h1>models</h1>
    <p class="sub">downloaded to <code>~/Library/Caches/pico/models</code></p>
  </div>

  {#if loading}
    <p class="hint">loading…</p>
  {:else if groups.length === 0}
    <div class="empty">
      <p class="hint">no models yet</p>
      <button onclick={() => (app.section = "discover")}>browse discover</button>
    </div>
  {:else}
    <div class="list">
      {#each groups as g}
        {@const m = g.rep}
        {@const active = app.activeModel?.path === m.path}
        {@const st = groupStatus(g)}
        <div class="row" class:active>
          <div class="info">
            <div class="file">{shardBaseName(m.file)}</div>
            <div class="meta">
              <span class="repo">{m.repo}</span>
              <span class="size">{formatBytes(g.totalSize)}</span>
              {#if g.shardCount > 1}
                <span class="shards">{st.have}/{st.total} parts</span>
              {/if}
              {#if !st.complete}
                <span class="tag-warn">incomplete</span>
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
          {:else if !st.complete}
            <span class="badge-mute" title="missing shards">incomplete</span>
          {:else}
            <button
              disabled={!!busy}
              onclick={() => loadGroup(g)}
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
    font-size: 26px;
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
    padding: 2px 9px;
    border-radius: var(--r-sm);
    font-size: 11px;
  }
  .shards {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--fg-mute);
    padding: 2px 9px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--r-sm);
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
