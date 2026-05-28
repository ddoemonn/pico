<script lang="ts">
  import { detectLlamaServer, type LlamaServerStatus } from "../api";
  import { app } from "../state.svelte";

  let status = $state<LlamaServerStatus | null>(null);

  refresh();

  async function refresh() {
    status = await detectLlamaServer();
  }
</script>

<section class="view">
  <div class="header">
    <h1>settings</h1>
    <p class="sub">pico v0.1</p>
  </div>

  {#if app.system}
    <div class="group">
      <div class="label">machine</div>
      <div class="kv"><span class="k">cpu</span><span class="v">{app.system.cpu}</span></div>
      <div class="kv"><span class="k">ram</span><span class="v">{app.system.ram_gb} GB</span></div>
      <div class="kv"><span class="k">os</span><span class="v">{app.system.os}</span></div>
    </div>
  {/if}

  <div class="group">
    <div class="label">runtime</div>
    {#if status?.installed}
      <div class="kv">
        <span class="k">llama-server</span>
        <span class="v">{status.path}</span>
      </div>
      {#if status.version}
        <div class="kv">
          <span class="k">version</span>
          <span class="v">{status.version}</span>
        </div>
      {/if}
    {:else}
      <p class="warn">llama-server not found on PATH</p>
    {/if}
  </div>

  <div class="group">
    <div class="label">about</div>
    <p class="body">
      pico is a native client for local LLMs. It runs on top of llama.cpp,
      which it does not bundle. Open source, MIT.
    </p>
  </div>
</section>

<style>
  .view {
    flex: 1;
    overflow-y: auto;
    padding: 24px 28px 48px;
    max-width: 820px;
    width: 100%;
    margin: 0 auto;
  }
  .header {
    margin-bottom: 28px;
  }
  h1 {
    font-size: 24px;
    margin: 0;
    font-weight: 600;
    letter-spacing: -0.015em;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--fg-mute);
    font-family: var(--mono);
  }
  .group {
    margin-bottom: 32px;
  }
  .label {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.1em;
    color: var(--fg-mute);
    margin-bottom: 12px;
    text-transform: uppercase;
  }
  .kv {
    display: grid;
    grid-template-columns: 120px 1fr;
    padding: 10px 0;
    border-bottom: 1px solid var(--line);
    font-family: var(--mono);
    font-size: 12px;
    gap: 16px;
  }
  .k {
    color: var(--fg-mute);
  }
  .v {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--fg-dim);
  }
  .body {
    font-size: 13px;
    line-height: 1.65;
    color: var(--fg-dim);
    margin: 0;
    max-width: 560px;
  }
  .warn {
    color: var(--danger);
    background: var(--danger-bg);
    padding: 8px 12px;
    border-radius: var(--r);
    font-family: var(--mono);
    font-size: 12px;
    margin: 0;
    display: inline-block;
  }
</style>
