<script lang="ts">
  import {
    downloadModel,
    formatBytes,
    listHfFiles,
    listLocalModels,
    onDownloadProgress,
    searchHfModels,
    type DownloadProgress,
    type HfFile,
    type HfModel,
    type LocalModel,
  } from "./api";

  let {
    onModelReady,
  }: { onModelReady: (model: LocalModel) => void } = $props();

  let query = $state("qwen2.5 0.5b");
  let results = $state<HfModel[]>([]);
  let searching = $state(false);
  let expanded = $state<string | null>(null);
  let files = $state<Record<string, HfFile[]>>({});
  let downloading = $state<string | null>(null);
  let progress = $state<DownloadProgress | null>(null);
  let local = $state<LocalModel[]>([]);
  let error = $state<string | null>(null);

  onDownloadProgress((p) => {
    progress = p;
  });

  refreshLocal();

  async function refreshLocal() {
    try {
      local = await listLocalModels();
    } catch (e) {
      error = String(e);
    }
  }

  async function search() {
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
    error = null;
    downloading = `${repo}/${file.path}`;
    progress = { repo, file: file.path, downloaded: 0, total: file.size };
    try {
      await downloadModel(repo, file.path);
      await refreshLocal();
    } catch (e) {
      error = String(e);
    } finally {
      downloading = null;
      progress = null;
    }
  }

  search();
</script>

<section class="browser">
  <h2>Models</h2>

  {#if local.length > 0}
    <div class="local">
      <div class="label">Installed</div>
      {#each local as m}
        <button class="row local-row" onclick={() => onModelReady(m)}>
          <span class="repo">{m.repo}</span>
          <span class="file">{m.file}</span>
          <span class="size">{formatBytes(m.size)}</span>
        </button>
      {/each}
    </div>
  {/if}

  <div class="search">
    <input
      bind:value={query}
      placeholder="Search Hugging Face for GGUF"
      onkeydown={(e) => e.key === "Enter" && search()}
    />
    <button onclick={search} disabled={searching}>
      {searching ? "…" : "Search"}
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="results">
    {#each results as m}
      <div class="model">
        <button class="row" onclick={() => toggle(m.id)}>
          <span class="repo">{m.id}</span>
          <span class="meta">↓ {m.downloads}</span>
        </button>
        {#if expanded === m.id}
          <div class="files">
            {#if !files[m.id]}
              <div class="hint">Loading…</div>
            {:else if files[m.id].length === 0}
              <div class="hint">No GGUF files in this repo.</div>
            {:else}
              {#each files[m.id] as f}
                {@const isDownloading = downloading === `${m.id}/${f.path}`}
                <button
                  class="row file-row"
                  disabled={!!downloading}
                  onclick={() => pull(m.id, f)}
                >
                  <span class="file">{f.path}</span>
                  <span class="size">{formatBytes(f.size)}</span>
                  {#if isDownloading && progress}
                    <span class="progress">
                      {progress.total
                        ? `${Math.round((progress.downloaded / progress.total) * 100)}%`
                        : formatBytes(progress.downloaded)}
                    </span>
                  {/if}
                </button>
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</section>

<style>
  .browser {
    max-width: 720px;
    margin: 48px auto 0;
    padding: 0 24px 48px;
  }
  h2 {
    font-size: 22px;
    margin: 0 0 16px;
    font-weight: 600;
  }
  .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.5;
    margin: 16px 0 6px;
  }
  .local {
    margin-bottom: 24px;
  }
  .search {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }
  .search input {
    flex: 1;
    padding: 8px 12px;
    font-size: 14px;
    border: 1px solid rgba(127, 127, 127, 0.3);
    border-radius: 6px;
    background: transparent;
    color: inherit;
  }
  button {
    padding: 8px 14px;
    font-size: 13px;
    border: 1px solid rgba(127, 127, 127, 0.3);
    background: transparent;
    color: inherit;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
  }
  button:hover:not(:disabled) {
    background: rgba(127, 127, 127, 0.08);
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .row {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 12px;
    text-align: left;
    padding: 8px 12px;
    border: 1px solid rgba(127, 127, 127, 0.2);
    border-radius: 6px;
    margin-top: 4px;
  }
  .repo {
    flex: 1;
    font-weight: 500;
    font-size: 13px;
  }
  .meta,
  .size {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 11px;
    opacity: 0.6;
  }
  .file {
    flex: 1;
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
  }
  .files {
    padding: 8px 0 8px 16px;
  }
  .file-row {
    border-style: dashed;
  }
  .progress {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 11px;
    color: #2a8;
  }
  .hint {
    font-size: 12px;
    opacity: 0.5;
    padding: 6px 12px;
  }
  .error {
    color: #d33;
    font-size: 13px;
  }
  .local-row {
    cursor: pointer;
  }
</style>
