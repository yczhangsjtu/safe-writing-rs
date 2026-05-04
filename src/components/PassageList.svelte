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
    <button class="btn-sm" title="Add Passage" onclick={handleAdd}>+</button>
    <button class="btn-sm" title="Save" onclick={onSave} disabled={!isDirtyProp}>
      💾
    </button>
    <button class="btn-sm" title="Move Up" onclick={handleMoveUp} disabled={currentIndex === 0}>
      ↑
    </button>
    <button class="btn-sm" title="Move Down" onclick={handleMoveDown} disabled={currentIndex >= passagesProp.length - 1}>
      ↓
    </button>
    <div class="more-wrapper">
      <button class="btn-sm" title="More" onclick={toggleMoreMenu}>
        {#if showMoreMenu}✕{:else}⋯{/if}
      </button>
      {#if showMoreMenu}
        <div class="more-menu">
          <button class="menu-item" onclick={() => { onToggleEdit(); showMoreMenu = false; }}>
            {#if editMode}👁 Preview{:else}✏ Edit{/if}
          </button>
          <button class="menu-item" onclick={() => { handleRename(); }}>
            📝 Rename
          </button>
          <button class="menu-item warning" onclick={() => { handleDeleteClick(currentIndex); }}>
            🗑 Delete
          </button>
        </div>
      {/if}
    </div>
  </div>

  {#if showNewPassage}
    <div class="new-passage-input">
      <input
        type="text"
        bind:value={newPassageTitle}
        placeholder="Passage title"
        onkeydown={(e) => e.key === 'Enter' && handleNewPassageSubmit()}
        onblur={() => showNewPassage = false}
      />
    </div>
  {/if}

  {#if editingTitle}
    <div class="new-passage-input">
      <input
        type="text"
        bind:value={editingTitleValue}
        placeholder="New title"
        onkeydown={(e) => e.key === 'Enter' && handleRenameSubmit()}
        onblur={() => editingTitle = false}
      />
    </div>
  {/if}

  {#if confirmDelete}
    <div class="confirm-dialog">
      <p>Delete this passage?</p>
      <p class="passage-name">{passagesProp[pendingDeleteIndex]?.title}</p>
      <div class="confirm-actions">
        <button class="btn-danger" onclick={handleDeleteConfirm}>Delete</button>
        <button onclick={() => confirmDelete = false}>Cancel</button>
      </div>
    </div>
  {/if}

  <div class="passages">
    {#each passagesProp as passage, i}
      <div class="passage-item" class:selected={i === currentIndex}>
        <button
          class="passage-title"
          onclick={() => handleSelect(i)}
        >
          {passage.title}
        </button>
      </div>
    {/each}

    {#if passagesProp.length === 0}
      <button onclick={handleAdd}>Create first passage</button>
    {/if}
  </div>
</div>

<style>
  .passage-list {
    width: 180px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .passage-header {
    display: flex;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
    flex-wrap: wrap;
  }

  .btn-sm {
    width: 28px;
    height: 28px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .btn-sm:hover:not(:disabled) {
    background: var(--bg-button-hover);
  }

  .btn-sm:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-danger {
    background: var(--danger-color);
    color: white;
  }

  .more-wrapper {
    position: relative;
  }

  .more-menu {
    position: absolute;
    top: 32px;
    left: 0;
    z-index: 100;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 120px;
    box-shadow: var(--shadow-md);
  }

  .menu-item {
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    text-align: left;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .menu-item:hover {
    background: var(--bg-button-hover);
  }

  .menu-item.warning {
    color: var(--danger-color);
  }

  .new-passage-input, .confirm-dialog {
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
  }

  .new-passage-input input {
    width: 100%;
    padding: 6px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: 4px;
  }

  .confirm-dialog {
    background: rgba(231, 76, 60, 0.1);
  }

  .confirm-dialog p {
    font-size: 14px;
    color: var(--text-primary);
    margin-bottom: 4px;
  }

  .passage-name {
    font-weight: bold;
    color: var(--danger-color);
  }

  .confirm-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .confirm-actions button {
    flex: 1;
    padding: 8px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
  }

  .passages {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .passage-item {
    display: flex;
    align-items: center;
    gap: 4px;
    border-radius: 4px;
    background: var(--bg-input);
  }

  .passage-item.selected {
    background: var(--accent-color);
  }

  .passage-title {
    flex: 1;
    padding: 6px 8px;
    border: none;
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .passage-item.selected .passage-title {
    color: white;
  }
</style>