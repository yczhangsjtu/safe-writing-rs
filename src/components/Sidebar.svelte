<script lang="ts">
  import { files, copilotVisible } from '../lib/stores';
  import * as api from '../lib/tauri';

  let {
    filesProp,
    currentFile,
    isDirtyProp,
    onFileSelect,
    onChangePassword
  }: {
    filesProp: string[];
    currentFile: string | null;
    isDirtyProp: boolean;
    onFileSelect: (filename: string) => void;
    onChangePassword: () => void;
  } = $props();

  let newFilename = $state('');
  let showNewFileInput = $state(false);

  async function handleRefresh() {
    if (isDirtyProp) return;
    try {
      const fileList = await api.listFiles();
      files.set(fileList);
    } catch (e) {
      console.error('Failed to refresh:', e);
    }
  }

  function handleNewFile() {
    if (isDirtyProp) return;
    showNewFileInput = true;
    newFilename = '';
  }

  function handleNewFileSubmit() {
    if (!newFilename.trim()) {
      showNewFileInput = false;
      return;
    }
    if (filesProp.includes(newFilename.trim())) {
      return;
    }
    onFileSelect(newFilename.trim());
    showNewFileInput = false;
    newFilename = '';
  }

  function toggleCopilot() {
    copilotVisible.update(v => !v);
  }
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <button class="btn-icon" title="New File" onclick={handleNewFile} disabled={isDirtyProp}>
      <span class="icon">+</span>
    </button>
    <button class="btn-icon" title="Refresh" onclick={handleRefresh} disabled={isDirtyProp}>
      <span class="icon">↻</span>
    </button>
    <button class="btn-icon" title="AI Copilot" onclick={toggleCopilot}>
      <span class="icon">{#if $copilotVisible}✓{:else}AI{/if}</span>
    </button>
    {#if currentFile}
      <button class="btn-icon" title="Change Password" onclick={onChangePassword}>
        <span class="icon">🔑</span>
      </button>
    {/if}
  </div>

  {#if showNewFileInput}
    <div class="new-file-input">
      <input
        type="text"
        bind:value={newFilename}
        placeholder="New file name..."
        onkeydown={(e) => e.key === 'Enter' && handleNewFileSubmit()}
        onblur={() => showNewFileInput = false}
      />
    </div>
  {/if}

  <div class="file-list">
    {#each filesProp as filename}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="file-item"
        class:selected={filename === currentFile}
        onclick={() => !isDirtyProp && filename !== currentFile && onFileSelect(filename)}
      >
        <span class="file-icon">📄</span>
        <span class="file-name">{filename}</span>
      </div>
    {/each}

    {#if filesProp.length === 0}
      <div class="empty-state">
        <span class="text-muted">No files</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .sidebar {
    width: var(--sidebar-width);
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border-color-faint);
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    display: flex;
    gap: 2px;
    padding: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
  }

  .btn-icon {
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .icon {
    font-size: var(--font-size-xs);
    line-height: 1;
  }

  .btn-icon:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-icon:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .new-file-input {
    padding: var(--spacing-sm) var(--spacing-md);
    border-bottom: 1px solid var(--border-color-faint);
  }

  .new-file-input input {
    width: 100%;
    padding: 4px 8px;
    border: none;
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
  }

  .new-file-input input:focus {
    outline: none;
    box-shadow: inset 0 0 0 1px var(--accent-color);
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-sm);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color 0.15s ease;
    user-select: none;
  }

  .file-item:hover {
    background: var(--bg-hover);
  }

  .file-item.selected {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .file-icon {
    font-size: 12px;
    opacity: 0.6;
  }

  .file-name {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-item.selected .file-name {
    color: var(--text-primary);
  }

  .empty-state {
    padding: var(--spacing-lg);
    text-align: center;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
</style>