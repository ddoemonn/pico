<script lang="ts">
  import {
    formatBytes,
    listHfFiles,
    searchHfModels,
    type HfFile,
    type HfModel,
  } from "../api";
  import { app, fitVerdict, type FitVerdict } from "../state.svelte";
  import { downloads } from "../downloads.svelte";
  import { parseQuant, quantTier } from "../quant";
  import {
    groupShards,
    hfFileName,
    hfFileSize,
    shardBaseName,
    type Group,
  } from "../shard";

  type Sort = "trendingScore" | "downloads" | "likes" | "lastModified";

  const sortOptions: { id: Sort; label: string }[] = [
    { id: "trendingScore", label: "trending" },
    { id: "downloads", label: "popular" },
    { id: "likes", label: "loved" },
    { id: "lastModified", label: "recent" },
  ];

  const tagPresets: { id: string; label: string; tags: string[] }[] = [
    { id: "all", label: "all", tags: [] },
    { id: "llama", label: "llama", tags: ["llama"] },
    { id: "qwen", label: "qwen", tags: ["qwen2"] },
    { id: "gemma", label: "gemma", tags: ["gemma"] },
    { id: "mistral", label: "mistral", tags: ["mistral"] },
    { id: "phi", label: "phi", tags: ["phi"] },
    { id: "code", label: "code", tags: ["code"] },
    { id: "vision", label: "vision", tags: ["multimodal"] },
  ];

  let query = $state("");
  let sort = $state<Sort>("trendingScore");
  let activeTag = $state("all");
  let results = $state<HfModel[]>([]);
  let searching = $state(false);
  let expanded = $state<string | null>(null);
  let files = $state<Record<string, Group<HfFile>[]>>({});
  let error = $state<string | null>(null);

  search();

  async function search() {
    error = null;
    searching = true;
    expanded = null;
    files = {};
    const tags = tagPresets.find((t) => t.id === activeTag)?.tags ?? [];
    try {
      results = await searchHfModels(query, sort, tags);
    } catch (e) {
      error = String(e);
    } finally {
      searching = false;
    }
  }

  function selectSort(s: Sort) {
    sort = s;
    search();
  }

  function selectTag(id: string) {
    activeTag = id;
    search();
  }

  async function toggle(repo: string) {
    if (expanded === repo) {
      expanded = null;
      return;
    }
    expanded = repo;
    if (!files[repo]) {
      try {
        const list = await listHfFiles(repo);
        const grouped = groupShards(list, hfFileName, hfFileSize);
        grouped.sort((a, b) => {
          const qa = parseQuant(a.rep.path)?.quality ?? 0;
          const qb = parseQuant(b.rep.path)?.quality ?? 0;
          if (qa !== qb) return qb - qa;
          return a.totalSize - b.totalSize;
        });
        files[repo] = grouped;
      } catch (e) {
        error = String(e);
      }
    }
  }

  async function pullGroup(repo: string, g: Group<HfFile>) {
    error = null;
    try {
      for (const m of g.members) {
        try {
          await downloads.pull(repo, m.path, m.size, g.key);
        } catch (e) {
          if (String(e).includes("cancelled")) return;
          throw e;
        }
      }
    } catch (e) {
      error = String(e);
    }
  }

  function cancelGroupClick(repo: string, g: Group<HfFile>, e: MouseEvent) {
    e.stopPropagation();
    downloads.cancelGroup(repo, g.key);
  }

  function shortRepo(id: string): { org: string; name: string } {
    const [org, ...rest] = id.split("/");
    return { org, name: rest.join("/") || org };
  }

  function relTime(iso: string | null): string {
    if (!iso) return "";
    const then = new Date(iso).getTime();
    if (Number.isNaN(then)) return "";
    const diff = Date.now() - then;
    const d = Math.floor(diff / 86400000);
    if (d < 1) return "today";
    if (d < 30) return `${d}d ago`;
    const m = Math.floor(d / 30);
    if (m < 12) return `${m}mo ago`;
    return `${Math.floor(m / 12)}y ago`;
  }

  function paramHint(tags: string[]): string | null {
    const sizes = ["0.5b", "1b", "1.5b", "2b", "3b", "4b", "7b", "8b", "13b", "14b", "32b", "70b"];
    for (const t of tags) {
      const lower = t.toLowerCase();
      const m = sizes.find((s) => lower.includes(s));
      if (m) return m.toUpperCase();
    }
    return null;
  }

  function fitLabel(v: FitVerdict): string {
    if (v === "fits") return "fits";
    if (v === "tight") return "tight";
    return "won't fit";
  }
</script>

<section class="view">
  <div class="head">
    <div class="title">
      <h1>discover</h1>
      <p class="sub">
        Hugging Face GGUF · {app.system?.ram_gb ?? "?"}GB RAM
      </p>
    </div>
    <div class="search">
      <input
        bind:value={query}
        placeholder="search models"
        aria-label="Search Hugging Face"
        onkeydown={(e) => e.key === "Enter" && search()}
      />
    </div>
  </div>

  <div class="filters">
    <div class="row-filters" role="tablist" aria-label="Sort">
      {#each sortOptions as s}
        <button
          class="chip"
          class:active={sort === s.id}
          role="tab"
          aria-selected={sort === s.id}
          onclick={() => selectSort(s.id)}
        >
          {s.label}
        </button>
      {/each}
    </div>
    <div class="row-filters" role="tablist" aria-label="Family">
      {#each tagPresets as t}
        <button
          class="chip tag"
          class:active={activeTag === t.id}
          role="tab"
          aria-selected={activeTag === t.id}
          onclick={() => selectTag(t.id)}
        >
          {t.label}
        </button>
      {/each}
    </div>
  </div>

  {#if error}
    <div class="error" role="alert">{error}</div>
  {/if}

  {#if searching && results.length === 0}
    <p class="hint">searching…</p>
  {/if}

  <div class="results">
    {#each results as m}
      {@const r = shortRepo(m.id)}
      {@const size = paramHint(m.tags)}
      <div class="model" class:open={expanded === m.id}>
        <button class="card" onclick={() => toggle(m.id)}>
          <div class="top">
            <span class="caret">{expanded === m.id ? "▾" : "▸"}</span>
            <div class="name">
              <span class="org">{r.org}/</span><span class="rep">{r.name}</span>
            </div>
            {#if size}
              <span class="pill">{size}</span>
            {/if}
          </div>
          <div class="meta">
            <span>↓ {m.downloads.toLocaleString()}</span>
            <span>♡ {m.likes.toLocaleString()}</span>
            {#if m.lastModified}<span>{relTime(m.lastModified)}</span>{/if}
            {#if m.pipeline_tag}<span class="tag-tag">{m.pipeline_tag}</span>{/if}
          </div>
        </button>

        {#if expanded === m.id}
          <div class="files">
            {#if !files[m.id]}
              <div class="row hint">loading files…</div>
            {:else if files[m.id].length === 0}
              <div class="row hint">no gguf files</div>
            {:else}
              {#each files[m.id] as g}
                {@const active = downloads.hasActive(m.id, g.key)}
                {@const v = fitVerdict(g.totalSize, app.system?.ram_gb)}
                {@const q = parseQuant(g.rep.path)}
                {@const prog = downloads.groupProgress(m.id, g.key)}
                <div class="file-row" class:downloading={active}>
                  <button
                    class="file-main"
                    disabled={active}
                    onclick={() => pullGroup(m.id, g)}
                    title={q?.hint ?? ""}
                  >
                    <span class="file">{shardBaseName(g.rep.path)}</span>
                    {#if g.shardCount > 1}
                      <span class="shards">{g.shardCount} parts</span>
                    {/if}
                    {#if q}
                      <span class="quant quant-{quantTier(q)}" title={q.hint}>
                        {q.code}
                      </span>
                    {/if}
                    <span class="size">{formatBytes(g.totalSize)}</span>
                    <span class="fit fit-{v}">{fitLabel(v)}</span>
                  </button>
                  {#if active}
                    <div class="dl">
                      <div class="dl-bar">
                        <div
                          class="dl-fill"
                          style="width: {prog.total ? (prog.done / prog.total) * 100 : 0}%"
                        ></div>
                      </div>
                      <span class="dl-pct">
                        {prog.total
                          ? `${Math.round((prog.done / prog.total) * 100)}%`
                          : formatBytes(prog.done)}
                      </span>
                      <button
                        class="dl-cancel"
                        onclick={(e) => cancelGroupClick(m.id, g, e)}
                        aria-label="Cancel download"
                      >
                        cancel
                      </button>
                    </div>
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    {/each}
    {#if results.length === 0 && !searching}
      <p class="hint">no results</p>
    {/if}
  </div>
</section>

<style>
  .view {
    flex: 1;
    overflow-y: auto;
    padding: 24px 28px 48px;
    max-width: 920px;
    width: 100%;
    margin: 0 auto;
  }
  .head {
    display: grid;
    grid-template-columns: 1fr minmax(220px, 320px);
    align-items: end;
    gap: 24px;
    margin-bottom: 20px;
  }
  .title {
    min-width: 0;
  }
  h1 {
    font-size: 26px;
    margin: 0;
    font-weight: 600;
    letter-spacing: -0.015em;
  }
  .sub {
    margin: 6px 0 0;
    font-size: 12px;
    font-family: var(--mono);
    color: var(--fg-mute);
  }
  .search input {
    width: 100%;
    padding: 10px 14px;
    font-size: 14px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r);
    background: transparent;
    transition: border-color var(--t), background var(--t);
  }
  .search input:hover {
    border-color: var(--fg-mute);
  }
  .search input:focus-visible {
    outline: none;
    border-color: var(--accent);
    background: var(--surface);
  }

  .filters {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 20px;
  }
  .row-filters {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .chip {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--fg-dim);
    font-family: var(--mono);
    font-size: 12px;
    padding: 6px 14px;
    border-radius: var(--r-pill);
    transition: color var(--t), border-color var(--t), background var(--t);
  }
  .chip:hover {
    color: var(--fg);
    border-color: var(--fg-mute);
  }
  .chip.active {
    color: var(--fg);
    border-color: var(--fg);
    background: var(--surface);
  }
  .chip.tag.active {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .model {
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    transition: border-color var(--t), background var(--t);
  }
  .model:hover {
    border-color: var(--line-strong);
  }
  .model.open {
    border-color: var(--line-strong);
    background: var(--surface);
  }
  .card {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    border-radius: var(--r-lg);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .caret {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--fg-mute);
    width: 10px;
    flex-shrink: 0;
  }
  .name {
    flex: 1;
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .org {
    color: var(--fg-mute);
  }
  .rep {
    font-weight: 500;
  }
  .pill {
    font-family: var(--mono);
    font-size: 10px;
    padding: 2px 8px;
    border-radius: var(--r-sm);
    background: var(--surface-hover);
    color: var(--fg-dim);
    flex-shrink: 0;
  }
  .meta {
    display: flex;
    gap: 16px;
    padding-left: 20px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg-mute);
    flex-wrap: wrap;
  }
  .tag-tag {
    color: var(--accent);
  }

  .files {
    padding: 4px 16px 12px 36px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .file-row {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--line);
    border-radius: var(--r);
    background: var(--bg);
    transition: border-color var(--t), background var(--t);
    overflow: hidden;
  }
  .file-row:hover {
    border-color: var(--line-strong);
  }
  .file-row.downloading {
    border-color: var(--accent);
  }
  .file-main {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 12px;
    border: none;
    background: transparent;
    text-align: left;
    width: 100%;
  }
  .file-main:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .file-main:disabled {
    opacity: 0.7;
    cursor: default;
  }
  .quant {
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 500;
    padding: 3px 9px;
    border-radius: var(--r-sm);
    flex-shrink: 0;
    letter-spacing: 0.02em;
  }
  .quant-high {
    background: var(--accent-bg);
    color: var(--accent);
  }
  .quant-mid {
    background: var(--surface-hover);
    color: var(--fg-dim);
  }
  .quant-low {
    background: var(--warn-bg);
    color: var(--warn);
  }
  .shards {
    font-family: var(--mono);
    font-size: 10px;
    color: var(--fg-mute);
    padding: 2px 8px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--r-sm);
    flex-shrink: 0;
  }
  .dl {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: var(--accent-bg);
    border-top: 1px solid var(--line);
  }
  .dl-bar {
    flex: 1;
    height: 3px;
    background: var(--surface);
    border-radius: 2px;
    overflow: hidden;
  }
  .dl-fill {
    height: 100%;
    background: var(--accent);
    transition: width 200ms ease;
  }
  .dl-pct {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--accent);
    min-width: 44px;
    text-align: right;
  }
  .dl-cancel {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--fg-mute);
    background: transparent;
    border: 1px solid var(--line-strong);
    padding: 3px 10px;
    border-radius: var(--r-sm);
    transition: color var(--t), border-color var(--t);
  }
  .dl-cancel:hover {
    color: var(--danger);
    border-color: var(--danger);
  }
  .file {
    flex: 1;
    font-family: var(--mono);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .size {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg-mute);
    min-width: 70px;
    text-align: right;
    flex-shrink: 0;
  }
  .fit {
    font-family: var(--mono);
    font-size: 11px;
    padding: 4px 10px;
    border-radius: var(--r-sm);
    min-width: 72px;
    text-align: center;
    flex-shrink: 0;
    letter-spacing: 0.02em;
  }
  .fit-fits {
    background: var(--accent-bg);
    color: var(--accent);
  }
  .fit-tight {
    background: var(--warn-bg);
    color: var(--warn);
  }
  .fit-too-big {
    background: var(--danger-bg);
    color: var(--danger);
  }
  .progress {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--accent);
    min-width: 48px;
    text-align: right;
  }
  .hint {
    padding: 12px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg-mute);
    margin: 0;
    text-align: center;
  }
  .error {
    color: var(--danger);
    background: var(--danger-bg);
    padding: 8px 12px;
    border-radius: var(--r);
    font-size: 12px;
    margin: 0 0 14px;
  }
</style>
