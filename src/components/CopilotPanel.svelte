<script lang="ts">
  import { copilotSettings, passages, currentPassageIndex } from '../lib/stores';
  import * as api from '../lib/tauri';

  let userInput = $state('');
  let output = $state('');
  let waiting = $state(false);
  let showSettings = $state(false);

  function getCurrentPassageContent() {
    return $passages[$currentPassageIndex]?.content || '';
  }

  async function handleSend() {
    if (!userInput.trim()) return;

    waiting = true;
    output = '';

    try {
      await api.sendMessage(
        userInput,
        getCurrentPassageContent(),
        $copilotSettings.buffers,
        $copilotSettings.system_prompt,
        $copilotSettings.messages
      );

      output = 'Response from AI...';
    } catch (e: any) {
      output = `Error: ${e?.message || e}`;
    }

    waiting = false;
    userInput = '';
  }

  function handleInsert() {
    const currentContent = $passages[$currentPassageIndex]?.content || '';
    api.updatePassageContent($currentPassageIndex, currentContent + '\n\n' + output);
  }

  async function handleClearHistory() {
    $copilotSettings.messages = [];
    await api.saveCopilotSettings($copilotSettings);
  }

  function handleAddBuffer() {
    const emptyIndex = $copilotSettings.buffers.findIndex((b: string) => b === '');
    if (emptyIndex >= 0) {
      const content = getCurrentPassageContent();
      $copilotSettings.buffers[emptyIndex] = content.slice(0, 100);
    }
  }

  function handleRemoveBuffer(index: number) {
    $copilotSettings.buffers[index] = '';
  }

  function handleResetSettings() {
    $copilotSettings.system_prompt = 'You are a helpful writing assistant.';
    $copilotSettings.buffers = Array(10).fill('');
    $copilotSettings.favorite_prompts = [];
    $copilotSettings.messages = [];
    api.saveCopilotSettings($copilotSettings);
    showSettings = false;
  }
</script>

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
          rows="4"
          placeholder="You are a helpful writing assistant."
        ></textarea>
      </label>
      <button class="btn-reset" onclick={handleResetSettings}>Reset to Default</button>
    </div>
  {/if}

  <div class="copilot-content">
    <div class="messages">
      {#each $copilotSettings.messages as msg}
        <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
          <strong>{msg.role === 'user' ? 'You' : 'AI'}:</strong>
          <p>{msg.display}</p>
        </div>
      {/each}

      {#if waiting || output}
        <div class="message assistant">
          <strong>AI:</strong>
          <p>{output}</p>
          {#if !waiting && output}
            <button class="btn-insert" onclick={handleInsert}>Insert</button>
          {/if}
        </div>
      {/if}
    </div>

    <div class="buffers">
      <h4>Buffers</h4>
      <button onclick={handleAddBuffer}>Add Current</button>
      {#each $copilotSettings.buffers as buf, i}
        {#if buf}
          <div class="buffer-item">
            <span>#{i}: {buf.slice(0, 30)}...</span>
            <button onclick={() => handleRemoveBuffer(i)}>×</button>
          </div>
        {/if}
      {/each}
    </div>

    <div class="input-area">
      <textarea
        bind:value={userInput}
        placeholder="Ask AI..."
        rows="3"
      ></textarea>
      <button onclick={handleSend} disabled={waiting}>
        {#if waiting}Waiting...{:else}Send{/if}
      </button>
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
    font-size: var(--font-size-xs);
    color: var(--text-faint);
    font-weight: 500;
  }

  .header-buttons {
    display: flex;
    gap: 2px;
  }

  .copilot-header button {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
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
    font-size: var(--font-size-xs);
    color: var(--text-faint);
  }

  .settings-section textarea {
    width: 100%;
    padding: var(--spacing-sm);
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    resize: none;
    font-size: var(--font-size-xs);
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

  .message strong {
    display: block;
    margin-bottom: 2px;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .message p {
    font-size: var(--font-size-xs);
    white-space: pre-wrap;
    color: var(--text-primary);
    line-height: 1.5;
  }

  .btn-insert {
    margin-top: var(--spacing-sm);
    padding: 4px 8px;
    border: none;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    transition: all 0.15s ease;
  }

  .btn-insert:hover {
    opacity: 0.9;
  }

  .buffers {
    border-top: 1px solid var(--border-color-faint);
    padding-top: var(--spacing-md);
  }

  .buffers h4 {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
    margin-bottom: var(--spacing-sm);
    font-weight: 500;
  }

  .buffers button {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
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

  .buffer-item span {
    flex: 1;
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .buffer-item button {
    padding: 2px 4px;
    background: transparent;
    color: var(--danger-color);
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
    font-size: var(--font-size-xs);
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

  .input-area button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>