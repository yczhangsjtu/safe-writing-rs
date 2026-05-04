<script lang="ts">
  import { files, copilotVisible } from '../lib/stores';
  import * as api from '../lib/tauri';

  let {
    filesProp,
    currentFile,
    isDirtyProp,
    onFileSelect
  }: {
    filesProp: string[];
    currentFile: string | null;
    isDirtyProp: boolean;
    onFileSelect: (filename: string) => void;
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
      +
    </button>
    <button class="btn-icon" title="Refresh" onclick={handleRefresh} disabled={isDirtyProp}>
      ↻
    </button>
    <button class="btn-icon" title="AI Copilot" onclick={toggleCopilot}>
      {#if $copilotVisible}AI✓{:else}AI{/if}
    </button>
  </div>

  {#if showNewFileInput}
    <div class="new-file-input">
      <input
        type="text"
        bind:value={newFilename}
        placeholder="filename"
        onkeydown={(e) => e.key === 'Enter' && handleNewFileSubmit()}
        onblur={() => showNewFileInput = false}
      />
    </div>
  {/if}

  <div class="file-list">
    {#each filesProp as filename}
      <button
        class="file-item"
        class:selected={filename === currentFile}
        disabled={isDirtyProp || filename === currentFile}
        onclick={() => onFileSelect(filename)}
      >
        {filename}
      </button>
    {/each}

    {#if filesProp.length === 0}
      <p class="empty-text">No files</p>
    {/if}
  </div>
</div>

<style>
  .sidebar {
    width: 200px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    display: flex;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
  }

  .btn-icon {
    width: 32px;
    height: 32px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
  }

  .btn-icon:hover:not(:disabled) {
    background: var(--bg-button-hover);
  }

  .btn-icon:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .new-file-input {
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
  }

  .new-file-input input {
    width: 100%;
    padding: 8px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: 4px;
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-item {
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    border-radius: 4px;
    cursor: pointer;
  }

  .file-item:hover:not(:disabled) {
    background: var(--bg-button-hover);
  }

  .file-item.selected {
    background: var(--accent-color);
    color: white;
  }

  .file-item:disabled {
    opacity: 0.5;
  }

  .empty-text {
    color: var(--text-muted);
    text-align: center;
    font-size: 14px;
  }
</style>