<script lang="ts">
  import { copilotSettings, passages, currentPassageIndex } from '../lib/stores';
  import { listen } from '@tauri-apps/api/event';
  import * as api from '../lib/tauri';

  let userInput = $state('');
  let output = $state('');
  let waiting = $state(false);
  let showSettings = $state(false);
  let selectedText = $state('');
  let newFavName = $state('');
  let newFavPrompt = $state('');

  const bufferNames = ['first', 'second', 'third', 'fourth', 'fifth', 'sixth', 'seventh', 'eighth', 'ninth', 'tenth'];

  function makeBriefSummary(text: string, maxLen: number): string {
    if (text.length <= maxLen) return text;
    const half = maxLen / 2;
    return text.slice(0, half) + '...' + text.slice(text.length - half);
  }

  // Listen for selected text changes from Editor
  $effect(() => {
    const unlisten = listen<string>('editor-selection', (event) => {
      selectedText = event.payload;
    });
    return async () => {
      (await unlisten)();
    };
  });

  // Listen for streaming events from backend
  $effect(() => {
    const unlistenUserMessage = listen<{role: string; content: string; display: string}>('copilot-user-message', (event) => {
      copilotSettings.update(s => {
        s.messages = [...s.messages, event.payload];
        return s;
      });
    });

    const unlistenStart = listen('copilot-start', () => {
      output = '';
    });

    const unlistenChunk = listen<string>('copilot-chunk', (event) => {
      output += event.payload;
    });

    const unlistenDone = listen<{role: string; content: string; display: string}>('copilot-done', (event) => {
      copilotSettings.update(s => {
        s.messages = [...s.messages, event.payload];
        return s;
      });
      output = '';
      waiting = false;
    });

    return async () => {
      (await unlistenUserMessage)();
      (await unlistenStart)();
      (await unlistenChunk)();
      (await unlistenDone)();
    };
  });

  function getCurrentPassageContent() {
    return $passages[$currentPassageIndex]?.content || '';
  }

  async function handleSend() {
    if (!userInput.trim() || waiting) return;

    const prompt = userInput.trim();
    userInput = '';
    waiting = true;
    output = '';

    try {
      await api.sendMessage(
        prompt,
        getCurrentPassageContent(),
        $copilotSettings.buffers,
        $copilotSettings.system_prompt,
        $copilotSettings.messages
      );
    } catch (e: any) {
      output = `Error: ${e?.message || e}`;
      waiting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'Enter') {
      e.preventDefault();
      handleSend();
    }
  }

  async function handleInsert() {
    if (!output) return;
    const currentContent = $passages[$currentPassageIndex]?.content || '';
    const newContent = currentContent + '\n\n' + output;
    await api.updatePassageContent($currentPassageIndex, newContent);
    output = '';
  }

  async function handleInsertFromHistory(index: number) {
    const msg = $copilotSettings.messages[index];
    if (!msg || msg.role !== 'assistant') return;
    const currentContent = $passages[$currentPassageIndex]?.content || '';
    const newContent = currentContent + '\n\n' + msg.content;
    await api.updatePassageContent($currentPassageIndex, newContent);
  }

  function handleClearHistory() {
    copilotSettings.update(s => {
      s.messages = [];
      return s;
    });
  }

  // Save copilot settings (will trigger state-changed which updates .ai passage)
  async function saveSettings() {
    await api.saveCopilotSettings($copilotSettings);
  }

  async function handleAddBuffer() {
    const textToAdd = selectedText || getCurrentPassageContent();
    if (!textToAdd) return;

    copilotSettings.update(s => {
      const emptyIndex = s.buffers.findIndex((b: string) => b === '');
      if (emptyIndex >= 0) {
        s.buffers[emptyIndex] = textToAdd.slice(0, 100);
      }
      return s;
    });
    await saveSettings();
  }

  async function handleRemoveBuffer(index: number) {
    copilotSettings.update(s => {
      s.buffers[index] = '';
      return s;
    });
    await saveSettings();
  }

  async function handleStopGeneration() {
    await api.abortGeneration();
    waiting = false;
  }

  function handleDeleteMessage(index: number) {
    copilotSettings.update(s => {
      s.messages = s.messages.filter((_, i) => i !== index);
      return s;
    });
  }

  async function handleAddMessageToFavorites(index: number) {
    const msg = $copilotSettings.messages[index];
    if (!msg || msg.role !== 'user') return;
    const name = makeBriefSummary(msg.display, 20);
    copilotSettings.update(s => {
      s.favorite_prompts = [...s.favorite_prompts, { name, prompt: msg.display }];
      return s;
    });
    await saveSettings();
  }

  async function handleResetSettings() {
    copilotSettings.update(s => {
      s.system_prompt = 'You are a helpful writing assistant.';
      s.buffers = Array(10).fill('');
      s.favorite_prompts = [];
      return s;
    });
    await saveSettings();
    showSettings = false;
  }

  function handleSelectFavoritePrompt(prompt: string) {
    userInput = prompt;
  }

  async function handleAddFavoritePrompt() {
    if (!newFavName.trim() || !newFavPrompt.trim()) return;

    copilotSettings.update(s => {
      s.favorite_prompts = [...s.favorite_prompts, {
        name: newFavName.trim(),
        prompt: newFavPrompt.trim()
      }];
      return s;
    });
    await saveSettings();
    newFavName = '';
    newFavPrompt = '';
  }

  async function handleRemoveFavoritePrompt(index: number) {
    copilotSettings.update(s => {
      s.favorite_prompts = s.favorite_prompts.filter((_, i) => i !== index);
      return s;
    });
    await saveSettings();
  }

  // Save system prompt when it changes
  let saveTimeout: number | null = null;
  function handleSystemPromptChange() {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      saveSettings();
    }, 500);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="copilot-panel">
  <div class="copilot-header">
    <h3>AI Copilot</h3>
    <div class="header-buttons">
      <button onclick={() => showSettings = !showSettings}>
        {#if showSettings}✕{:else}⚙{/if}
      </button>
      <button onclick={handleClearHistory}>Clear</button>
    </div>
  </div>

  {#if showSettings}
    <div class="settings-section">
      <label>
        <span>System Prompt</span>
        <textarea
          bind:value={$copilotSettings.system_prompt}
          oninput={handleSystemPromptChange}
          rows="4"
          placeholder="You are a helpful writing assistant."
        ></textarea>
      </label>

      {#if $copilotSettings.favorite_prompts.length > 0}
        <div class="favorite-prompts-settings">
          <span class="label-text">Favorite Prompts</span>
          <div class="favorite-list">
            {#each $copilotSettings.favorite_prompts as fav, i}
              <div class="favorite-item-settings">
                <span class="fav-name">{fav.name}</span>
                <button class="btn-remove-fav" onclick={() => handleRemoveFavoritePrompt(i)}>×</button>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <div class="add-favorite-row">
        <input
          type="text"
          bind:value={newFavName}
          placeholder="Name..."
          class="fav-name-input"
        />
        <input
          type="text"
          bind:value={newFavPrompt}
          placeholder="Prompt..."
          class="fav-prompt-input"
        />
        <button class="btn-add-fav" onclick={handleAddFavoritePrompt}>+</button>
      </div>

      <button class="btn-reset" onclick={handleResetSettings}>Reset to Default</button>
    </div>
  {/if}

  <div class="copilot-content">
    <div class="messages">
      {#each $copilotSettings.messages as msg, i}
        <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
          <div class="message-header">
            <strong>{msg.role === 'user' ? 'You' : 'AI'}:</strong>
            <div class="message-actions">
              {#if msg.role === 'user'}
                <button class="btn-add-fav-msg" title="Add to favorites" onclick={() => handleAddMessageToFavorites(i)}>+</button>
              {/if}
              {#if msg.role === 'assistant' && i === $copilotSettings.messages.filter(m => m.role === 'assistant').length - 1}
                <button class="btn-insert-small" onclick={() => handleInsertFromHistory(i)}>Insert</button>
              {/if}
              <button class="btn-delete-msg" onclick={() => handleDeleteMessage(i)}>Delete</button>
            </div>
          </div>
          <p>{msg.role === 'user' ? msg.display : msg.content}</p>
        </div>
      {/each}

      {#if waiting || output}
        <div class="message assistant">
          <div class="message-header">
            <strong>AI:</strong>
            {#if !waiting && output}
              <button class="btn-insert-small" onclick={handleInsert}>Insert</button>
            {/if}
          </div>
          <p>{output}</p>
        </div>
      {/if}
    </div>

    <div class="buffers">
      <h4>Buffers <span class="buffer-hint">(#0-#9, &lt;all&gt; for current passage)</span></h4>
      <button onclick={handleAddBuffer}>
        {#if selectedText}Add Selection{:else}Add Current{/if}
      </button>
      {#each $copilotSettings.buffers as buf, i}
        {#if buf}
          <div class="buffer-item">
            <button class="btn-remove-buf" onclick={() => handleRemoveBuffer(i)}>×</button>
            <span class="buffer-info">#{i} ({bufferNames[i]}): {makeBriefSummary(buf, 30)} ({buf.length} chars)</span>
          </div>
        {/if}
      {/each}
    </div>

    <div class="input-area">
      {#if $copilotSettings.favorite_prompts.length > 0}
        <div class="favorite-prompts-quick">
          {#each $copilotSettings.favorite_prompts as fav}
            <button class="btn-fav-quick" onclick={() => handleSelectFavoritePrompt(fav.prompt)}>
              {fav.name}
            </button>
          {/each}
        </div>
      {/if}

      <textarea
        bind:value={userInput}
        placeholder="Ask AI... (Ctrl+Enter to send)"
        rows="3"
      ></textarea>
      {#if waiting}
        <button class="btn-stop" onclick={handleStopGeneration}>Stop</button>
      {:else}
        <button onclick={handleSend}>Send</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .copilot-panel {
    width: var(--copilot-width);
    background: var(--bg-sidebar);
    border-left: 1px solid var(--border-color-faint);
    display: flex;
    flex-direction: column;
  }

  .copilot-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border-color-faint);
    height: 40px;
  }

  .copilot-header h3 {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    font-weight: 500;
  }

  .header-buttons {
    display: flex;
    gap: 4px;
  }

  .copilot-header button {
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .copilot-header button:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .settings-section {
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border-color-faint);
  }

  .settings-section label {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-sm);
  }

  .settings-section label span {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .settings-section textarea {
    width: 100%;
    padding: var(--spacing-sm);
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    resize: none;
    font-size: var(--font-size-sm);
  }

  .btn-reset {
    width: 100%;
    padding: var(--spacing-sm);
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    transition: all 0.15s ease;
  }

  .btn-reset:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .copilot-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .messages {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .message {
    padding: var(--spacing-sm);
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
  }

  .message.user {
    background: rgba(124, 58, 237, 0.1);
  }

  .message.assistant {
    background: rgba(22, 163, 74, 0.1);
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2px;
  }

  .message-header strong {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .message-actions {
    display: flex;
    gap: 2px;
  }

  .btn-add-fav-msg {
    padding: 2px 4px;
    border: none;
    background: var(--success-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }

  .btn-delete-msg {
    padding: 2px 4px;
    border: none;
    background: var(--danger-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }

  .btn-insert-small {
    padding: 2px 6px;
    border: none;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }

  .message p {
    font-size: var(--font-size-sm);
    white-space: pre-wrap;
    color: var(--text-primary);
    line-height: 1.5;
  }

  .buffers {
    border-top: 1px solid var(--border-color-faint);
    padding-top: var(--spacing-md);
  }

  .buffers h4 {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-bottom: var(--spacing-sm);
    font-weight: 500;
  }

  .buffer-hint {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
    margin-left: var(--spacing-sm);
  }

  .buffers button {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .buffers button:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .buffer-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    margin-top: 2px;
  }

  .btn-remove-buf {
    padding: 2px 4px;
    border: none;
    background: transparent;
    color: var(--danger-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }

  .buffer-info {
    flex: 1;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .input-area {
    border-top: 1px solid var(--border-color-faint);
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .input-area textarea {
    width: 100%;
    padding: var(--spacing-sm);
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    resize: none;
    font-size: var(--font-size-sm);
  }

  .input-area textarea:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .input-area button {
    padding: var(--spacing-sm);
    border: none;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    transition: all 0.15s ease;
  }

  .input-area button:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-stop {
    padding: var(--spacing-sm);
    border: none;
    background: var(--danger-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    transition: all 0.15s ease;
  }

  .btn-stop:hover {
    opacity: 0.9;
  }

  .favorite-prompts-quick {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-sm);
  }

  .btn-fav-quick {
    padding: 4px 8px;
    border: 1px solid var(--border-color-faint);
    background: var(--bg-hover);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    transition: all 0.15s ease;
  }

  .btn-fav-quick:hover {
    background: var(--accent-color);
    color: var(--text-inverse);
    border-color: var(--accent-color);
  }

  .favorite-prompts-settings {
    margin-bottom: var(--spacing-sm);
  }

  .label-text {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    display: block;
    margin-bottom: var(--spacing-xs);
  }

  .favorite-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .favorite-item-settings {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 4px 8px;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
  }

  .fav-name {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    flex: 1;
  }

  .btn-remove-fav {
    padding: 2px 4px;
    border: none;
    background: transparent;
    color: var(--danger-color);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }

  .add-favorite-row {
    display: flex;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-sm);
  }

  .fav-name-input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }

  .fav-prompt-input {
    flex: 2;
    padding: 4px 8px;
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }

  .btn-add-fav {
    padding: 4px 8px;
    border: none;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }
</style>