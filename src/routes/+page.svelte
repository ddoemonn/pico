<script lang="ts">
  import { detectLlamaServer, type LlamaServerStatus } from "$lib/api";
  import Onboarding from "$lib/Onboarding.svelte";
  import Shell from "$lib/Shell.svelte";
  import ChatView from "$lib/views/ChatView.svelte";
  import DiscoverView from "$lib/views/DiscoverView.svelte";
  import ModelsView from "$lib/views/ModelsView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";
  import { app } from "$lib/state.svelte";
  import { theme } from "$lib/theme.svelte";
  import { downloads } from "$lib/downloads.svelte";
  import DownloadTray from "$lib/DownloadTray.svelte";

  theme.init();
  downloads.init();

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
  <DownloadTray />
{/if}

<style>
  :global(:root),
  :global(:root[data-theme="light"]) {
    --bg: #fafafa;
    --fg: #1a1a1a;
    --fg-dim: rgba(26, 26, 26, 0.65);
    --fg-mute: rgba(26, 26, 26, 0.45);
    --surface: rgba(0, 0, 0, 0.04);
    --surface-hover: rgba(0, 0, 0, 0.07);
    --line: rgba(0, 0, 0, 0.08);
    --line-strong: rgba(0, 0, 0, 0.18);
    --accent: #00a064;
    --accent-strong: #008a55;
    --accent-bg: rgba(0, 160, 100, 0.1);
    --warn: #c88a00;
    --warn-bg: rgba(200, 138, 0, 0.12);
    --danger: #d6443a;
    --danger-bg: rgba(214, 68, 58, 0.1);
  }
  :global(:root) {
    --mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    --t: 120ms ease;
    --r-sm: 4px;
    --r: 6px;
    --r-lg: 8px;
    --r-pill: 999px;

    font-family:
      -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
    color: var(--fg);
    background: var(--bg);
    font-size: 15px;
    -webkit-font-smoothing: antialiased;
    line-height: 1.55;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root:not([data-theme="light"])) {
      --bg: #0f0f10;
      --fg: #ededed;
      --fg-dim: rgba(237, 237, 237, 0.65);
      --fg-mute: rgba(237, 237, 237, 0.4);
      --surface: rgba(255, 255, 255, 0.04);
      --surface-hover: rgba(255, 255, 255, 0.08);
      --line: rgba(255, 255, 255, 0.08);
      --line-strong: rgba(255, 255, 255, 0.18);
      --accent: #4ade80;
      --accent-strong: #6ee7a3;
      --accent-bg: rgba(74, 222, 128, 0.12);
      --warn: #facc15;
      --warn-bg: rgba(250, 204, 21, 0.12);
      --danger: #f87171;
      --danger-bg: rgba(248, 113, 113, 0.12);
    }
  }
  :global(:root[data-theme="dark"]) {
    --bg: #0f0f10;
    --fg: #ededed;
    --fg-dim: rgba(237, 237, 237, 0.65);
    --fg-mute: rgba(237, 237, 237, 0.4);
    --surface: rgba(255, 255, 255, 0.04);
    --surface-hover: rgba(255, 255, 255, 0.08);
    --line: rgba(255, 255, 255, 0.08);
    --line-strong: rgba(255, 255, 255, 0.18);
    --accent: #4ade80;
    --accent-strong: #6ee7a3;
    --accent-bg: rgba(74, 222, 128, 0.12);
    --warn: #facc15;
    --warn-bg: rgba(250, 204, 21, 0.12);
    --danger: #f87171;
    --danger-bg: rgba(248, 113, 113, 0.12);
  }
  :global(body) {
    margin: 0;
    overscroll-behavior: none;
  }
  :global(*) {
    box-sizing: border-box;
  }
  :global(button),
  :global(input),
  :global(textarea) {
    font-family: inherit;
    color: inherit;
  }
  :global(button) {
    cursor: pointer;
  }
  :global(button:disabled) {
    cursor: not-allowed;
  }
  :global(*:focus) {
    outline: none;
  }
  :global(button:focus-visible),
  :global(input:focus-visible),
  :global(textarea:focus-visible),
  :global(a:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: var(--r);
  }
  :global(::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }
  :global(::-webkit-scrollbar-thumb) {
    background: var(--line-strong);
    border-radius: 4px;
  }
  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    opacity: 0.4;
    font-family: var(--mono);
  }
</style>
