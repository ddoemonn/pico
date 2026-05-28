<script lang="ts">
  import { app } from "../state.svelte";
  import { chatStream, onChatToken, type ChatMessage } from "../api";

  let systemPrompt = $state("You are a helpful assistant.");
  let temperature = $state(0.7);
  let topP = $state(0.95);
  let showAdvanced = $state(false);

  let messages = $state<ChatMessage[]>([]);
  let pending = $state("");
  let input = $state("");
  let streaming = $state(false);
  let ttft = $state<number | null>(null);
  let tokPerS = $state<number | null>(null);
  let error = $state<string | null>(null);

  onChatToken((delta) => {
    pending += delta;
  });

  async function send() {
    if (!input.trim() || streaming || !app.activeModel) return;
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
      const m = await chatStream(payload, temperature, topP);
      messages = [...messages, { role: "assistant", content: pending }];
      pending = "";
      ttft = m.ttft_ms;
      tokPerS = m.tok_per_s;
    } catch (e) {
      error = String(e);
    } finally {
      streaming = false;
    }
  }

  function clearChat() {
    messages = [];
    pending = "";
    ttft = null;
    tokPerS = null;
  }
</script>

{#if !app.activeModel}
  <div class="empty">
    <p class="hint">no model loaded</p>
    <button onclick={() => (app.section = "models")}>Pick a model</button>
    <button class="ghost" onclick={() => (app.section = "discover")}>
      or browse Hugging Face
    </button>
  </div>
{:else}
  <div class="chat">
    <div class="thread">
      {#if messages.length === 0 && !pending}
        <div class="welcome">
          <p>send a message to start</p>
        </div>
      {/if}
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

    <div class="footer">
      <div class="composer">
        <textarea
          bind:value={input}
          placeholder="message"
          rows="1"
          disabled={streaming}
          onkeydown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              send();
            }
          }}
        ></textarea>
        <button class="send" onclick={send} disabled={streaming || !input.trim()}>
          {streaming ? "…" : "send"}
        </button>
      </div>

      <div class="meta">
        <button class="toggle" onclick={() => (showAdvanced = !showAdvanced)}>
          {showAdvanced ? "−" : "+"} controls
        </button>
        {#if messages.length > 0}
          <button class="toggle" onclick={clearChat}>clear</button>
        {/if}
        {#if ttft !== null}
          <span class="metric">
            ttft {ttft}ms · {tokPerS?.toFixed(1)} tok/s
          </span>
        {/if}
      </div>

      {#if showAdvanced}
        <div class="advanced">
          <label class="row">
            <span class="lbl">system</span>
            <input type="text" bind:value={systemPrompt} />
          </label>
          <label class="row">
            <span class="lbl">temp</span>
            <input type="range" min="0" max="2" step="0.05" bind:value={temperature} />
            <span class="val">{temperature.toFixed(2)}</span>
          </label>
          <label class="row">
            <span class="lbl">top_p</span>
            <input type="range" min="0" max="1" step="0.05" bind:value={topP} />
            <span class="val">{topP.toFixed(2)}</span>
          </label>
        </div>
      {/if}

      {#if error}
        <p class="error">{error}</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 32px;
  }
  .empty .hint {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg-mute);
    margin: 0 0 8px;
  }
  .empty button {
    padding: 9px 18px;
    font-size: 13px;
    background: transparent;
    border: 1px solid var(--line-strong);
    border-radius: var(--r);
    transition: border-color var(--t), background var(--t), color var(--t);
  }
  .empty button:hover {
    border-color: var(--fg-mute);
    background: var(--surface);
  }
  .empty button.ghost {
    border-color: transparent;
    color: var(--fg-dim);
  }
  .empty button.ghost:hover {
    color: var(--fg);
    background: var(--surface);
  }

  .chat {
    flex: 1;
    display: flex;
    flex-direction: column;
    max-width: 820px;
    width: 100%;
    margin: 0 auto;
    padding: 0 28px;
    min-height: 0;
  }
  .thread {
    flex: 1;
    overflow-y: auto;
    padding: 28px 0;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }
  .welcome {
    margin: auto;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg-mute);
  }
  .msg .role {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    color: var(--fg-mute);
    margin-bottom: 5px;
    text-transform: lowercase;
  }
  .msg .content {
    white-space: pre-wrap;
    font-size: 14px;
    line-height: 1.65;
  }
  .caret {
    display: inline-block;
    width: 6px;
    height: 14px;
    background: var(--accent);
    vertical-align: middle;
    margin-left: 3px;
    animation: blink 1s steps(1) infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0;
    }
  }

  .footer {
    border-top: 1px solid var(--line);
    padding: 14px 0 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .composer {
    display: flex;
    gap: 8px;
  }
  textarea {
    flex: 1;
    padding: 10px 12px;
    font-size: 14px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r);
    background: transparent;
    resize: none;
    line-height: 1.5;
    transition: border-color var(--t), background var(--t);
  }
  textarea:hover {
    border-color: var(--fg-mute);
  }
  textarea:focus-visible {
    outline: none;
    border-color: var(--accent);
    background: var(--surface);
  }
  .send {
    padding: 0 20px;
    background: transparent;
    border: 1px solid var(--line-strong);
    border-radius: var(--r);
    font-size: 13px;
    transition: border-color var(--t), background var(--t);
  }
  .send:hover:not(:disabled) {
    border-color: var(--fg);
    background: var(--surface);
  }
  .send:disabled {
    opacity: 0.45;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 14px;
    font-family: var(--mono);
    font-size: 11px;
  }
  .toggle {
    background: transparent;
    border: none;
    color: var(--fg-mute);
    font-family: var(--mono);
    font-size: 11px;
    padding: 3px 6px;
    border-radius: var(--r-sm);
    transition: color var(--t), background var(--t);
  }
  .toggle:hover {
    color: var(--fg);
    background: var(--surface);
  }
  .metric {
    margin-left: auto;
    color: var(--accent);
  }

  .advanced {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 4px 0 0;
  }
  .row {
    display: grid;
    grid-template-columns: 60px 1fr 50px;
    align-items: center;
    gap: 12px;
    font-family: var(--mono);
    font-size: 11px;
  }
  .lbl {
    color: var(--fg-mute);
  }
  .row input[type="text"] {
    padding: 5px 10px;
    font-size: 12px;
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    background: transparent;
    transition: border-color var(--t);
  }
  .row input[type="text"]:hover {
    border-color: var(--line-strong);
  }
  .row input[type="text"]:focus-visible {
    outline: none;
    border-color: var(--accent);
  }
  .row input[type="range"] {
    accent-color: var(--accent);
  }
  .val {
    color: var(--fg-dim);
    text-align: right;
  }
  .error {
    color: var(--danger);
    background: var(--danger-bg);
    padding: 8px 12px;
    border-radius: var(--r);
    font-size: 12px;
    margin: 0;
  }
</style>
