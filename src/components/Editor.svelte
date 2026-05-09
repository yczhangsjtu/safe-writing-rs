<script lang="ts">
  import { passages, currentPassageIndex, config } from '../lib/stores';
  import * as api from '../lib/tauri';
  import { emit } from '@tauri-apps/api/event';
  import { isCommandKey } from '../lib/platform';
  import PassageList from './PassageList.svelte';
  import AISettingsEditor from './AISettingsEditor.svelte';

  let {
    passagesProp,
    currentIndex,
    isDirtyProp,
    onSave,
    onLock,
    passageListWidth,
    onPassageListWidthResize,
    onPassageListWidthSave
  }: {
    passagesProp: any[];
    currentIndex: number;
    isDirtyProp: boolean;
    onSave: () => Promise<void>;
    onLock: () => void;
    passageListWidth: number;
    onPassageListWidthResize: (width: number) => void;
    onPassageListWidthSave: (width: number) => void;
  } = $props();

  let editMode = $state(true);
  let editorContent = $state('');
  let editorTitle = $state('');
  let images = $state<Map<string, { data: string; index: number }>>(new Map());
  let imagesLoaded = $state(false);
  let showMetadataIndex = $state<number | null>(null);
  let metadataText = $state<string>('');
  let currentLineTop = $state(0);
  let currentLineHeight = $state(0);
  let textareaElement: HTMLTextAreaElement | undefined = $state();
  let mirrorElement: HTMLDivElement | undefined = $state();
  let isComposing = $state(false); // Track IME composition state
  let lastSyncedIndex = $state<number | null>(null); // Track last synced passage index
  let initialized = $state(false); // Track if editor is initialized
  let isAIPassage = $derived(passagesProp[currentIndex]?.title === '.ai');
  let aiSettingsEditor: { save?: () => Promise<void> } = $state({});

  // Sync editor content with current passage
  $effect(() => {
    if (passagesProp && passagesProp.length > 0 && currentIndex < passagesProp.length) {
      const backendContent = passagesProp[currentIndex]?.content || '';
      const backendTitle = passagesProp[currentIndex]?.title || '';

      if (currentIndex !== lastSyncedIndex) {
        // Passage switched — always sync
        editorContent = backendContent;
        editorTitle = backendTitle;
        lastSyncedIndex = currentIndex;
      } else if (editMode && editorContent !== backendContent) {
        // Same passage, content changed externally (e.g., CopilotPanel Insert)
        // Only sync when textarea is not focused (user isn't actively typing)
        if (document.activeElement !== textareaElement) {
          editorContent = backendContent;
        }
      }
    }
  });

  // Initialize after elements are mounted
  $effect(() => {
    if (textareaElement && mirrorElement && !initialized) {
      initialized = true;
      updateCurrentLine();
    }
  });

  // Sync font styles when config changes
  $effect(() => {
    if (mirrorElement && $config.font_size) {
      mirrorElement.style.fontSize = `${$config.font_size}px`;
      updateCurrentLine();
    }
  });

  function updateCurrentLine() {
    if (!textareaElement || !mirrorElement) return;
    const textarea = textareaElement;
    const mirror = mirrorElement;
    const text = textarea.value;
    const selectionStart = textarea.selectionStart;

    // Ensure mirror width and font styles match textarea exactly
    mirror.style.width = `${textarea.clientWidth}px`;
    mirror.style.fontSize = textarea.style.fontSize || `${$config.font_size}px`;
    mirror.style.fontFamily = textarea.style.fontFamily || "'LXGW WenKai', sans-serif";

    // Find current paragraph (logical line)
    const lines = text.substring(0, selectionStart).split('\n');
    const currentParagraphIndex = lines.length - 1;
    const allParagraphs = text.split('\n');

    // Calculate height of paragraphs before current one
    let topOffset = 0;
    for (let i = 0; i < currentParagraphIndex; i++) {
      // Use a space for empty lines to ensure proper height calculation
      mirror.textContent = allParagraphs[i] || ' ';
      topOffset += mirror.offsetHeight;
    }

    // Calculate current paragraph height
    mirror.textContent = allParagraphs[currentParagraphIndex] || ' ';
    const paragraphHeight = mirror.offsetHeight;

    // Clear mirror after calculation
    mirror.textContent = '';

    // Adjust for scroll position
    const scrollTop = textarea.scrollTop;

    currentLineTop = topOffset - scrollTop;
    currentLineHeight = paragraphHeight;
  }

  async function loadImages() {
    if (!editMode && !imagesLoaded) {
      try {
        const imageList = await api.getImages();
        const newImages = new Map<string, { data: string; index: number }>();
        for (const img of imageList) {
          newImages.set(img.digest, { data: img.data || '', index: img.index });
        }
        images = newImages;
        imagesLoaded = true;
      } catch (e) {
        console.error('Failed to load images:', e);
      }
    }
  }

  async function handleImageClick(digest: string) {
    const imgInfo = images.get(digest);
    if (!imgInfo) return;

    if (showMetadataIndex === imgInfo.index) {
      showMetadataIndex = null;
      metadataText = '';
    } else {
      showMetadataIndex = imgInfo.index;
      try {
        metadataText = await api.getImageMetadata(imgInfo.index);
      } catch (e) {
        metadataText = 'No metadata available';
      }
    }
  }

  // Just call API, state will be updated by state-changed event
  async function handleContentChange() {
    // Don't update during IME composition (macOS Chinese input fix)
    if (isComposing) return;
    if (currentIndex < passagesProp.length) {
      await api.updatePassageContent(currentIndex, editorContent);
    }
  }

  // Handle IME composition events for macOS Chinese input
  function handleCompositionStart() {
    isComposing = true;
  }

  function handleCompositionEnd() {
    isComposing = false;
    // Now update the content after composition ends
    handleContentChange();
  }

  async function handleTitleChange() {
    if (currentIndex < passagesProp.length && editorTitle.trim()) {
      await api.updatePassageTitle(currentIndex, editorTitle.trim());
    }
  }

  function handleSelectionChange(e: Event) {
    const textarea = e.target as HTMLTextAreaElement;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const cursorPos = start;
    const selected = start !== end ? editorContent.slice(start, end) : '';
    emit('editor-selection', { selected, cursorPos });
    updateCurrentLine();
  }

  function handleClick(e: Event) {
    handleSelectionChange(e);
  }

  function handleKeyUp(e: KeyboardEvent) {
    updateCurrentLine();
    // Update cursor position on arrow keys and other navigation
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight' || e.key === 'ArrowUp' || e.key === 'ArrowDown' || e.key === 'Home' || e.key === 'End') {
      handleSelectionChange(e);
    }
  }

  function handleScroll() {
    updateCurrentLine();
  }

  async function handleKeydown(e: KeyboardEvent) {
    if (isCommandKey(e) && e.key === 's') {
      e.preventDefault();
      // If AI Settings editor is active, save it first
      if (isAIPassage && aiSettingsEditor.save) {
        await aiSettingsEditor.save();
      }
      await onSave();
    }
    if (isCommandKey(e) && e.key === 'l') {
      e.preventDefault();
      if (isDirtyProp) {
        await onSave();
      }
      onLock();
    }
  }

  function toggleEditMode() {
    editMode = !editMode;
    if (!editMode) {
      imagesLoaded = false;
      showMetadataIndex = null;
      metadataText = '';
      loadImages();
    }
  }

  // Parse content and render with images
  function renderPreviewContent(content: string): { type: 'text' | 'image'; content: string; digest?: string }[] {
    const parts: { type: 'text' | 'image'; content: string; digest?: string }[] = [];
    const imagePattern = /^image!\(([a-fA-F0-9]{64})\)$/;
    const lines = content.split('\n');
    let textBuffer = '';

    for (const line of lines) {
      const match = line.match(imagePattern);
      if (match) {
        if (textBuffer) {
          parts.push({ type: 'text', content: textBuffer });
          textBuffer = '';
        }
        parts.push({ type: 'image', content: line, digest: match[1] });
      } else {
        if (textBuffer) {
          textBuffer += '\n' + line;
        } else {
          textBuffer = line;
        }
      }
    }

    if (textBuffer) {
      parts.push({ type: 'text', content: textBuffer });
    }

    return parts;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="editor-container">
  <PassageList
    passagesProp={passagesProp}
    currentIndex={currentIndex}
    isDirtyProp={isDirtyProp}
    onSave={onSave}
    onLock={onLock}
    onToggleEdit={toggleEditMode}
    editMode={editMode}
    width={passageListWidth}
    onWidthResize={onPassageListWidthResize}
    onWidthSave={onPassageListWidthSave}
  />

  <div class="editor-area">
    {#if passagesProp && passagesProp.length > 0 && currentIndex < passagesProp.length}
      {#if isAIPassage}
        <AISettingsEditor bind:this={aiSettingsEditor} />
      {:else if editMode}
        <div class="editor-content">
          <input
            type="text"
            class="title-input"
            bind:value={editorTitle}
            oninput={handleTitleChange}
            placeholder="Untitled"
          />
          <div class="textarea-wrapper">
            <div class="line-highlight" style="top: {currentLineTop}px; height: {currentLineHeight}px;"></div>
            <div class="mirror-div" bind:this={mirrorElement} style="font-size: {$config.font_size}px; font-family: 'LXGW WenKai', sans-serif;"></div>
            <textarea
              bind:value={editorContent}
              bind:this={textareaElement}
              oninput={handleContentChange}
              oncompositionstart={handleCompositionStart}
              oncompositionend={handleCompositionEnd}
              onselect={handleSelectionChange}
              onclick={handleClick}
              onkeyup={handleKeyUp}
              onscroll={handleScroll}
              placeholder="Start writing..."
              style="font-size: {$config.font_size}px; font-family: 'LXGW WenKai', sans-serif;"
            ></textarea>
          </div>
        </div>
      {:else}
        <div class="editor-content">
          <h1 class="title-display">{editorTitle || 'Untitled'}</h1>
          <div class="preview-content" style="font-size: {$config.font_size}px;">
          {#each renderPreviewContent(editorContent) as part}
            {#if part.type === 'text'}
              <pre class="text-block">{part.content}</pre>
            {:else if part.type === 'image'}
              {#if images.get(part.digest || '')}
                <div class="image-container">
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <!-- svelte-ignore a11y_missing_attribute -->
                  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                  <img
                    src="data:image/png;base64,{images.get(part.digest || '')?.data}"
                    class="embedded-image"
                    onclick={() => handleImageClick(part.digest || '')}
                  />
                  {#if showMetadataIndex === images.get(part.digest || '')?.index}
                    <div class="metadata-panel">
                      <pre class="metadata-text">{metadataText}</pre>
                    </div>
                  {/if}
                  <span class="image-digest">Click image for metadata: {part.digest?.slice(0, 16)}...</span>
                </div>
              {:else}
                <div class="image-placeholder">
                  <span class="material-icons">image</span>
                  <span>Image: {part.digest?.slice(0, 16)}...</span>
                  <span class="loading-text">Loading...</span>
                </div>
              {/if}
            {/if}
          {/each}
        </div>
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

  .title-input {
    width: 100%;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 28px;
    font-weight: 600;
    font-family: 'LXGW WenKai', sans-serif;
    padding: 0 0 var(--spacing-lg) 0;
    margin-bottom: var(--spacing-lg);
    outline: none;
    border-bottom: 1px solid transparent;
    transition: border-color 0.15s ease;
  }

  .title-input:focus {
    border-bottom-color: var(--border-color);
  }

  .title-input::placeholder {
    color: var(--text-faint);
  }

  .title-display {
    font-size: 28px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--spacing-lg);
    padding-bottom: var(--spacing-lg);
    border-bottom: 1px solid var(--border-color-faint);
    font-family: 'LXGW WenKai', sans-serif;
  }

  .textarea-wrapper {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .line-highlight {
    position: absolute;
    left: 0;
    right: 0;
    background: var(--bg-hover);
    border-radius: 4px;
    pointer-events: none;
    transition: top 0.05s ease, height 0.05s ease;
    z-index: 0;
  }

  .mirror-div {
    position: absolute;
    visibility: hidden;
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow-wrap: break-word;
    line-height: 1.8;
    padding: 0;
    pointer-events: none;
    box-sizing: border-box;
    border: none;
  }

  textarea {
    width: 100%;
    height: 100%;
    resize: none;
    border: none;
    background: transparent;
    color: var(--text-primary);
    line-height: 1.8;
    padding: 0;
    caret-color: var(--accent-color);
    outline: none;
    position: relative;
    z-index: 1;
    box-sizing: border-box;
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow-wrap: break-word;
    overflow-y: auto;
  }

  .preview-content {
    flex: 1;
    overflow-y: auto;
    line-height: 1.8;
    color: var(--text-primary);
  }

  .text-block {
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
    font-family: 'LXGW WenKai', sans-serif;
  }

  .image-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin: var(--spacing-lg) 0;
    padding: var(--spacing-sm);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color-faint);
  }

  .embedded-image {
    max-width: 100%;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .embedded-image:hover {
    opacity: 0.85;
  }

  .metadata-panel {
    width: 100%;
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--bg-input);
    border-radius: var(--radius-sm);
    max-height: 160px;
    overflow-y: auto;
  }

  .metadata-text {
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    font-family: monospace;
  }

  .image-digest {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
    margin-top: var(--spacing-xs);
  }

  .image-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    margin: var(--spacing-lg) 0;
    color: var(--text-muted);
    border: 1px solid var(--border-color-faint);
  }

  .loading-text {
    font-size: var(--font-size-xs);
    margin-top: var(--spacing-sm);
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
    margin-right: var(--spacing-xs);
  }
</style>