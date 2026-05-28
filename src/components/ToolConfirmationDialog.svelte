<script lang="ts">
  import { pendingToolCalls } from '../lib/stores';
  import * as api from '../lib/tauri';
  import type { ToolCall, ToolDefinition } from '../types';

  let toolDefs = $state<ToolDefinition[]>([]);

  async function loadToolDefs() {
    try {
      toolDefs = await api.getToolDefinitions();
    } catch (e) {
      console.error('Failed to load tool definitions:', e);
    }
  }

  function getToolDef(name: string): ToolDefinition | undefined {
    return toolDefs.find(t => t.name === name);
  }

  function formatArgs(args: object): string {
    return JSON.stringify(args, null, 2);
  }

  async function handleConfirm(tc: ToolCall) {
    try {
      await api.confirmToolCall(tc.id, true);
      const result = await api.executeConfirmedTool(tc.id);
      if (result.success) {
        console.log('Tool executed:', result.output);
      } else {
        console.error('Tool failed:', result.error);
      }
    } catch (e) {
      console.error('Failed to confirm/execute tool:', e);
    }
  }

  async function handleCancel(tc: ToolCall) {
    try {
      await api.confirmToolCall(tc.id, false);
    } catch (e) {
      console.error('Failed to cancel tool:', e);
    }
  }

  $effect(() => {
    loadToolDefs();
  });
</script>

{#if $pendingToolCalls.length > 0}
  <div class="tool-confirmation">
    <div class="header">
      <h5>Pending Tool Calls</h5>
      <span class="count">{$pendingToolCalls.length}</span>
    </div>

    <div class="calls">
      {#each $pendingToolCalls as tc}
        <div class="call-item">
          <div class="call-header">
            <span class="tool-name">{tc.tool_name}</span>
          </div>

          <div class="call-description">
            {getToolDef(tc.tool_name)?.description || 'Unknown tool'}
          </div>

          <div class="call-args">
            <pre>{formatArgs(tc.arguments)}</pre>
          </div>

          <div class="call-actions">
            <button class="btn-confirm" onclick={() => handleConfirm(tc)}>
              Execute
            </button>
            <button class="btn-cancel" onclick={() => handleCancel(tc)}>
              Cancel
            </button>
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .tool-confirmation {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: rgba(var(--warning-color-rgb), 0.1);
    border: 1px solid rgba(var(--warning-color-rgb), 0.3);
    border-radius: var(--card-radius);
    margin: var(--spacing-sm);
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .header h5 {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    margin: 0;
  }

  .count {
    padding: 2px 8px;
    background: var(--warning-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }

  .calls {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .call-item {
    padding: var(--spacing-sm);
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .call-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .tool-name {
    font-weight: 500;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
  }

  .call-description {
    color: var(--text-secondary);
    font-size: var(--font-size-xs);
    margin-top: 4px;
  }

  .call-args {
    margin-top: var(--spacing-xs);
    padding: var(--spacing-xs);
    background: var(--bg-input);
    border-radius: var(--radius-sm);
  }

  .call-args pre {
    font-size: var(--font-size-xs);
    color: var(--text-primary);
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .call-actions {
    display: flex;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-sm);
    justify-content: flex-end;
  }

  .btn-confirm {
    padding: 6px 12px;
    border: none;
    background: var(--success-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .btn-cancel {
    padding: 6px 12px;
    border: 1px solid var(--danger-color);
    background: transparent;
    color: var(--danger-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }
</style>