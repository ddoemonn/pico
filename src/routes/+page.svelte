<script lang="ts">
  import { detectLlamaServer, type LlamaServerStatus, type LocalModel } from "$lib/api";
  import Onboarding from "$lib/Onboarding.svelte";
  import ModelBrowser from "$lib/ModelBrowser.svelte";

  let status = $state<LlamaServerStatus | null>(null);
  let loading = $state(true);
  let activeModel = $state<LocalModel | null>(null);

  async function refresh() {
    loading = true;
    status = await detectLlamaServer();
    loading = false;
  }

  refresh();
</script>

{#if loading}
  <div class="loading">…</div>
{:else if !status?.installed}
  <Onboarding onInstalled={refresh} />
{:else if !activeModel}
  <header class="topbar">
    <span class="brand">pico</span>
    <span class="status">llama.cpp ready</span>
  </header>
  <ModelBrowser onModelReady={(m) => (activeModel = m)} />
{:else}
  <header class="topbar">
    <span class="brand">pico</span>
    <button class="back" onclick={() => (activeModel = null)}>← Models</button>
  </header>
  <main class="ready">
    <h2>{activeModel.file}</h2>
    <p class="path">{activeModel.path}</p>
    <p class="next">Chat coming next.</p>
  </main>
{/if}

<style>
  :global(:root) {
    font-family:
      -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
    color: #1a1a1a;
    background: #fafafa;
    font-size: 15px;
  }
  :global(body) {
    margin: 0;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      color: #eaeaea;
      background: #1a1a1a;
    }
  }
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    opacity: 0.4;
    font-size: 24px;
  }
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    border-bottom: 1px solid rgba(127, 127, 127, 0.15);
  }
  .brand {
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .status {
    font-size: 12px;
    opacity: 0.5;
  }
  .back {
    font-size: 12px;
    background: transparent;
    border: 1px solid rgba(127, 127, 127, 0.3);
    color: inherit;
    border-radius: 6px;
    padding: 4px 10px;
    cursor: pointer;
    font-family: inherit;
  }
  .ready {
    max-width: 720px;
    margin: 48px auto 0;
    padding: 0 24px;
  }
  h2 {
    font-size: 18px;
    margin: 0 0 4px;
  }
  .path {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    opacity: 0.55;
  }
  .next {
    margin-top: 32px;
    opacity: 0.5;
    font-size: 13px;
  }
</style>
