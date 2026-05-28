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

<section class="onboarding">
  <h1>pico</h1>
  <p class="tagline">Local AI. 5MB.</p>
  <p class="body">
    pico runs on top of <code>llama.cpp</code>. We don't bundle it — we use yours.
  </p>

  {#if !installing}
    <button onclick={install}>Install llama.cpp via Homebrew</button>
    <p class="hint">
      Already installed elsewhere? Make sure <code>llama-server</code> is on your PATH and relaunch.
    </p>
  {:else}
    <div class="log">
      {#each log as line}
        <div>{line}</div>
      {/each}
    </div>
  {/if}

  {#if error}
    <p class="error">{error}</p>
  {/if}
</section>

<style>
  .onboarding {
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
  .tagline {
    font-size: 18px;
    opacity: 0.6;
    margin: 4px 0 32px;
  }
  .body {
    font-size: 15px;
    line-height: 1.5;
    margin-bottom: 24px;
  }
  code {
    background: rgba(127, 127, 127, 0.15);
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.92em;
  }
  button {
    padding: 10px 18px;
    font-size: 15px;
    border-radius: 8px;
    border: 1px solid currentColor;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button:hover {
    background: rgba(127, 127, 127, 0.1);
  }
  .hint {
    font-size: 13px;
    opacity: 0.55;
    margin-top: 18px;
  }
  .log {
    margin-top: 16px;
    text-align: left;
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    background: rgba(0, 0, 0, 0.04);
    padding: 12px;
    border-radius: 6px;
    max-height: 280px;
    overflow-y: auto;
    line-height: 1.4;
  }
  .error {
    color: #d33;
    margin-top: 12px;
    font-size: 13px;
  }
</style>
