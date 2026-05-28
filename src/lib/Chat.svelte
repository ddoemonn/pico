<script lang="ts">
  import {
    chatStream,
    onChatToken,
    startInference,
    stopInference,
    type ChatMessage,
    type LocalModel,
  } from "./api";

  let { model, onUnload }: { model: LocalModel; onUnload: () => void } = $props();

  let port = $state<number | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let systemPrompt = $state("You are a helpful assistant.");
  let temperature = $state(0.7);
  let topP = $state(0.95);

  let messages = $state<ChatMessage[]>([]);
  let pending = $state("");
  let input = $state("");
  let streaming = $state(false);
  let ttft = $state<number | null>(null);
  let tokPerS = $state<number | null>(null);

  let unlistener: (() => void) | null = null;

  init();

  async function init() {
    try {
      port = await startInference(model.path, 8192);
      const u = await onChatToken((delta) => {
        pending += delta;
      });
      unlistener = u;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function unload() {
    unlistener?.();
    await stopInference().catch(() => {});
    onUnload();
  }

  async function send() {
    if (!input.trim() || streaming) return;
    const userMsg: ChatMessage = { role: "user", content: input.trim() };
    input = "";

    const payload: ChatMessage[] = [
      { role: "system", content: systemPrompt },
      ...messages,
      userMsg,
    ];

    messages = [...messages, userMsg];
    pending = "";
    streaming = true;
    ttft = null;
    tokPerS = null;
    error = null;

    try {
      const metrics = await chatStream(payload, temperature, topP);
      messages = [...messages, { role: "assistant", content: pending }];
      pending = "";
      ttft = metrics.ttft_ms;
      tokPerS = metrics.tok_per_s;
    } catch (e) {
      error = String(e);
    } finally {
      streaming = false;
    }
  }
</script>

<header class="topbar">
  <div class="left">
    <span class="brand">pico</span>
    <span class="model">{model.file}</span>
  </div>
  <button class="back" onclick={unload}>Unload</button>
</header>

{#if loading}
  <div class="loading">Loading model…</div>
{:else if error && !port}
  <div class="loading error">{error}</div>
{:else}
  <main class="chat">
    <div class="messages">
      {#each messages as m}
        <div class="msg {m.role}">
          <div class="role">{m.role}</div>
          <div class="content">{m.content}</div>
        </div>
      {/each}
      {#if streaming || pending}
        <div class="msg assistant">
          <div class="role">assistant</div>
          <div class="content">{pending}<span class="caret"></span></div>
        </div>
      {/if}
    </div>

    <div class="composer">
      <textarea
        bind:value={input}
        placeholder="Message"
        rows="1"
        disabled={streaming}
        onkeydown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            send();
          }
        }}
      ></textarea>
      <button onclick={send} disabled={streaming || !input.trim()}>
        {streaming ? "…" : "Send"}
      </button>
    </div>

    <div class="controls">
      <label>
        system
        <input type="text" bind:value={systemPrompt} />
      </label>
      <label>
        temp <span class="val">{temperature.toFixed(2)}</span>
        <input type="range" min="0" max="2" step="0.05" bind:value={temperature} />
      </label>
      <label>
        top_p <span class="val">{topP.toFixed(2)}</span>
        <input type="range" min="0" max="1" step="0.05" bind:value={topP} />
      </label>
      {#if ttft !== null}
        <span class="metric">TTFT {ttft}ms · {tokPerS?.toFixed(1)} tok/s</span>
      {/if}
    </div>

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </main>
{/if}

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    border-bottom: 1px solid rgba(127, 127, 127, 0.15);
  }
  .left {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  .brand {
    font-weight: 700;
  }
  .model {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    opacity: 0.55;
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
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 60vh;
    opacity: 0.5;
  }
  .chat {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 49px);
    max-width: 760px;
    margin: 0 auto;
    padding: 0 24px;
  }
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 24px 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .msg .role {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.5;
    margin-bottom: 4px;
  }
  .msg .content {
    white-space: pre-wrap;
    font-size: 14px;
    line-height: 1.55;
  }
  .caret {
    display: inline-block;
    width: 6px;
    height: 14px;
    background: currentColor;
    opacity: 0.5;
    animation: blink 1s steps(1) infinite;
    vertical-align: middle;
    margin-left: 2px;
  }
  @keyframes blink {
    50% {
      opacity: 0;
    }
  }
  .composer {
    display: flex;
    gap: 8px;
    padding: 12px 0;
    border-top: 1px solid rgba(127, 127, 127, 0.15);
  }
  textarea {
    flex: 1;
    padding: 10px 12px;
    font-size: 14px;
    font-family: inherit;
    border: 1px solid rgba(127, 127, 127, 0.3);
    border-radius: 8px;
    background: transparent;
    color: inherit;
    resize: none;
  }
  .composer button {
    padding: 0 18px;
    background: transparent;
    border: 1px solid rgba(127, 127, 127, 0.3);
    color: inherit;
    border-radius: 8px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
  }
  .composer button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    padding: 8px 0 16px;
    font-size: 12px;
    align-items: center;
  }
  .controls label {
    display: flex;
    align-items: center;
    gap: 6px;
    opacity: 0.75;
  }
  .controls input[type="text"] {
    width: 240px;
    padding: 4px 8px;
    font-size: 12px;
    border: 1px solid rgba(127, 127, 127, 0.3);
    border-radius: 4px;
    background: transparent;
    color: inherit;
  }
  .val {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 11px;
    min-width: 32px;
  }
  .metric {
    margin-left: auto;
    font-family: ui-monospace, Menlo, monospace;
    color: #2a8;
  }
  .error {
    color: #d33;
    font-size: 12px;
  }
</style>
