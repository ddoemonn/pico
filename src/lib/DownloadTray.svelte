<script lang="ts">
  import { downloads } from "./downloads.svelte";
  import { formatBytes } from "./api";
  import { shardBaseName } from "./shard";

  let open = $state(true);

  function shortRepo(repo: string): string {
    const parts = repo.split("/");
    return parts[parts.length - 1] ?? repo;
  }

  function pct(done: number, total: number): number {
    return total > 0 ? Math.round((done / total) * 100) : 0;
  }
</script>

{#if downloads.list().length > 0}
  <aside class="tray" class:open>
    <button class="head" onclick={() => (open = !open)}>
      <span class="dot"></span>
      <span class="title">
        {downloads.list().length} downloading
      </span>
      {#snippet totalsView()}
        {@const t = downloads.totals()}
        <span class="totals">
          {formatBytes(t.done)} / {formatBytes(t.total)}
        </span>
        <span class="big-pct">{pct(t.done, t.total)}%</span>
      {/snippet}
      {@render totalsView()}
      <span class="toggle">{open ? "▾" : "▴"}</span>
    </button>

    {#if open}
      <div class="list">
        {#each downloads.list() as e}
          <div class="item">
            <div class="line">
              <span class="repo">{shortRepo(e.repo)}</span>
              <span class="file">{shardBaseName(e.file)}</span>
            </div>
            <div class="bar">
              <div
                class="fill"
                style="width: {e.total ? (e.downloaded / e.total) * 100 : 0}%"
              ></div>
            </div>
            <div class="line meta">
              <span class="size">
                {formatBytes(e.downloaded)} / {formatBytes(e.total)}
              </span>
              <span class="pct">{pct(e.downloaded, e.total)}%</span>
              <button
                class="cancel"
                onclick={() => downloads.cancel(e.repo, e.file)}
              >
                cancel
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </aside>
{/if}

<style>
  .tray {
    position: fixed;
    bottom: 16px;
    right: 16px;
    width: 360px;
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: var(--r-lg);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
    overflow: hidden;
    z-index: 50;
  }
  .head {
    width: 100%;
    display: grid;
    grid-template-columns: auto 1fr auto auto auto;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    background: var(--accent-bg);
    border: none;
    color: inherit;
    font-family: inherit;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1.2s infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.3;
    }
  }
  .title {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg);
    text-align: left;
  }
  .totals {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--fg-mute);
  }
  .big-pct {
    font-family: var(--mono);
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
    min-width: 44px;
    text-align: right;
  }
  .toggle {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--fg-mute);
  }
  .list {
    display: flex;
    flex-direction: column;
    max-height: 240px;
    overflow-y: auto;
  }
  .item {
    padding: 10px 14px;
    border-top: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--mono);
    font-size: 11px;
  }
  .repo {
    color: var(--fg-dim);
    flex-shrink: 0;
  }
  .file {
    color: var(--fg-mute);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
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
  .meta {
    color: var(--fg-mute);
  }
  .size {
    flex: 1;
  }
  .pct {
    color: var(--accent);
    min-width: 36px;
    text-align: right;
  }
  .cancel {
    font-family: var(--mono);
    font-size: 10px;
    background: transparent;
    color: var(--fg-mute);
    border: 1px solid var(--line-strong);
    padding: 2px 9px;
    border-radius: var(--r-sm);
    transition: color var(--t), border-color var(--t);
  }
  .cancel:hover {
    color: var(--danger);
    border-color: var(--danger);
  }
</style>
