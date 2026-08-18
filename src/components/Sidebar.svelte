<script lang="ts">
  import { files, theme, isDirty } from '../lib/stores';
  import * as api from '../lib/tauri';
  import NameDialog from './NameDialog.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import ThemeToggle from './ThemeToggle.svelte';
  import Resizable from './Resizable.svelte';

  let {
    filesProp,
    currentFile,
    isDirtyProp,
    onSave,
    onFileSelect,
    onOpenSettings,
    width,
    onWidthResize,
    onWidthSave
  }: {
    filesProp: string[];
    currentFile: string | null;
    isDirtyProp: boolean;
    onSave: () => void;
    onFileSelect: (filename: string) => void;
    onOpenSettings: () => void;
    width: number;
    onWidthResize: (width: number) => void;
    onWidthSave: (width: number) => void;
  } = $props();

  let showNewFileDialog = $state(false);
  let showUnsavedDialog = $state(false);
  let pendingFilename = $state('');

  async function handleRefresh() {
    if ($isDirty) return;
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

    if ($isDirty) {
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

<div class="sidebar" style="width: {width}px;">
  <Resizable width={width} side="right" onResize={onWidthResize} onSave={onWidthSave} />
  <div class="sidebar-header">
    <button class="btn-icon" title="New File" onclick={handleNewFile}>
      <span class="material-icons icon">add</span>
    </button>
    <button class="btn-icon" title="Refresh" onclick={handleRefresh} disabled={$isDirty}>
      <span class="material-icons icon">refresh</span>
    </button>
  </div>

  <div class="file-list">
    {#each filesProp as filename}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="file-item"
        class:selected={filename === currentFile}
        onclick={() => handleFileClick(filename)}
        role="button"
        tabindex="0"
      >
        <span class="material-icons file-icon">description</span>
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
      <span class="material-icons icon">settings</span>
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
    background: var(--bg-card);
    border-radius: var(--card-radius);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
    position: relative;
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
    font-size: 18px;
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
    font-size: 16px;
    opacity: 0.6;
  }

  .material-icons {
    font-family: 'Material Icons';
    font-weight: normal;
    font-style: normal;
    display: inline-block;
    line-height: 1;
    text-transform: none;
    letter-spacing: normal;
    word-wrap: normal;
    white-space: nowrap;
    direction: ltr;
    -webkit-font-smoothing: antialiased;
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
    margin-top: auto;
  }
</style>