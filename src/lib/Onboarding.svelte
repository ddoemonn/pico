<script lang="ts">
  import { installLlamaCpp, onInstallOutput } from "./api";

  let { onInstalled }: { onInstalled: () => void } = $props();

  let installing = $state(false);
  let log = $state<string[]>([]);
  let error = $state<string | null>(null);

  async function install() {
    installing = true;
    error = null;
    log = [];

    const unlisteners = await onInstallOutput((line) => {
      log = [...log, line];
    });

    try {
      await installLlamaCpp();
      onInstalled();
    } catch (e) {
      error = String(e);
    } finally {
      installing = false;
      unlisteners.flat().forEach((u) => u());
    }
  }
</script>

<section class="page">
  <div class="hero">
    <h1>pico</h1>
    <p class="tagline">Local AI. 5MB.</p>
  </div>

  <div class="card">
    <p class="body">
      pico runs on <code>llama.cpp</code>. It uses yours, not ours.
    </p>

    {#if !installing}
      <button onclick={install}>Install via Homebrew</button>
      <p class="hint">
        Already installed? Put <code>llama-server</code> on your PATH and relaunch.
      </p>
    {:else}
      <div class="log">
        {#each log as line}<div>{line}</div>{/each}
      </div>
    {/if}

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </div>
</section>

<style>
  .page {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px;
    gap: 40px;
  }
  .hero {
    text-align: center;
  }
  h1 {
    font-size: 64px;
    font-weight: 700;
    margin: 0;
    letter-spacing: -0.03em;
    line-height: 1;
  }
  .tagline {
    font-size: 15px;
    opacity: 0.5;
    margin: 8px 0 0;
    font-family: var(--mono);
  }
  .card {
    width: 100%;
    max-width: 480px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
  .body {
    font-size: 14px;
    margin: 0;
    text-align: center;
    opacity: 0.8;
  }
  code {
    font-family: var(--mono);
    background: var(--surface);
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.9em;
  }
  button {
    padding: 10px 22px;
    font-size: 14px;
    font-family: inherit;
    border: 1px solid var(--line-strong);
    background: transparent;
    color: inherit;
    border-radius: 6px;
    cursor: pointer;
  }
  button:hover {
    background: var(--surface);
  }
  .hint {
    font-size: 11px;
    font-family: var(--mono);
    opacity: 0.45;
    margin: 0;
    text-align: center;
  }
  .log {
    width: 100%;
    max-height: 320px;
    overflow-y: auto;
    font-family: var(--mono);
    font-size: 11px;
    background: var(--surface);
    padding: 12px 14px;
    border-radius: 8px;
    line-height: 1.5;
    white-space: pre;
  }
  .error {
    color: #e55;
    font-size: 12px;
    margin: 0;
    text-align: center;
  }
</style>
