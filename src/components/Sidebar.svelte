<script lang="ts">
  import { files, theme } from '../lib/stores';
  import * as api from '../lib/tauri';
  import NameDialog from './NameDialog.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import ThemeToggle from './ThemeToggle.svelte';

  let {
    filesProp,
    currentFile,
    isDirtyProp,
    onSave,
    onFileSelect,
    onOpenSettings
  }: {
    filesProp: string[];
    currentFile: string | null;
    isDirtyProp: boolean;
    onSave: () => void;
    onFileSelect: (filename: string) => void;
    onOpenSettings: () => void;
  } = $props();

  let showNewFileDialog = $state(false);
  let showUnsavedDialog = $state(false);
  let pendingFilename = $state('');

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
    showNewFileDialog = true;
  }

  function handleNewFileSubmit(filename: string) {
    showNewFileDialog = false;
    if (filesProp.includes(filename)) {
      return;
    }
    onFileSelect(filename);
  }

  function handleFileClick(filename: string) {
    if (filename === currentFile) return;

    if (isDirtyProp) {
      pendingFilename = filename;
      showUnsavedDialog = true;
    } else {
      onFileSelect(filename);
    }
  }

  function handleSaveAndSwitch() {
    showUnsavedDialog = false;
    onSave();
    onFileSelect(pendingFilename);
    pendingFilename = '';
  }

  function handleDiscardAndSwitch() {
    showUnsavedDialog = false;
    onFileSelect(pendingFilename);
    pendingFilename = '';
  }

  function handleCancelSwitch() {
    showUnsavedDialog = false;
    pendingFilename = '';
  }

  function handleThemeChange(newTheme: 'light' | 'dark') {
    theme.set(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    api.updateConfig({ theme: newTheme });
  }
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <button class="btn-icon" title="New File" onclick={handleNewFile}>
      <span class="icon">+</span>
    </button>
    <button class="btn-icon" title="Refresh" onclick={handleRefresh} disabled={isDirtyProp}>
      <span class="icon">↻</span>
    </button>
  </div>

  <div class="file-list">
    {#each filesProp as filename}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="file-item"
        class:selected={filename === currentFile}
        onclick={() => handleFileClick(filename)}
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

  <div class="sidebar-footer">
    <button class="btn-icon" title="Settings" onclick={onOpenSettings}>
      <span class="icon">⚙</span>
    </button>
    <ThemeToggle currentTheme={$theme} onThemeChange={handleThemeChange} />
  </div>
</div>

{#if showNewFileDialog}
  <NameDialog
    title="Create New File"
    placeholder="Enter file name..."
    onSubmit={handleNewFileSubmit}
    onCancel={() => showNewFileDialog = false}
  />
{/if}

{#if showUnsavedDialog}
  <ConfirmDialog
    message="Current file has unsaved changes. Do you want to save before switching?"
    confirmText="Save"
    cancelText="Discard"
    onConfirm={handleSaveAndSwitch}
    onCancel={handleCancelSwitch}
    onDiscard={handleDiscardAndSwitch}
  />
{/if}

<style>
  .sidebar {
    width: var(--sidebar-width);
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border-color-faint);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    gap: 4px;
    padding: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
  }

  .btn-icon {
    width: 32px;
    height: 32px;
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
    font-size: 16px;
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

  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-sm);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 6px 10px;
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
    font-size: 14px;
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

  .sidebar-footer {
    display: flex;
    gap: 4px;
    padding: var(--spacing-md);
    border-top: 1px solid var(--border-color-faint);
    margin-top: auto;
  }
</style>