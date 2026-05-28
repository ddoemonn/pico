<script lang="ts">
  import { detectLlamaServer, type LlamaServerStatus } from "$lib/api";
  import Onboarding from "$lib/Onboarding.svelte";

  let status = $state<LlamaServerStatus | null>(null);
  let loading = $state(true);

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
{:else}
  <main class="ready">
    <h1>pico</h1>
    <p class="ok">llama.cpp detected</p>
    <p class="path">{status.path}</p>
    {#if status.version}
      <p class="version">{status.version}</p>
    {/if}
    <p class="next">Model browser coming next.</p>
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
  .ready {
    max-width: 520px;
    margin: 12vh auto 0;
    padding: 0 24px;
    text-align: center;
  }
  h1 {
    font-size: 48px;
    font-weight: 700;
    margin: 0;
    letter-spacing: -0.02em;
  }
  .ok {
    color: #2a8;
    font-size: 16px;
    margin: 12px 0 4px;
  }
  .path,
  .version {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    opacity: 0.6;
    margin: 2px 0;
  }
  .next {
    margin-top: 32px;
    opacity: 0.5;
    font-size: 13px;
  }
</style>
