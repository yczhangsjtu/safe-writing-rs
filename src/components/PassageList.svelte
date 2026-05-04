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

  async function handleDelete(index: number) {
    if (confirm('Delete this passage?')) {
      await api.removePassage(index);
      const updatedPassages = await api.getPassages();
    passages.set(updatedPassages);
    isDirty.set(true);
      if (currentIndex >= updatedPassages.length) {
        currentPassageIndex.set(Math.max(0, updatedPassages.length - 1));
      }
    }
  }
</script>

<div class="passage-list">
  <div class="passage-header">
    <button class="btn-sm" title="Add" onclick={handleAdd}>+</button>
    <button class="btn-sm" title="Save" onclick={onSave} disabled={!isDirtyProp}>
      💾
    </button>
    <button class="btn-sm" title="Move Up" onclick={handleMoveUp} disabled={currentIndex === 0}>
      ↑
    </button>
    <button class="btn-sm" title="Move Down" onclick={handleMoveDown} disabled={currentIndex >= passagesProp.length - 1}>
      ↓
    </button>
    <button class="btn-sm" title="Toggle Preview" onclick={onToggleEdit}>
      {#if editMode}👁{:else}✏{/if}
    </button>
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

  <div class="passages">
    {#each passagesProp as passage, i}
      <div class="passage-item" class:selected={i === currentIndex}>
        <button
          class="passage-title"
          onclick={() => handleSelect(i)}
        >
          {passage.title}
        </button>
        <button class="btn-sm btn-danger" onclick={() => handleDelete(i)}>
          ×
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

  .new-passage-input {
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