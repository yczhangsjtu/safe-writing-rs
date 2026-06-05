<script lang="ts">
  import { session, passages, currentPassageIndex } from '../lib/stores';
  import { listen } from '@tauri-apps/api/event';
  import * as api from '../lib/tauri';
  import { isCommandKey } from '../lib/platform';
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
  let editingId = $state<number | null>(null);
  let editText = $state('');

  // Track which tool cards are expanded
  let expandedTools = $state<Set<string>>(new Set());

  // Listen for streaming events only (content display)
  $effect(() => {
    const unlistenStart = listen('agent-start', () => {
      output = '';
    });

    const unlistenChunk = listen<string>('agent-chunk', (event) => {
      output += event.payload;
    });

    const unlistenDone = listen('agent-done', (event) => {
      // agent-done event is handled by App.svelte's state-changed listener
      // We just clear output and waiting here
      output = '';
      waiting = false;
    });

    return async () => {
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
      await api.sendAgentMessage(prompt, getCurrentPassageContent());
    } catch (e: any) {
      output = `Error: ${e?.message || e}`;
      waiting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (isCommandKey(e) && e.key === 'Enter') {
      e.preventDefault();
      handleSend();
    }
  }

  async function handleStop() {
    await api.abortGeneration();
    waiting = false;
  }

  async function handleConfirmTool(tc: any) {
    try {
      await api.confirmToolCall(tc.id, true);
      await api.executeConfirmedTool(tc.id);
      // State will be updated by emit_state_change -> App.svelte -> session store
    } catch (e) {
      console.error('Tool execution failed:', e);
    }
  }

  async function handleCancelTool(tc: any) {
    try {
      await api.confirmToolCall(tc.id, false);
      // State will be updated by emit_state_change
    } catch (e) {
      console.error('Tool cancellation failed:', e);
    }
  }

  function toggleToolExpand(id: string) {
    expandedTools.update(s => {
      if (s.has(id)) {
        s.delete(id);
      } else {
        s.add(id);
      }
      return s;
    });
  }

  function startEdit(id: number, content: string) {
    editingId = id;
    editText = content;
  }

  async function saveEdit() {
    if (editingId === null) return;
    await api.editSessionMessage(editingId, editText);
    editingId = null;
    editText = '';
  }

  function cancelEdit() {
    editingId = null;
    editText = '';
  }

  async function deleteMessage(id: number) {
    await api.deleteSessionMessage(id);
  }

  // Format tool name for display
  function formatToolName(name: string): string {
    const names: Record<string, string> = {
      'read_passage': '📖 Read Passage',
      'list_passages': '📋 List Passages',
      'continue_writing': '✍️ Continue Writing',
      'insert_text': '➕ Insert Text',
      'replace_text': '🔄 Replace Text',
      'create_character': '👤 Create Character',
      'update_character': '📝 Update Character',
      'delete_character': '🗑️ Delete Character',
      'list_characters': '👥 List Characters',
      'get_character': '🔍 Get Character',
      'create_relationship': '🔗 Create Relationship',
      'update_relationship': '📝 Update Relationship',
      'delete_relationship': '🗑️ Delete Relationship',
      'list_relationships': '📋 List Relationships',
      'query_character_relationships': '🔍 Query Relationships',
      'set_kv': '💾 Set Key-Value',
      'get_kv': '🔍 Get Key-Value',
      'delete_kv': '🗑️ Delete Key-Value',
      'list_kv': '📋 List Key-Values',
      'get_ai_settings': '⚙️ Get AI Settings',
      'update_ai_settings': '📝 Update AI Settings',
    };
    return names[name] || name;
  }

  // Get status badge
  function getStatusBadge(status: string): { text: string; class: string } {
    switch (status) {
      case 'Pending': return { text: '⏳ Pending', class: 'pending' };
      case 'Confirmed': return { text: '✓ Confirmed', class: 'confirmed' };
      case 'Executed': return { text: '✅ Done', class: 'executed' };
      case 'Failed': return { text: '❌ Failed', class: 'failed' };
      case 'Cancelled': return { text: '⊘ Cancelled', class: 'cancelled' };
      default: return { text: status, class: '' };
    }
  }

  // Get all pending tool calls from session (derived from session store)
  function getPendingToolCalls(): any[] {
    return $session.messages
      .filter(m => m.tool_calls)
      .flatMap(m => m.tool_calls!.filter(tc => tc.status === 'Pending'));
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="agent-panel" style="width: {width}px;">
  <Resizable width={width} side="left" onResize={onWidthResize} onSave={onWidthSave} />

  <!-- Messages -->
  <div class="messages-area">
    {#each $session.messages as msg}
      <div class="msg" class:user={msg.role === 'user'} class:agent={msg.role === 'assistant'} class:tool={msg.role === 'tool'}>
        {#if editingId === msg.id}
          <div class="edit-box">
            <textarea bind:value={editText} rows="3"></textarea>
            <div class="edit-btns">
              <button onclick={saveEdit}>Save</button>
              <button onclick={cancelEdit}>Cancel</button>
            </div>
          </div>
        {:else}
          <div class="msg-header">
            <span class="role">{msg.role === 'user' ? 'You' : msg.role === 'assistant' ? 'Agent' : msg.role === 'tool' ? 'Tool Result' : msg.role}</span>
            {#if msg.role === 'user'}
              <button class="btn-icon" onclick={() => startEdit(msg.id, msg.content)}>✎</button>
              <button class="btn-icon" onclick={() => deleteMessage(msg.id)}>×</button>
            {/if}
          </div>

          <!-- Tool calls embedded in message -->
          {#if msg.tool_calls && msg.tool_calls.length > 0}
            <div class="tool-calls-list">
              {#each msg.tool_calls as tc}
                <div class="tool-block" class:expanded={expandedTools.has(tc.id)} class:pending={tc.status === 'Pending'}>
                  <div class="tool-summary" onclick={() => toggleToolExpand(tc.id)}>
                    <span class="tool-icon">🔧</span>
                    <span class="tool-title">{formatToolName(tc.tool_name)}</span>
                    <span class="tool-status {getStatusBadge(tc.status).class}">{getStatusBadge(tc.status).text}</span>
                    <span class="expand-arrow">{expandedTools.has(tc.id) ? '▼' : '▶'}</span>
                  </div>

                  {#if expandedTools.has(tc.id)}
                    <div class="tool-details">
                      <div class="tool-args-section">
                        <span class="detail-label">Arguments:</span>
                        <pre class="tool-args">{JSON.stringify(tc.arguments, null, 2)}</pre>
                      </div>

                      {#if tc.status === 'Pending'}
                        <div class="tool-actions">
                          <button class="btn-confirm" onclick={() => handleConfirmTool(tc)}>✓ Confirm</button>
                          <button class="btn-reject" onclick={() => handleCancelTool(tc)}>✕ Reject</button>
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}

          <!-- Message content (hide if it's just tool call markers) -->
          {#if msg.content && !msg.content.startsWith('<<TOOL_CALL')}
            <div class="msg-content">{msg.content}</div>
          {/if}
        {/if}
      </div>
    {/each}

    <!-- Pending tool calls from getPendingToolCalls() (all Pending status from session) -->
    {#if getPendingToolCalls().length > 0}
      {#each getPendingToolCalls() as tc}
        <div class="msg agent">
          <div class="msg-header">
            <span class="role">Tool Request</span>
          </div>
          <div class="tool-calls-list">
            <div class="tool-block pending expanded">
              <div class="tool-summary" onclick={() => toggleToolExpand(tc.id)}>
                <span class="tool-icon">🔧</span>
                <span class="tool-title">{formatToolName(tc.tool_name)}</span>
                <span class="tool-status pending">⏳ Pending approval</span>
                <span class="expand-arrow">▼</span>
              </div>
              <div class="tool-details">
                <div class="tool-args-section">
                  <span class="detail-label">Arguments:</span>
                  <pre class="tool-args">{JSON.stringify(tc.arguments, null, 2)}</pre>
                </div>
                <div class="tool-actions">
                  <button class="btn-confirm" onclick={() => handleConfirmTool(tc)}>✓ Confirm</button>
                  <button class="btn-reject" onclick={() => handleCancelTool(tc)}>✕ Reject</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      {/each}
    {/if}

    {#if waiting || output}
      <div class="msg agent streaming">
        <div class="msg-header">
          <span class="role">Agent</span>
          {#if waiting && !output}
            <span class="thinking-indicator">Thinking...</span>
          {/if}
        </div>
        {#if output}
          <div class="msg-content">{output}</div>
        {/if}
      </div>
    {/if}

    {#if $session.summary}
      <div class="summary-box">
        <span class="summary-label">Previous summary:</span>
        {$session.summary}
      </div>
    {/if}
  </div>

  <!-- Input -->
  <div class="input-box">
    <textarea
      bind:value={userInput}
      placeholder="Message Agent..."
      rows="2"
      disabled={waiting}
    ></textarea>
    {#if waiting}
      <button class="btn-stop" onclick={handleStop}>Stop</button>
    {:else}
      <button class="btn-send" onclick={handleSend}>Send</button>
    {/if}
  </div>
</div>

<style>
  .agent-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-card);
    border-radius: var(--card-radius);
    overflow: hidden;
    position: relative;
  }

  /* Messages area */
  .messages-area {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .msg {
    padding: var(--spacing-sm);
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
  }

  .msg.user {
    background: rgba(var(--accent-color-rgb), 0.1);
  }

  .msg.agent {
    background: var(--bg-hover);
  }

  .msg.tool {
    background: rgba(var(--success-color-rgb), 0.05);
  }

  .msg.streaming {
    background: rgba(var(--accent-color-rgb), 0.15);
  }

  .msg-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .role {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-weight: 500;
  }

  .thinking-indicator {
    font-size: var(--font-size-xs);
    color: var(--accent-color);
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }

  .btn-icon {
    padding: 2px 6px;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 12px;
  }

  .btn-icon:hover {
    color: var(--text-primary);
  }

  .msg-content {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    white-space: pre-wrap;
    line-height: 1.5;
    margin-top: var(--spacing-sm);
  }

  /* Tool calls styling */
  .tool-calls-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-sm);
  }

  .tool-block {
    background: var(--bg-secondary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color-faint);
    overflow: hidden;
  }

  .tool-block.pending {
    border-color: rgba(var(--warning-color-rgb), 0.3);
    background: rgba(var(--warning-color-rgb), 0.05);
  }

  .tool-block.expanded {
    background: var(--bg-secondary);
  }

  .tool-summary {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    cursor: pointer;
    user-select: none;
  }

  .tool-summary:hover {
    background: var(--bg-hover);
  }

  .tool-icon {
    font-size: var(--font-size-sm);
  }

  .tool-title {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-primary);
    flex: 1;
  }

  .tool-status {
    font-size: var(--font-size-xs);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    background: var(--bg-input);
  }

  .tool-status.pending {
    color: var(--warning-color);
    background: rgba(var(--warning-color-rgb), 0.1);
  }

  .tool-status.confirmed {
    color: var(--accent-color);
  }

  .tool-status.executed {
    color: var(--success-color);
    background: rgba(var(--success-color-rgb), 0.1);
  }

  .tool-status.failed {
    color: var(--danger-color);
    background: rgba(var(--danger-color-rgb), 0.1);
  }

  .tool-status.cancelled {
    color: var(--text-muted);
  }

  .expand-arrow {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .tool-details {
    padding: var(--spacing-sm);
    border-top: 1px solid var(--border-color-faint);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .tool-args-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .detail-label {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-weight: 500;
  }

  .tool-args {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    background: var(--bg-input);
    padding: var(--spacing-sm);
    border-radius: var(--radius-sm);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    max-height: 200px;
    overflow-y: auto;
  }

  .tool-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
  }

  .btn-confirm {
    padding: 6px 16px;
    background: var(--success-color);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
  }

  .btn-confirm:hover {
    opacity: 0.9;
  }

  .btn-reject {
    padding: 6px 16px;
    background: transparent;
    color: var(--danger-color);
    border: 1px solid var(--danger-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
  }

  .btn-reject:hover {
    background: rgba(var(--danger-color-rgb), 0.1);
  }

  /* Summary box */
  .summary-box {
    padding: var(--spacing-sm);
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
  }

  .summary-label {
    font-weight: 500;
    color: var(--text-muted);
  }

  /* Edit box */
  .edit-box {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .edit-box textarea {
    width: 100%;
    padding: var(--spacing-sm);
    background: var(--bg-input);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    resize: vertical;
  }

  .edit-btns {
    display: flex;
    gap: var(--spacing-xs);
    justify-content: flex-end;
  }

  .edit-btns button {
    padding: 4px 12px;
    background: var(--bg-hover);
    border: none;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  /* Input box */
  .input-box {
    padding: var(--spacing-sm);
    background: var(--bg-card);
    border-top: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .input-box textarea {
    width: 100%;
    padding: var(--spacing-sm);
    background: var(--bg-input);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    resize: none;
  }

  .input-box textarea:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .input-box textarea:disabled {
    opacity: 0.5;
  }

  .btn-send {
    padding: var(--spacing-sm);
    background: var(--accent-color);
    color: var(--text-inverse);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
  }

  .btn-send:hover {
    opacity: 0.9;
  }

  .btn-stop {
    padding: var(--spacing-sm);
    background: var(--danger-color);
    color: var(--text-inverse);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
  }
</style>