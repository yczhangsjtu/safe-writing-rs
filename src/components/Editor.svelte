<script lang="ts">
  import { passages, currentPassageIndex, isDirty } from '../lib/stores';
  import * as api from '../lib/tauri';
  import PassageList from './PassageList.svelte';

  export let passagesProp: any[];
  export let currentIndex: number;
  export let isDirtyProp: boolean;

  export function onSave() {}
  export function onLock() {}

  let editMode = true;
  let editorContent = '';

  $: {
    if (passagesProp && passagesProp.length > 0 && currentIndex < passagesProp.length) {
      editorContent = passagesProp[currentIndex]?.content || '';
    }
  }

  async function handleContentChange() {
    if (currentIndex < passagesProp.length) {
      await api.updatePassageContent(currentIndex, editorContent);
      isDirty.set(true);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault();
      onSave();
    }
    if (e.ctrlKey && e.key === 'l') {
      e.preventDefault();
      onLock();
    }
  }

  function toggleEditMode() {
    editMode = !editMode;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="editor-container">
  <PassageList
    passagesProp={passagesProp}
    currentIndex={currentIndex}
    isDirtyProp={isDirtyProp}
    onSave={onSave}
    onToggleEdit={toggleEditMode}
    editMode={editMode}
  />

  <div class="editor-area">
    {#if passagesProp && passagesProp.length > 0 && currentIndex < passagesProp.length}
      {#if editMode}
        <textarea
          value={editorContent}
          oninput={handleContentChange}
          placeholder="Start writing..."
        ></textarea>
      {:else}
        <div class="preview-content">
          {editorContent}
        </div>
      {/if}
    {:else}
      <div class="empty-editor">
        <p>No passage selected</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .editor-container {
    display: flex;
    flex: 1;
    background: var(--bg-editor);
    overflow: hidden;
  }

  .editor-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px;
    overflow: hidden;
  }

  textarea {
    flex: 1;
    width: 100%;
    min-height: 100%;
    resize: none;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 16px;
    line-height: 1.6;
    padding: 0;
  }

  textarea:focus {
    outline: none;
  }

  .preview-content {
    flex: 1;
    overflow-y: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
    font-size: 16px;
    line-height: 1.6;
    color: var(--text-primary);
  }

  .empty-editor {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }
</style>