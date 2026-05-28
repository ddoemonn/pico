<script lang="ts">
  import { detectLlamaServer, type LlamaServerStatus, type LocalModel } from "$lib/api";
  import Onboarding from "$lib/Onboarding.svelte";
  import ModelBrowser from "$lib/ModelBrowser.svelte";
  import Chat from "$lib/Chat.svelte";

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
  <Chat model={activeModel} onUnload={() => (activeModel = null)} />
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
</style>
