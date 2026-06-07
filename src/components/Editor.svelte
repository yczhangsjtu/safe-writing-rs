<script lang="ts">
  import { galleryVisible } from '../lib/stores';
  import * as api from '../lib/tauri';
  import PassageList from './PassageList.svelte';
  import MarkdownEditor from './MarkdownEditor.svelte';
  import AISettingsEditor from './AISettingsEditor.svelte';
  import ImageGallery from './ImageGallery.svelte';

  let {
    passagesProp,
    currentIndex,
    isDirtyProp,
    numImages,
    onSave,
    onLock,
    passageListWidth,
    onPassageListWidthResize,
    onPassageListWidthSave,
    copilotVisible,
    workspacePanelVisible,
    sessionPanelVisible,
    onToggleCopilot,
    onToggleWorkspace,
    onToggleSession
  }: {
    passagesProp: any[];
    currentIndex: number;
    isDirtyProp: boolean;
    numImages: number;
    onSave: () => Promise<void>;
    onLock: () => void;
    passageListWidth: number;
    onPassageListWidthResize: (width: number) => void;
    onPassageListWidthSave: (width: number) => void;
    copilotVisible: boolean;
    workspacePanelVisible: boolean;
    sessionPanelVisible: boolean;
    onToggleCopilot: () => void;
    onToggleWorkspace: () => void;
    onToggleSession: () => void;
  } = $props();

  let editorTitle = $state('');
  let editorContent = $state('');
  let lastSyncedIndex = $state<number | null>(null);
  let isAIPassage = $derived(passagesProp[currentIndex]?.title === '.ai');
  let aiSettingsEditor: { save?: () => Promise<void> } = $state({});

  // Sync content from backend when passage changes
  $effect(() => {
    if (passagesProp && passagesProp.length > 0 && currentIndex < passagesProp.length) {
      const backendContent = passagesProp[currentIndex]?.content || '';
      const backendTitle = passagesProp[currentIndex]?.title || '';

      editorTitle = backendTitle;

      if (currentIndex !== lastSyncedIndex) {
        // Passage switched — always sync content
        editorContent = backendContent;
        lastSyncedIndex = currentIndex;
      }
      // If same passage but content changed externally (e.g. CopilotPanel Insert),
      // the MarkdownEditor component handles syncing via its own effect
    }
  });

  async function handleTitleChange() {
    if (currentIndex < passagesProp.length && editorTitle.trim()) {
      await api.updatePassageTitle(currentIndex, editorTitle);
    }
  }

  async function handleContentChange(markdown: string) {
    if (currentIndex < passagesProp.length) {
      editorContent = markdown;
      await api.updatePassageContent(currentIndex, markdown);
    }
  }

  // Ctrl+L to lock
  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'l') {
      e.preventDefault();
      if (isDirtyProp) {
        onSave().then(() => onLock());
      } else {
        onLock();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="editor-container">
  <PassageList
    passagesProp={passagesProp}
    currentIndex={currentIndex}
    isDirtyProp={isDirtyProp}
    numImages={numImages}
    onSave={onSave}
    onLock={onLock}
    width={passageListWidth}
    onWidthResize={onPassageListWidthResize}
    onWidthSave={onPassageListWidthSave}
  />

  <div class="editor-area">
    {#if $galleryVisible}
      <ImageGallery />
    {:else if passagesProp && passagesProp.length > 0 && currentIndex < passagesProp.length}
      {#if isAIPassage}
        <AISettingsEditor bind:this={aiSettingsEditor} />
      {:else}
        <div class="editor-content">
          <div class="title-row">
            <input
              type="text"
              class="title-input"
              bind:value={editorTitle}
              oninput={handleTitleChange}
              placeholder="Untitled"
            />
            <div class="title-buttons">
              <button class="btn-icon" class:active={copilotVisible} title="AI Copilot" onclick={onToggleCopilot}>
                <span class="material-icons icon">auto_awesome</span>
              </button>
              <button class="btn-icon" class:active={workspacePanelVisible} title="Workspace" onclick={onToggleWorkspace}>
                <span class="material-icons icon">group</span>
              </button>
              <button class="btn-icon" class:active={sessionPanelVisible} title="Session" onclick={onToggleSession}>
                <span class="material-icons icon">history</span>
              </button>
            </div>
          </div>
          <MarkdownEditor
            content={editorContent}
            onContentChange={handleContentChange}
            onSave={onSave}
          />
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
    overflow: hidden;
    gap: var(--card-gap);
    padding: 0;
  }

  .editor-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-card);
    border-radius: var(--card-radius);
  }

  .editor-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--spacing-xl);
    overflow: hidden;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
  }

  .title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
    flex-shrink: 0;
  }

  .title-input {
    flex: 1;
    border: none !important;
    background: transparent;
    color: var(--text-primary);
    font-size: 28px;
    font-weight: 600;
    font-family: 'LXGW WenKai', sans-serif;
    padding: 0;
    outline: none !important;
    box-shadow: none !important;
  }

  .title-buttons {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
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

  .btn-icon:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-icon.active {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .btn-icon.active:hover {
    opacity: 0.9;
  }

  .icon {
    font-size: 16px;
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

  .title-input:focus {
    border: none !important;
    outline: none !important;
    box-shadow: none !important;
  }

  .title-input::placeholder {
    color: var(--text-faint);
  }

  .empty-editor {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }
</style>
