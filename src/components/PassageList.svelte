<script lang="ts">
  import { passages, currentPassageIndex, isDirty } from '../lib/stores';
  import * as api from '../lib/tauri';

  let {
    passagesProp,
    currentIndex,
    isDirtyProp,
    editMode,
    onSave,
    onToggleEdit
  }: {
    passagesProp: any[];
    currentIndex: number;
    isDirtyProp: boolean;
    editMode: boolean;
    onSave: () => void;
    onToggleEdit: () => void;
  } = $props();

  let newPassageTitle = $state('');
  let showNewPassage = $state(false);
  let showMoreMenu = $state(false);
  let editingTitle = $state(false);
  let editingTitleValue = $state('');
  let confirmDelete = $state(false);
  let pendingDeleteIndex = $state(-1);

  async function handleSelect(index: number) {
    await api.setCurrentPassage(index);
    currentPassageIndex.set(index);
  }

  function handleAdd() {
    showNewPassage = true;
    newPassageTitle = '';
  }

  async function handleNewPassageSubmit() {
    if (!newPassageTitle.trim()) {
      showNewPassage = false;
      return;
    }
    await api.addPassage(newPassageTitle.trim());
    const updatedPassages = await api.getPassages();
    passages.set(updatedPassages);
    isDirty.set(true);
    showNewPassage = false;
  }

  async function handleMoveUp() {
    if (currentIndex > 0) {
      await api.movePassage(currentIndex, currentIndex - 1);
      currentPassageIndex.set(currentIndex - 1);
      const updatedPassages = await api.getPassages();
      passages.set(updatedPassages);
      isDirty.set(true);
    }
  }

  async function handleMoveDown() {
    if (currentIndex < passagesProp.length - 1) {
      await api.movePassage(currentIndex, currentIndex + 1);
      currentPassageIndex.set(currentIndex + 1);
      const updatedPassages = await api.getPassages();
      passages.set(updatedPassages);
      isDirty.set(true);
    }
  }

  function handleRename() {
    editingTitleValue = passagesProp[currentIndex]?.title || '';
    editingTitle = true;
    showMoreMenu = false;
  }

  async function handleRenameSubmit() {
    if (editingTitleValue.trim()) {
      await api.updatePassageTitle(currentIndex, editingTitleValue.trim());
      const updatedPassages = await api.getPassages();
      passages.set(updatedPassages);
      isDirty.set(true);
    }
    editingTitle = false;
  }

  function handleDeleteClick(index: number) {
    pendingDeleteIndex = index;
    confirmDelete = true;
    showMoreMenu = false;
  }

  async function handleDeleteConfirm() {
    await api.removePassage(pendingDeleteIndex);
    const updatedPassages = await api.getPassages();
    passages.set(updatedPassages);
    isDirty.set(true);
    if (currentIndex >= updatedPassages.length) {
      currentPassageIndex.set(Math.max(0, updatedPassages.length - 1));
    }
    confirmDelete = false;
    pendingDeleteIndex = -1;
  }

  function toggleMoreMenu() {
    showMoreMenu = !showMoreMenu;
  }
</script>

<div class="passage-list">
  <div class="passage-header">
    <button class="btn-icon" title="Add" onclick={handleAdd}>
      <span class="icon">+</span>
    </button>
    <button class="btn-icon" title="Save" onclick={onSave} disabled={!isDirtyProp}>
      <span class="icon">{#if isDirtyProp}💾{:else}○{/if}</span>
    </button>
    <button class="btn-icon" title="Up" onclick={handleMoveUp} disabled={currentIndex === 0}>
      <span class="icon">↑</span>
    </button>
    <button class="btn-icon" title="Down" onclick={handleMoveDown} disabled={currentIndex >= passagesProp.length - 1}>
      <span class="icon">↓</span>
    </button>
    <div class="more-wrapper">
      <button class="btn-icon" title="More" onclick={toggleMoreMenu}>
        <span class="icon">{#if showMoreMenu}✕{:else}⋮{/if}</span>
      </button>
      {#if showMoreMenu}
        <div class="more-menu">
          <button class="menu-item" onclick={() => { onToggleEdit(); showMoreMenu = false; }}>
            {#if editMode}Preview{:else}Edit{/if}
          </button>
          <button class="menu-item" onclick={() => { handleRename(); }}>
            Rename
          </button>
          <button class="menu-item danger" onclick={() => { handleDeleteClick(currentIndex); }}>
            Delete
          </button>
        </div>
      {/if}
    </div>
  </div>

  {#if showNewPassage}
    <div class="input-row">
      <input
        type="text"
        bind:value={newPassageTitle}
        placeholder="Title..."
        onkeydown={(e) => e.key === 'Enter' && handleNewPassageSubmit()}
        onblur={() => showNewPassage = false}
      />
    </div>
  {/if}

  {#if editingTitle}
    <div class="input-row">
      <input
        type="text"
        bind:value={editingTitleValue}
        placeholder="New title..."
        onkeydown={(e) => e.key === 'Enter' && handleRenameSubmit()}
        onblur={() => editingTitle = false}
      />
    </div>
  {/if}

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
        onclick={() => handleSelect(i)}
      >
        <span class="passage-title">{passage.title}</span>
      </div>
    {/each}

    {#if passagesProp.length === 0}
      <div class="empty-state">
        <button class="btn-link" onclick={handleAdd}>Create first passage</button>
      </div>
    {/if}
  </div>
</div>

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
    gap: 2px;
    padding: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    position: relative;
  }

  .btn-icon {
    width: 24px;
    height: 24px;
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

  .more-wrapper {
    position: relative;
    z-index: 10;
  }

  .more-menu {
    position: absolute;
    top: 28px;
    left: 0;
    z-index: 1000;
    background: var(--bg-modal);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: var(--spacing-xs);
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 100px;
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
    font-size: var(--font-size-xs);
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

  .input-row {
    padding: var(--spacing-sm) var(--spacing-md);
    border-bottom: 1px solid var(--border-color-faint);
  }

  .input-row input {
    width: 100%;
    padding: 4px 8px;
    border: none;
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
  }

  .input-row input:focus {
    outline: none;
    box-shadow: inset 0 0 0 1px var(--accent-color);
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
    font-size: var(--font-size-xs);
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
    font-size: var(--font-size-xs);
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
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color 0.15s ease;
    user-select: none;
  }

  .passage-item:hover {
    background: var(--bg-hover);
  }

  .passage-item.selected {
    background: var(--bg-hover-active);
  }

  .passage-title {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .passage-item.selected .passage-title {
    color: var(--text-primary);
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
    font-size: var(--font-size-xs);
    padding: 4px;
    border-radius: var(--radius-sm);

    &:hover {
      background: var(--bg-hover);
    }
  }
</style>