<script lang="ts">
  import { session } from '../lib/stores';
  import * as api from '../lib/tauri';
  import type { SessionMessage } from '../types';

  let editingMessageId = $state<number | null>(null);
  let editContent = $state('');
  let compressSummary = $state('');
  let showCompressDialog = $state(false);

  async function loadSession() {
    try {
      const s = await api.getSession();
      session.set(s);
    } catch (e) {
      console.error('Failed to load session:', e);
    }
  }

  function startEdit(msg: SessionMessage) {
    if (msg.role !== 'user') return;
    editingMessageId = msg.id;
    editContent = msg.content;
  }

  async function saveEdit() {
    if (editingMessageId === null) return;
    try {
      await api.editSessionMessage(editingMessageId, editContent);
      editingMessageId = null;
      editContent = '';
      await loadSession();
    } catch (e) {
      console.error('Failed to edit message:', e);
    }
  }

  function cancelEdit() {
    editingMessageId = null;
    editContent = '';
  }

  async function deleteMessage(id: number) {
    try {
      await api.deleteSessionMessage(id);
      await loadSession();
    } catch (e) {
      console.error('Failed to delete message:', e);
    }
  }

  async function handleCompress() {
    if (!compressSummary.trim()) return;
    try {
      await api.compressSession(compressSummary);
      compressSummary = '';
      showCompressDialog = false;
      await loadSession();
    } catch (e) {
      console.error('Failed to compress session:', e);
    }
  }

  async function handleClear() {
    try {
      await api.clearSession();
      await loadSession();
    } catch (e) {
      console.error('Failed to clear session:', e);
    }
  }

  function formatTimestamp(ts: number): string {
    const date = new Date(ts * 1000);
    return date.toLocaleTimeString();
  }

  function roleLabel(role: string): string {
    switch (role) {
      case 'user': return 'You';
      case 'assistant': return 'AI';
      case 'tool': return 'Tool';
      default: return role;
    }
  }
</script>

<div class="session-panel">
  <div class="header">
    <h4>Session History</h4>
    <div class="actions">
      <button class="btn-small" onclick={() => showCompressDialog = true}>Compress</button>
      <button class="btn-small danger" onclick={handleClear}>Clear</button>
    </div>
  </div>

  {#if $session.summary}
    <div class="summary">
      <strong>Summary:</strong> {$session.summary}
    </div>
  {/if}

  <div class="messages">
    {#each $session.messages as msg}
      <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'} class:tool={msg.role === 'tool'} class:compressed={msg.compressed}>
        <div class="message-header">
          <span class="role">{roleLabel(msg.role)}</span>
          <span class="time">{formatTimestamp(msg.timestamp)}</span>
          {#if msg.role === 'user' && !msg.compressed}
            <button class="btn-tiny" onclick={() => startEdit(msg)}>Edit</button>
          {/if}
          {#if !msg.compressed}
            <button class="btn-tiny danger" onclick={() => deleteMessage(msg.id)}>×</button>
          {/if}
        </div>

        {#if editingMessageId === msg.id}
          <div class="edit-area">
            <textarea bind:value={editContent} rows="3"></textarea>
            <div class="edit-actions">
              <button class="btn-tiny" onclick={saveEdit}>Save</button>
              <button class="btn-tiny" onclick={cancelEdit}>Cancel</button>
            </div>
          </div>
        {:else}
          <div class="message-content">
            {msg.content}
            {#if msg.tool_calls && msg.tool_calls.length > 0}
              <div class="tool-calls">
                {#each msg.tool_calls as tc}
                  <div class="tool-call" class:pending={tc.status === 'Pending'} class:executed={tc.status === 'Executed'}>
                    <span class="tool-name">{tc.tool_name}</span>
                    <span class="tool-status">{tc.status}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {:else}
      <div class="empty">No messages</div>
    {/each}
  </div>

  {#if showCompressDialog}
    <div class="dialog">
      <div class="dialog-content">
        <h5>Compress Session</h5>
        <textarea placeholder="Enter summary for older messages..." bind:value={compressSummary} rows="3"></textarea>
        <div class="dialog-actions">
          <button class="btn-primary" onclick={handleCompress}>Compress</button>
          <button class="btn-secondary" onclick={() => showCompressDialog = false}>Cancel</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .session-panel {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--card-radius);
    height: 100%;
    overflow: hidden;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: var(--spacing-sm);
  }

  .header h4 {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 4px;
  }

  .btn-small {
    padding: 4px 8px;
    border: none;
    background: var(--bg-hover);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }

  .btn-small.danger {
    color: var(--danger-color);
  }

  .summary {
    padding: var(--spacing-sm);
    background: rgba(var(--accent-color-rgb), 0.1);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .message {
    padding: var(--spacing-sm);
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
  }

  .message.user {
    background: rgba(168, 85, 247, 0.15);
  }

  .message.assistant {
    background: rgba(139, 92, 246, 0.15);
  }

  .message.tool {
    background: rgba(22, 163, 74, 0.15);
  }

  .message.compressed {
    opacity: 0.5;
  }

  .message-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .role {
    font-weight: 500;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .time {
    font-size: 11px;
    color: var(--text-faint);
  }

  .btn-tiny {
    padding: 2px 6px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 12px;
  }

  .btn-tiny.danger {
    color: var(--danger-color);
  }

  .message-content {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    white-space: pre-wrap;
    margin-top: 4px;
  }

  .tool-calls {
    margin-top: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tool-call {
    display: flex;
    gap: var(--spacing-xs);
    padding: 4px 8px;
    background: rgba(var(--accent-color-rgb), 0.2);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }

  .tool-call.pending {
    background: rgba(var(--warning-color-rgb), 0.2);
  }

  .tool-call.executed {
    background: rgba(var(--success-color-rgb), 0.2);
  }

  .tool-name {
    font-weight: 500;
  }

  .tool-status {
    color: var(--text-muted);
  }

  .edit-area {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-top: 6px;
  }

  .edit-area textarea {
    width: 100%;
    padding: 6px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    resize: vertical;
  }

  .edit-actions {
    display: flex;
    gap: 4px;
    justify-content: flex-end;
  }

  .empty {
    color: var(--text-faint);
    font-size: var(--font-size-sm);
    text-align: center;
    padding: var(--spacing-md);
  }

  .dialog {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog-content {
    background: var(--bg-modal);
    padding: var(--spacing-lg);
    border-radius: var(--radius-md);
    min-width: 300px;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .dialog-content h5 {
    margin: 0;
    color: var(--text-primary);
  }

  .dialog-content textarea {
    width: 100%;
    padding: var(--spacing-sm);
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    resize: vertical;
  }

  .dialog-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
  }

  .btn-primary {
    padding: 8px 16px;
    border: none;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .btn-secondary {
    padding: 8px 16px;
    border: 1px solid var(--border-color);
    background: transparent;
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
</style>