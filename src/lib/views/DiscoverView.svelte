<script lang="ts">
  import {
    downloadModel,
    formatBytes,
    listHfFiles,
    onDownloadProgress,
    searchHfModels,
    type DownloadProgress,
    type HfFile,
    type HfModel,
  } from "../api";
  import { app, fitVerdict, type FitVerdict } from "../state.svelte";

  type Sort = "trending" | "downloads" | "likes" | "lastModified";

  const sortOptions: { id: Sort; label: string }[] = [
    { id: "trending", label: "trending" },
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
  let sort = $state<Sort>("trending");
  let activeTag = $state("all");
  let results = $state<HfModel[]>([]);
  let searching = $state(false);
  let expanded = $state<string | null>(null);
  let files = $state<Record<string, HfFile[]>>({});
  let downloads = $state<Record<string, DownloadProgress>>({});
  let error = $state<string | null>(null);

  onDownloadProgress((p) => {
    downloads[`${p.repo}/${p.file}`] = p;
  });

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
        files[repo] = await listHfFiles(repo);
      } catch (e) {
        error = String(e);
      }
    }
  }

  async function pull(repo: string, file: HfFile) {
    const key = `${repo}/${file.path}`;
    downloads[key] = { repo, file: file.path, downloaded: 0, total: file.size };
    try {
      await downloadModel(repo, file.path);
    } catch (e) {
      error = String(e);
    } finally {
      delete downloads[key];
      downloads = { ...downloads };
    }
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
    <div>
      <h1>discover</h1>
      <p class="sub">
        Hugging Face · GGUF · {app.system?.ram_gb ?? "?"}GB RAM detected
      </p>
    </div>
    <div class="search">
      <input
        bind:value={query}
        placeholder="search models"
        onkeydown={(e) => e.key === "Enter" && search()}
      />
    </div>
  </div>

  <div class="filters">
    <div class="sorts">
      {#each sortOptions as s}
        <button
          class="chip"
          class:active={sort === s.id}
          onclick={() => selectSort(s.id)}
        >
          {s.label}
        </button>
      {/each}
    </div>
    <div class="tags">
      {#each tagPresets as t}
        <button
          class="chip tag"
          class:active={activeTag === t.id}
          onclick={() => selectTag(t.id)}
        >
          {t.label}
        </button>
      {/each}
    </div>
  </div>

  {#if error}
    <p class="error">{error}</p>
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
              {#each files[m.id] as f}
                {@const key = `${m.id}/${f.path}`}
                {@const p = downloads[key]}
                {@const v = fitVerdict(f.size, app.system?.ram_gb)}
                <button
                  class="row file-row"
                  disabled={!!p}
                  onclick={() => pull(m.id, f)}
                >
                  <span class="file">{f.path}</span>
                  <span class="size">{formatBytes(f.size)}</span>
                  <span class="fit fit-{v}">{fitLabel(v)}</span>
                  {#if p}
                    <span class="progress">
                      {p.total
                        ? `${Math.round((p.downloaded / p.total) * 100)}%`
                        : formatBytes(p.downloaded)}
                    </span>
                  {/if}
                </button>
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
    padding: 24px 24px 48px;
    max-width: 880px;
    width: 100%;
    margin: 0 auto;
  }
  .head {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 24px;
    margin-bottom: 14px;
  }
  h1 {
    font-size: 22px;
    margin: 0;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 11px;
    font-family: var(--mono);
    opacity: 0.5;
  }
  .search {
    flex: 1;
    max-width: 320px;
  }
  .search input {
    width: 100%;
    padding: 8px 12px;
    font-size: 13px;
    border: 1px solid var(--line-strong);
    border-radius: 6px;
    background: transparent;
    color: inherit;
    font-family: inherit;
  }
  .search input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .filters {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 16px;
  }
  .sorts,
  .tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .chip {
    background: transparent;
    border: 1px solid var(--line);
    color: inherit;
    font-family: var(--mono);
    font-size: 11px;
    padding: 4px 10px;
    border-radius: 999px;
    cursor: pointer;
    opacity: 0.65;
  }
  .chip:hover {
    opacity: 1;
  }
  .chip.active {
    opacity: 1;
    border-color: var(--fg);
    background: var(--surface);
  }
  .chip.tag {
    opacity: 0.5;
  }
  .chip.tag.active {
    opacity: 1;
    border-color: var(--accent);
    color: var(--accent);
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .model {
    border: 1px solid var(--line);
    border-radius: 8px;
    background: transparent;
  }
  .model.open {
    background: var(--surface);
    border-color: var(--line-strong);
  }
  .card {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 10px 14px;
    cursor: pointer;
    color: inherit;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-family: inherit;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .caret {
    font-family: var(--mono);
    font-size: 10px;
    opacity: 0.5;
    width: 10px;
  }
  .name {
    flex: 1;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .org {
    opacity: 0.5;
  }
  .rep {
    font-weight: 500;
  }
  .pill {
    font-family: var(--mono);
    font-size: 10px;
    padding: 2px 7px;
    border-radius: 4px;
    background: var(--surface-hover);
    opacity: 0.8;
  }
  .meta {
    display: flex;
    gap: 12px;
    padding-left: 18px;
    font-family: var(--mono);
    font-size: 11px;
    opacity: 0.55;
    flex-wrap: wrap;
  }
  .tag-tag {
    color: var(--accent);
    opacity: 0.9;
  }

  .files {
    padding: 4px 14px 10px 32px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .file-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border: 1px dashed var(--line-strong);
    border-radius: 6px;
    background: transparent;
    color: inherit;
    font-family: inherit;
    cursor: pointer;
  }
  .file-row:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .file-row:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .file {
    flex: 1;
    font-family: var(--mono);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .size {
    font-family: var(--mono);
    font-size: 11px;
    opacity: 0.6;
    min-width: 64px;
    text-align: right;
  }
  .fit {
    font-family: var(--mono);
    font-size: 10px;
    padding: 2px 7px;
    border-radius: 4px;
    min-width: 64px;
    text-align: center;
  }
  .fit-fits {
    background: var(--accent-bg);
    color: var(--accent);
  }
  .fit-tight {
    background: rgba(234, 179, 8, 0.12);
    color: #e0a800;
  }
  .fit-too-big {
    background: rgba(229, 85, 85, 0.12);
    color: #e55;
  }
  .progress {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--accent);
    min-width: 48px;
    text-align: right;
  }
  .hint {
    padding: 8px 12px;
    font-family: var(--mono);
    font-size: 11px;
    opacity: 0.45;
    margin: 0;
  }
  .error {
    color: #e55;
    font-size: 12px;
    margin: 0 0 12px;
  }
</style>
