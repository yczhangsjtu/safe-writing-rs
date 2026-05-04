<script lang="ts">
  import { copilotSettings, passages, currentPassageIndex } from '../lib/stores';
  import { listen } from '@tauri-apps/api/event';
  import * as api from '../lib/tauri';
  import Resizable from './Resizable.svelte';

  let {
    width,
    onWidthResize,
    onWidthSave
  }: {
    width: number;
    onWidthResize: (width: number) => void;
    onWidthSave: (width: number) => void;
  } = $props();

  let userInput = $state('');
  let output = $state('');
  let waiting = $state(false);
  let selectedText = $state('');
  let collapsedSystem = $state(false);
  let collapsedBuffers = $state(false);
  let collapsedConversation = $state(false);

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

<div class="copilot-panel" style="width: {width}px;">
  <Resizable width={width} side="left" onResize={onWidthResize} onSave={onWidthSave} />
  <!-- Section 1: System Prompt (blue-ish background) -->
  <div class="section system-section" class:collapsed={collapsedSystem}>
    <div class="section-header clickable" onclick={() => collapsedSystem = !collapsedSystem}>
      <h4>System Prompt</h4>
      <span class="collapse-icon">{collapsedSystem ? '▸' : '▾'}</span>
    </div>
    {#if !collapsedSystem}
      <textarea
        bind:value={$copilotSettings.system_prompt}
        oninput={handleSystemPromptChange}
        rows="3"
        placeholder="You are a helpful writing assistant."
        class="system-input"
      ></textarea>
    {/if}
  </div>

  <!-- Section 2: Buffers (green-ish background) -->
  <div class="section buffers-section" class:collapsed={collapsedBuffers}>
    <div class="section-header">
      <div class="header-left clickable" onclick={() => collapsedBuffers = !collapsedBuffers}>
        <h4>Buffers <span class="hint">#0-#9</span></h4>
        <span class="collapse-icon">{collapsedBuffers ? '▸' : '▾'}</span>
      </div>
      {#if !collapsedBuffers}
        <button class="btn-small" onclick={handleAddBuffer}>
          {#if selectedText}Add Selection{:else}Add Current{/if}
        </button>
      {/if}
    </div>
    {#if !collapsedBuffers}
      <div class="buffer-list">
        {#each $copilotSettings.buffers as buf, i}
          {#if buf}
            <div class="buffer-item">
              <button class="btn-remove" onclick={() => handleRemoveBuffer(i)}>×</button>
              <span class="buffer-info">#{i}: {makeBriefSummary(buf, 25)}</span>
            </div>
          {/if}
        {/each}
        {#if $copilotSettings.buffers.every(b => b === '')}
          <span class="empty-hint">No buffers</span>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Section 3: Conversation (purple-ish background) -->
  <div class="section conversation-section" class:collapsed={collapsedConversation}>
    <div class="section-header">
      <div class="header-left clickable" onclick={() => collapsedConversation = !collapsedConversation}>
        <h4>Conversation</h4>
        <span class="collapse-icon">{collapsedConversation ? '▸' : '▾'}</span>
      </div>
      {#if !collapsedConversation}
        <button class="btn-small" onclick={handleClearHistory}>Clear</button>
      {/if}
    </div>

    {#if !collapsedConversation}
      <div class="messages">
        {#each $copilotSettings.messages as msg, i}
          <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
            <div class="message-header">
              <strong>{msg.role === 'user' ? 'You' : 'AI'}</strong>
              <div class="message-actions">
                {#if msg.role === 'assistant'}
                  <button class="btn-tiny accent" onclick={() => handleInsertFromHistory(i)}>Insert</button>
                {/if}
                <button class="btn-tiny danger" onclick={() => handleDeleteMessage(i)}>×</button>
              </div>
            </div>
            <p class="message-content">{msg.role === 'user' ? msg.display : msg.content}</p>
          </div>
        {/each}

        {#if waiting || output}
          <div class="message assistant streaming">
            <div class="message-header">
              <strong>AI</strong>
              {#if !waiting && output}
                <button class="btn-tiny accent" onclick={handleInsert}>Insert</button>
              {/if}
            </div>
            <p class="message-content">{output}</p>
          </div>
        {/if}
      </div>
    {/if}

    {#if !collapsedConversation}
      <div class="input-area">
        <textarea
          bind:value={userInput}
          placeholder="Ask AI... (Ctrl+Enter)"
          rows="2"
        ></textarea>
        {#if waiting}
          <button class="btn-send danger" onclick={handleStopGeneration}>Stop</button>
        {:else}
          <button class="btn-send" onclick={handleSend}>Send</button>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .copilot-panel {
    background: var(--bg-card);
    border-radius: var(--card-radius);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
    overflow-y: auto;
    overflow: hidden;
    position: relative;
  }

  .section {
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    transition: all 0.2s ease;
  }

  .section.collapsed {
    padding: var(--spacing-md);
  }

  /* Section 1: System Prompt - blue tint */
  .system-section {
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.2);
  }

  /* Section 2: Buffers - green tint */
  .buffers-section {
    background: rgba(22, 163, 74, 0.1);
    border: 1px solid rgba(22, 163, 74, 0.2);
    max-height: 120px;
    overflow-y: auto;
  }

  /* Section 3: Conversation - purple tint */
  .conversation-section {
    background: rgba(124, 58, 237, 0.1);
    border: 1px solid rgba(124, 58, 237, 0.2);
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .conversation-section.collapsed {
    flex: 0 0 auto;
    overflow: visible;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .clickable {
    cursor: pointer;
    user-select: none;
  }

  .clickable:hover h4 {
    color: var(--text-primary);
  }

  .collapse-icon {
    font-size: 12px;
    color: var(--text-muted);
  }

  .section-header h4 {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-secondary);
    margin: 0;
  }

  .hint {
    font-size: 12px;
    color: var(--text-faint);
  }

  .system-input {
    width: 100%;
    padding: var(--spacing-sm);
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    resize: none;
    font-size: var(--font-size-sm);
  }

  .system-input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .btn-small {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .btn-small:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .buffer-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .buffer-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: rgba(22, 163, 74, 0.2);
    border-radius: var(--radius-sm);
  }

  .btn-remove {
    padding: 0 4px;
    border: none;
    background: transparent;
    color: var(--danger-color);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
  }

  .buffer-info {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .empty-hint {
    font-size: var(--font-size-sm);
    color: var(--text-faint);
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    min-height: 0;
  }

  .message {
    padding: var(--spacing-sm);
    border-radius: var(--radius-sm);
    background: rgba(124, 58, 237, 0.15);
  }

  .message.user {
    background: rgba(168, 85, 247, 0.2);
  }

  .message.assistant {
    background: rgba(139, 92, 246, 0.15);
  }

  .message.streaming {
    background: rgba(139, 92, 246, 0.25);
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .message-header strong {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .message-actions {
    display: flex;
    gap: 2px;
  }

  .btn-tiny {
    padding: 2px 6px;
    border: none;
    background: var(--bg-hover);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 12px;
  }

  .btn-tiny.accent {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .btn-tiny.danger {
    background: var(--danger-color);
    color: var(--text-inverse);
  }

  .message-content {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    white-space: pre-wrap;
    line-height: 1.4;
    margin: 4px 0 0 0;
  }

  .input-area {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .input-area textarea {
    width: 100%;
    padding: var(--spacing-sm);
    border: 1px solid rgba(124, 58, 237, 0.3);
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

  .btn-send {
    padding: var(--spacing-sm);
    border: none;
    background: rgba(124, 58, 237, 0.3);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .btn-send:hover {
    background: rgba(124, 58, 237, 0.4);
  }

  .btn-send.danger {
    background: var(--danger-color);
    color: var(--text-inverse);
  }

  .btn-send.danger:hover {
    opacity: 0.9;
  }
</style>