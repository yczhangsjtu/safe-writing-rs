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
    width: 350px;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .copilot-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .copilot-header h3 {
    font-size: 14px;
    color: var(--accent-color);
  }

  .header-buttons {
    display: flex;
    gap: 4px;
  }

  .copilot-header button {
    padding: 6px 12px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .settings-section {
    padding: 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .settings-section label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 8px;
  }

  .settings-section label span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .settings-section textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: 4px;
    resize: none;
  }

  .btn-reset {
    width: 100%;
    padding: 8px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .copilot-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .messages {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .message {
    padding: 8px;
    border-radius: 4px;
    background: var(--bg-input);
  }

  .message.user {
    background: rgba(74, 144, 217, 0.1);
  }

  .message.assistant {
    background: rgba(46, 204, 113, 0.1);
  }

  .message strong {
    display: block;
    margin-bottom: 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .message p {
    font-size: 14px;
    white-space: pre-wrap;
    color: var(--text-primary);
  }

  .btn-insert {
    margin-top: 8px;
    padding: 6px 12px;
    border: none;
    background: var(--accent-color);
    color: white;
    border-radius: 4px;
    cursor: pointer;
  }

  .buffers {
    border-top: 1px solid var(--border-color);
    padding-top: 12px;
  }

  .buffers h4 {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .buffers button {
    padding: 6px 12px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .buffer-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px;
    background: var(--bg-input);
    border-radius: 4px;
    margin-top: 4px;
  }

  .buffer-item span {
    flex: 1;
    font-size: 12px;
    color: var(--text-primary);
  }

  .buffer-item button {
    padding: 2px 6px;
    background: var(--danger-color);
    color: white;
  }

  .input-area {
    border-top: 1px solid var(--border-color);
    padding-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .input-area textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: 4px;
    resize: none;
  }

  .input-area button {
    padding: 8px;
    border: none;
    background: var(--accent-color);
    color: white;
    border-radius: 4px;
    cursor: pointer;
  }

  .input-area button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>