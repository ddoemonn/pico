<script lang="ts">
  import { detectLlamaServer, type LlamaServerStatus } from "$lib/api";
  import Onboarding from "$lib/Onboarding.svelte";
  import Shell from "$lib/Shell.svelte";
  import ChatView from "$lib/views/ChatView.svelte";
  import DiscoverView from "$lib/views/DiscoverView.svelte";
  import ModelsView from "$lib/views/ModelsView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";
  import { app } from "$lib/state.svelte";

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
  <Shell>
    {#if app.section === "chat"}
      <ChatView />
    {:else if app.section === "discover"}
      <DiscoverView />
    {:else if app.section === "models"}
      <ModelsView />
    {:else if app.section === "settings"}
      <SettingsView />
    {/if}
  </Shell>
{/if}

<style>
  :global(:root) {
    --bg: #fafafa;
    --fg: #1a1a1a;
    --surface: rgba(0, 0, 0, 0.04);
    --surface-hover: rgba(0, 0, 0, 0.06);
    --line: rgba(0, 0, 0, 0.08);
    --line-strong: rgba(0, 0, 0, 0.18);
    --accent: #00b070;
    --accent-bg: rgba(0, 176, 112, 0.08);
    --mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace;

    font-family:
      -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
    color: var(--fg);
    background: var(--bg);
    font-size: 14px;
    -webkit-font-smoothing: antialiased;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #121212;
      --fg: #eaeaea;
      --surface: rgba(255, 255, 255, 0.05);
      --surface-hover: rgba(255, 255, 255, 0.08);
      --line: rgba(255, 255, 255, 0.08);
      --line-strong: rgba(255, 255, 255, 0.2);
      --accent: #4ade80;
      --accent-bg: rgba(74, 222, 128, 0.1);
    }
  }
  :global(body) {
    margin: 0;
  }
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    opacity: 0.4;
  }
</style>
