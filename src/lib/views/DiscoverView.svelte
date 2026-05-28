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

  let query = $state("qwen2.5 0.5b");
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
    if (!query.trim()) return;
    error = null;
    searching = true;
    try {
      results = await searchHfModels(query);
    } catch (e) {
      error = String(e);
    } finally {
      searching = false;
    }
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
</script>

<section class="view">
  <div class="header">
    <h1>discover</h1>
    <p class="sub">search Hugging Face for GGUF models</p>
  </div>

  <div class="search">
    <input
      bind:value={query}
      placeholder="qwen2.5, llama 3.2, gemma…"
      onkeydown={(e) => e.key === "Enter" && search()}
    />
    <button onclick={search} disabled={searching}>
      {searching ? "…" : "search"}
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="results">
    {#each results as m}
      <div class="model" class:open={expanded === m.id}>
        <button class="row" onclick={() => toggle(m.id)}>
          <span class="caret">{expanded === m.id ? "▾" : "▸"}</span>
          <span class="repo">{m.id}</span>
          <span class="downloads">↓ {m.downloads.toLocaleString()}</span>
        </button>
        {#if expanded === m.id}
          <div class="files">
            {#if !files[m.id]}
              <div class="hint">loading…</div>
            {:else if files[m.id].length === 0}
              <div class="hint">no gguf files</div>
            {:else}
              {#each files[m.id] as f}
                {@const key = `${m.id}/${f.path}`}
                {@const p = downloads[key]}
                <button
                  class="row file-row"
                  disabled={!!p}
                  onclick={() => pull(m.id, f)}
                >
                  <span class="file">{f.path}</span>
                  <span class="size">{formatBytes(f.size)}</span>
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
  .search {
    display: flex;
    gap: 6px;
    margin-bottom: 16px;
  }
  .search input {
    flex: 1;
    padding: 8px 12px;
    font-size: 14px;
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
  button {
    padding: 8px 14px;
    font-size: 13px;
    font-family: inherit;
    border: 1px solid var(--line-strong);
    background: transparent;
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

  .results {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .model {
    border-radius: 6px;
  }
  .model.open {
    background: var(--surface);
  }
  .row {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 10px;
    text-align: left;
    padding: 9px 12px;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    background: transparent;
  }
  .row:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .caret {
    font-family: var(--mono);
    font-size: 10px;
    opacity: 0.5;
    width: 12px;
  }
  .repo {
    flex: 1;
    font-weight: 500;
  }
  .downloads {
    font-family: var(--mono);
    font-size: 11px;
    opacity: 0.5;
  }
  .files {
    padding: 2px 12px 10px 28px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .file-row {
    border: 1px dashed var(--line);
  }
  .file {
    flex: 1;
    font-family: var(--mono);
    font-size: 12px;
  }
  .size {
    font-family: var(--mono);
    font-size: 11px;
    opacity: 0.55;
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
  }
  .error {
    color: #e55;
    font-size: 13px;
  }
</style>
