<script lang="ts">
  import { onMount } from 'svelte';
  import { passages, currentPassageIndex, galleryVisible, isDirty } from '../lib/stores';
  import * as api from '../lib/tauri';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import Resizable from './Resizable.svelte';

  let {
    passagesProp,
    currentIndex,
    onSave,
    onLock,
    numImages,
    width,
    onWidthResize,
    onWidthSave
  }: {
    passagesProp: any[];
    currentIndex: number;
    onSave: () => void;
    onLock: () => void;
    numImages: number;
    width: number;
    onWidthResize: (width: number) => void;
    onWidthSave: (width: number) => void;
  } = $props();

  // Regular passages (exclude special .ai passage — shown in system section)
  let regularPassages = $derived(passagesProp.filter((p: any) => p.title !== '.ai' && p.title !== '.gallery'));
  // Find .ai passage index for navigation
  let aiPassageIdx = $derived(passagesProp.findIndex((p: any) => p.title === '.ai'));
  let showGallery = $derived(numImages > 0);

  // Debug: use subscribe instead of $effect for store
  onMount(() => {
    return isDirty.subscribe((value) => {
      console.log('[PassageList] isDirty.subscribe callback:', value, performance.now());
    });
  });

  let confirmDelete = $state(false);
  let pendingDeleteIndex = $state(-1);
  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let showItemMenu = $state<number | null>(null);


  function generateTimestampTitle(): string {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    const hours = String(now.getHours()).padStart(2, '0');
    const minutes = String(now.getMinutes()).padStart(2, '0');
    const seconds = String(now.getSeconds()).padStart(2, '0');
    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
  }

  async function handleSelect(index: number) {
    galleryVisible.set(false);
    await api.setCurrentPassage(index);
    showItemMenu = null;
  }

  function handleOpenGallery() {
    galleryVisible.set(true);
  }

  function handleOpenAISettings() {
    galleryVisible.set(false);
    if (aiPassageIdx >= 0) {
      api.setCurrentPassage(aiPassageIdx);
    }
  }

  async function handleAdd() {
    const title = generateTimestampTitle();
    await api.addPassage(title);
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

<div class="passage-list" style="width: {width}px;">
  <Resizable width={width} side="right" onResize={onWidthResize} onSave={onWidthSave} />
  <div class="passage-header">
    <button class="btn-icon" title="Add Passage" onclick={handleAdd}>
      <span class="material-icons icon">add</span>
    </button>
    <!-- Debug: show raw value -->
    <span style="color: red; font-size: 10px;">dirty={$isDirty}</span>
    {#if $isDirty}
      <button class="btn-icon" title="Save" onclick={onSave}>
        <span class="material-icons icon">save</span>
      </button>
    {:else}
      <button class="btn-icon" title="Lock" onclick={onLock}>
        <span class="material-icons icon">lock</span>
      </button>
    {/if}
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
      {#if passage.title === '.ai' || passage.title === '.gallery'}
        <!-- Skip special passages — shown in system section below -->
      {:else}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="passage-item"
          class:selected={i === currentIndex && !$galleryVisible}
          class:dragging={draggedIndex === i}
          class:drag-over={dragOverIndex === i}
          draggable={true}
          onclick={() => handleSelect(i)}
          ondragstart={(e) => handleDragStart(e, i)}
          ondragover={(e) => handleDragOver(e, i)}
          ondragleave={handleDragLeave}
          ondrop={(e) => handleDrop(e, i)}
          ondragend={handleDragEnd}
        >
          <span class="passage-title">{passage.title}</span>
          <button
            class="btn-more"
            onclick={(e) => { e.stopPropagation(); toggleItemMenu(i); }}
          >⋮</button>
          {#if showItemMenu === i}
            <div class="item-menu">
              <button class="menu-item danger" onclick={() => handleDeleteClick(i)}>Delete</button>
            </div>
          {/if}
        </div>
      {/if}
    {/each}

    {#if regularPassages.length > 0}
      <div
        class="drop-zone"
        class:drag-over-bottom={dragOverIndex === -1}
        ondragover={(e) => { e.preventDefault(); dragOverIndex = -1; }}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDropBottom(e)}
      ></div>
    {/if}

    {#if regularPassages.length === 0}
      <div class="empty-state">
        <button class="btn-link" onclick={handleAdd}>Create first passage</button>
      </div>
    {/if}
  </div>

  <!-- System section: AI Settings + Image Gallery at the bottom -->
  <div class="system-section">
    <div class="system-divider"></div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="passage-item system-item"
      class:selected={aiPassageIdx >= 0 && aiPassageIdx === currentIndex && !$galleryVisible}
      onclick={handleOpenAISettings}
    >
      <span class="material-icons system-icon">smart_toy</span>
      <span class="passage-title">AI Settings</span>
    </div>
    {#if showGallery}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="passage-item system-item"
        class:selected={$galleryVisible}
        onclick={handleOpenGallery}
      >
        <span class="material-icons system-icon">image</span>
        <span class="passage-title">Image Gallery</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .passage-list {
    background: var(--bg-card);
    border-radius: var(--card-radius);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
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

  .passage-title.special {
    font-weight: 600;
    color: var(--text-faint);
  }

  .passage-item.selected .passage-title.special {
    color: var(--text-muted);
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

  .system-section {
    padding: 0 var(--spacing-sm) var(--spacing-sm);
    flex-shrink: 0;
  }

  .system-divider {
    height: 1px;
    background: var(--border-color);
    margin: var(--spacing-sm) 0;
  }

  .system-item {
    opacity: 0.7;
    padding: 4px 10px;
  }

  .system-item:hover {
    opacity: 1;
  }

  .system-item.selected {
    opacity: 1;
    background: var(--bg-hover-active);
  }

  .system-icon {
    font-size: 14px;
    margin-right: 6px;
    color: var(--text-faint);
    line-height: 1;
    width: 18px;
    text-align: center;
  }

  .system-item.selected .system-icon {
    color: var(--text-muted);
  }
</style>