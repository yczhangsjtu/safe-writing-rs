<script lang="ts">
  import { passages, currentPassageIndex } from '../lib/stores';
  import * as api from '../lib/tauri';
  import NameDialog from './NameDialog.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';

  let {
    passagesProp,
    currentIndex,
    isDirtyProp,
    editMode,
    onSave,
    onLock,
    onToggleEdit
  }: {
    passagesProp: any[];
    currentIndex: number;
    isDirtyProp: boolean;
    editMode: boolean;
    onSave: () => void;
    onLock: () => void;
    onToggleEdit: () => void;
  } = $props();

  let showNewPassageDialog = $state(false);
  let showRenameDialog = $state(false);
  let confirmDelete = $state(false);
  let pendingDeleteIndex = $state(-1);
  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let showItemMenu = $state<number | null>(null);

  const editPreviewTitle = $derived(editMode ? "Preview" : "Edit");

  async function handleSelect(index: number) {
    await api.setCurrentPassage(index);
    showItemMenu = null;
  }

  function handleAdd() {
    showNewPassageDialog = true;
  }

  async function handleNewPassageSubmit(title: string) {
    showNewPassageDialog = false;
    await api.addPassage(title);
  }

  function handleRename(index: number) {
    pendingDeleteIndex = index;
    showRenameDialog = true;
    showItemMenu = null;
  }

  async function handleRenameSubmit(newTitle: string) {
    showRenameDialog = false;
    if (pendingDeleteIndex >= 0) {
      await api.updatePassageTitle(pendingDeleteIndex, newTitle);
    }
    pendingDeleteIndex = -1;
  }

  function handleDeleteClick(index: number) {
    pendingDeleteIndex = index;
    confirmDelete = true;
    showItemMenu = null;
  }

  async function handleDeleteConfirm() {
    await api.removePassage(pendingDeleteIndex);
    confirmDelete = false;
    pendingDeleteIndex = -1;
  }

  // Drag and drop handlers
  function handleDragStart(e: DragEvent, index: number) {
    draggedIndex = index;
    showItemMenu = null;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', index.toString());
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  async function handleDrop(e: DragEvent, targetIndex: number) {
    e.preventDefault();
    if (draggedIndex !== null && draggedIndex !== targetIndex) {
      await api.movePassage(draggedIndex, targetIndex);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  async function handleDropBottom(e: DragEvent) {
    e.preventDefault();
    if (draggedIndex !== null && passagesProp.length > 0) {
      await api.movePassage(draggedIndex, passagesProp.length);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd() {
    draggedIndex = null;
    dragOverIndex = null;
  }

  function toggleItemMenu(index: number) {
    showItemMenu = showItemMenu === index ? null : index;
  }
</script>

<div class="passage-list">
  <div class="passage-header">
    <button class="btn-icon" title="Add Passage" onclick={handleAdd}>
      <span class="material-icons icon">add</span>
    </button>
    {#if isDirtyProp}
      <button class="btn-icon" title="Save" onclick={onSave}>
        <span class="material-icons icon">save</span>
      </button>
    {:else}
      <button class="btn-icon" title="Lock" onclick={onLock}>
        <span class="material-icons icon">lock</span>
      </button>
    {/if}
    <button class="btn-icon" title={editPreviewTitle} onclick={onToggleEdit}>
      <span class="material-icons icon">{#if editMode}visibility{:else}edit{/if}</span>
    </button>
  </div>

  {#if confirmDelete}
    <div class="confirm-overlay">
      <div class="confirm-dialog">
        <p class="confirm-title">Delete passage?</p>
        <p class="confirm-name">{passagesProp[pendingDeleteIndex]?.title}</p>
        <div class="confirm-actions">
          <button class="btn-danger" onclick={handleDeleteConfirm}>Delete</button>
          <button onclick={() => confirmDelete = false}>Cancel</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="passages">
    {#each passagesProp as passage, i}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="passage-item"
        class:selected={i === currentIndex}
        class:dragging={draggedIndex === i}
        class:drag-over={dragOverIndex === i}
        draggable="true"
        onclick={() => handleSelect(i)}
        ondragstart={(e) => handleDragStart(e, i)}
        ondragover={(e) => handleDragOver(e, i)}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDrop(e, i)}
        ondragend={handleDragEnd}
      >
        <span class="passage-title">{passage.title}</span>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <button
          class="btn-more"
          onclick={(e) => { e.stopPropagation(); toggleItemMenu(i); }}
        >⋮</button>
        {#if showItemMenu === i}
          <div class="item-menu">
            <button class="menu-item" onclick={() => handleRename(i)}>Rename</button>
            <button class="menu-item danger" onclick={() => handleDeleteClick(i)}>Delete</button>
          </div>
        {/if}
      </div>
    {/each}

    {#if passagesProp.length > 0}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="drop-zone"
        class:drag-over-bottom={dragOverIndex === -1}
        ondragover={(e) => { e.preventDefault(); dragOverIndex = -1; }}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDropBottom(e)}
      ></div>
    {/if}

    {#if passagesProp.length === 0}
      <div class="empty-state">
        <button class="btn-link" onclick={handleAdd}>Create first passage</button>
      </div>
    {/if}
  </div>
</div>

{#if showNewPassageDialog}
  <NameDialog
    title="Create New Passage"
    placeholder="Enter passage title..."
    onSubmit={handleNewPassageSubmit}
    onCancel={() => showNewPassageDialog = false}
  />
{/if}

{#if showRenameDialog}
  <NameDialog
    title="Rename Passage"
    placeholder="Enter new title..."
    onSubmit={handleRenameSubmit}
    onCancel={() => { showRenameDialog = false; pendingDeleteIndex = -1; }}
  />
{/if}

<style>
  .passage-list {
    width: var(--passage-list-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color-faint);
    display: flex;
    flex-direction: column;
  }

  .passage-header {
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

  .btn-icon:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-icon:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .confirm-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(2px);
  }

  .confirm-dialog {
    background: var(--bg-modal);
    border-radius: var(--radius-md);
    padding: var(--spacing-lg);
    max-width: 140px;
    text-align: center;
  }

  .confirm-title {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    margin-bottom: var(--spacing-sm);
  }

  .confirm-name {
    font-size: var(--font-size-sm);
    color: var(--danger-color);
    margin-bottom: var(--spacing-md);
    word-break: break-word;
  }

  .confirm-actions {
    display: flex;
    gap: var(--spacing-sm);
  }

  .confirm-actions button {
    flex: 1;
    padding: 6px 8px;
    border: none;
    background: var(--bg-hover);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .btn-danger {
    background: var(--danger-color);
    color: var(--text-inverse);
  }

  .confirm-actions button:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .btn-danger:hover {
    opacity: 0.9;
    color: var(--text-inverse);
  }

  .passages {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-sm);
  }

  .passage-item {
    display: flex;
    align-items: center;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color 0.15s ease;
    user-select: none;
    position: relative;
  }

  .passage-item:hover {
    background: var(--bg-hover);
  }

  .passage-item.selected {
    background: var(--bg-hover-active);
  }

  .passage-item.dragging {
    opacity: 0.5;
    background: var(--accent-color);
  }

  .passage-item.drag-over {
    border-top: 2px solid var(--accent-color);
  }

  .passage-title {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .passage-item.selected .passage-title {
    color: var(--text-primary);
  }

  .btn-more {
    display: none;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 14px;
    padding: 0;
  }

  .passage-item:hover .btn-more,
  .passage-item.selected .btn-more {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-more:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .item-menu {
    position: absolute;
    top: 100%;
    right: 4px;
    z-index: 1000;
    background: var(--bg-modal);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: var(--spacing-xs);
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 80px;
    box-shadow: var(--shadow-md);
  }

  .menu-item {
    width: 100%;
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .menu-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .menu-item.danger {
    color: var(--danger-color);
  }

  .menu-item.danger:hover {
    background: rgba(248, 71, 71, 0.1);
  }

  .drop-zone {
    height: 24px;
    margin-top: 2px;
    border-radius: var(--radius-sm);
    transition: all 0.15s ease;
  }

  .drop-zone.drag-over-bottom {
    background: var(--accent-color);
    opacity: 0.3;
    border: 2px solid var(--accent-color);
  }

  .empty-state {
    padding: var(--spacing-lg);
    text-align: center;
    font-size: var(--font-size-sm);
  }

  .btn-link {
    background: transparent;
    border: none;
    color: var(--text-accent);
    cursor: pointer;
    font-size: var(--font-size-sm);
    padding: 4px;
    border-radius: var(--radius-sm);

    &:hover {
      background: var(--bg-hover);
    }
  }
</style>