<script lang="ts">
  import { onMount } from 'svelte';
  import Vditor from 'vditor';
  import 'vditor/dist/index.css';
  import { config } from '../lib/stores';
  import * as api from '../lib/tauri';

  let {
    content,
    onContentChange,
    onSave,
  }: {
    content: string;
    onContentChange: (markdown: string) => Promise<void>;
    onSave: () => Promise<void>;
  } = $props();

  let editorRoot = $state<HTMLDivElement>();
  let vditor = $state<Vditor>();
  let initialized = $state(false);
  let prevStoredContent = $state('');
  let imageMap: Map<string, string> = new Map();
  let suppressInput = $state(false);

  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve((reader.result as string).split(',')[1]);
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  async function refreshImageMap(): Promise<void> {
    try {
      const images = await api.getImages();
      const m = new Map<string, string>();
      for (const img of images) {
        if (img.data) m.set(img.digest, img.data);
      }
      imageMap = m;
    } catch (_) {}
  }

  function toDisplay(md: string): string {
    if (imageMap.size === 0) return md;
    return md.replace(
      /!\[([^\]]*)\]\(([a-fA-F0-9]{64})\)/g,
      (_m, alt, digest) => {
        const b64 = imageMap.get(digest.toLowerCase());
        return b64 ? `![${alt}](data:image/png;base64,${b64})` : _m;
      }
    );
  }

  function toStorage(md: string): string {
    return md.replace(
      /!\[([^\]]*)\]\(data:image\/[^;]+;base64,([A-Za-z0-9+/=]+)\)/g,
      (_m, _alt, b64) => {
        for (const [digest, data] of imageMap) {
          if (data === b64) return `![image](${digest})`;
        }
        return _m;
      }
    );
  }

  // Sync external content changes into editor
  $effect(() => {
    if (!vditor || !initialized) return;
    if (content === undefined || content === null) return;
    refreshImageMap().then(() => {
      if (content !== prevStoredContent) {
        prevStoredContent = content;
        const displayContent = toDisplay(content);
        suppressInput = true;
        vditor!.setValue(displayContent);
        suppressInput = false;
      }
    });
  });

  // Sync Vditor theme when app theme changes
  $effect(() => {
    if (vditor && initialized) {
      vditor.setTheme($config.theme === 'dark' ? 'dark' : 'classic');
    }
  });

  // Process image files: encrypt and insert into editor
  async function processImageFiles(files: FileList | File[]) {
    const arr = Array.from(files);
    let inserted = false;
    for (const file of arr) {
      const isImage = file.type?.startsWith('image/') || file.name?.match(/\.(png|jpg|jpeg|gif|webp|bmp)$/i);
      if (!isImage) continue;
      try {
        const buffer = await file.arrayBuffer();
        const bytes = new Uint8Array(buffer);
        const b64 = await fileToBase64(file);
        const digest = await api.insertImage(Array.from(bytes));
        imageMap.set(digest, b64);
        vditor!.insertValue(`![image](data:image/png;base64,${b64})`);
        inserted = true;
      } catch (e) {
        console.error('[MarkdownEditor] image insert failed:', e);
      }
    }

    if (inserted && vditor) {
      const stored = toStorage(vditor.getValue());
      if (stored !== prevStoredContent) {
        prevStoredContent = stored;
        await onContentChange(stored);
      }
    }
  }

  function isImageFile(file: File): boolean {
    return file.type?.startsWith('image/') || /\.(png|jpg|jpeg|gif|webp|bmp)$/i.test(file.name || '');
  }

  function imageFilesFromItems(items?: DataTransferItemList | null): File[] {
    if (!items) return [];

    const files: File[] = [];
    for (const item of Array.from(items)) {
      if (item.kind !== 'file' || !item.type.startsWith('image/')) continue;
      const file = item.getAsFile();
      if (file) files.push(file);
    }
    return files;
  }

  function imageFilesFromFileList(files?: FileList | null): File[] {
    if (!files) return [];
    return Array.from(files).filter(isImageFile);
  }

  function imageFilesFromTransfer(data?: DataTransfer | null): File[] {
    const itemFiles = imageFilesFromItems(data?.items);
    return itemFiles.length > 0 ? itemFiles : imageFilesFromFileList(data?.files);
  }

  function imageFilesFromClipboard(data?: DataTransfer | null): File[] {
    const itemFiles = imageFilesFromItems(data?.items);
    return itemFiles.length > 0 ? itemFiles : imageFilesFromFileList(data?.files);
  }

  async function processImagesFromBackend() {
    try {
      const images = await api.readClipboardImages();
      if (!images?.length) return;
      for (const img of images) {
        const bytes = new Uint8Array(img.data);
        const b64 = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve((reader.result as string).split(',')[1]);
          reader.onerror = reject;
          reader.readAsDataURL(new Blob([bytes]));
        });
        const digest = await api.insertImage(Array.from(bytes));
        imageMap.set(digest, b64);
        vditor!.insertValue(`![image](data:image/png;base64,${b64})`);
      }
      const stored = toStorage(vditor!.getValue());
      if (stored !== prevStoredContent) {
        prevStoredContent = stored;
        await onContentChange(stored);
      }
    } catch (_) {}
  }

  // Use capture phase to intercept drop/paste BEFORE Vditor's handlers
  function setupImageHandlers(el: HTMLElement) {
    el.addEventListener('dragover', (e: DragEvent) => {
      const data = e.dataTransfer;
      if (data && (data.types.includes('Files') || imageFilesFromItems(data.items).length > 0)) {
        e.preventDefault();
        e.stopPropagation();
        data.dropEffect = 'copy';
      }
    }, true); // capture phase

    el.addEventListener('drop', (e: DragEvent) => {
      const files = imageFilesFromTransfer(e.dataTransfer);
      if (files.length > 0) {
        e.preventDefault();
        e.stopPropagation();
        void processImageFiles(files);
      }
    }, true); // capture phase

    el.addEventListener('paste', (e: ClipboardEvent) => {
      const files = imageFilesFromClipboard(e.clipboardData);
      if (files.length > 0) {
        e.preventDefault();
        e.stopPropagation();
        void processImageFiles(files);
        return;
      }
      if (e.clipboardData?.types.includes('text/uri-list')) {
        e.preventDefault();
        e.stopPropagation();
        void processImagesFromBackend();
      }
    }, true); // capture phase
  }

  onMount(async () => {
    if (!editorRoot) return;
    setupImageHandlers(editorRoot);
    await refreshImageMap();
    prevStoredContent = content;
    const displayContent = toDisplay(content);

    vditor = new Vditor(editorRoot, {
      mode: 'ir',
      value: displayContent,
      toolbar: [],
      toolbarConfig: { hide: true },
      outline: { enable: false, position: 'left' },
      counter: { enable: false },
      cache: { enable: false },
      placeholder: 'Start writing...',
      height: '100%',
      theme: $config.theme === 'dark' ? 'dark' : 'classic',
      input(value) {
        if (suppressInput) return;
        const stored = toStorage(value);
        if (stored !== prevStoredContent) {
          prevStoredContent = stored;
          onContentChange(stored);
        }
      },
      keydown(event) {
        if ((event.metaKey || event.ctrlKey) && event.key === 's') {
          event.preventDefault();
          onSave();
        }
      },
      after() {
        initialized = true;
      },
    });
  });
</script>

<div
  bind:this={editorRoot}
  class="markdown-editor"
  style="font-size: {$config.font_size}px;"
></div>

<style>
  .markdown-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  /* === Vditor container — transparent, no border, no box === */
  .markdown-editor :global(.vditor) {
    flex: 1;
    display: flex !important;
    flex-direction: column;
    overflow: hidden;
    border: none !important;
    background: transparent !important;
    border-radius: 0 !important;
    box-shadow: none !important;
    outline: none !important;
  }

  .markdown-editor :global(.vditor-content) {
    flex: 1;
    overflow-y: auto;
    background: transparent !important;
    border: none !important;
    outline: none !important;
  }

  /* === IR editing area === */
  .markdown-editor :global(.vditor-ir) {
    min-height: 100%;
    padding: 0 !important;
    line-height: 1.8;
    background: transparent !important;
    color: var(--text-primary) !important;
    font-family: 'LXGW WenKai', sans-serif !important;
    font-size: inherit !important;
    border: none !important;
    outline: none !important;
  }

  /* The actual editable pre element */
  .markdown-editor :global(.vditor-ir pre.vditor-reset) {
    padding: 0 !important;
    margin: 0 !important;
    background: transparent !important;
    color: var(--text-primary) !important;
    font-family: 'LXGW WenKai', sans-serif !important;
    font-size: inherit !important;
    line-height: 1.8 !important;
    border: none !important;
    outline: none !important;
  }

  /* Kill focus highlight — no blue background on click */
  .markdown-editor :global(.vditor-ir pre.vditor-reset:focus) {
    background: transparent !important;
    outline: none !important;
    box-shadow: none !important;
    border: none !important;
  }

  /* === Override all Vditor reset text colors to use our theme === */
  .markdown-editor :global(.vditor-reset) {
    color: var(--text-primary) !important;
    font-size: inherit !important;
    font-family: 'LXGW WenKai', sans-serif !important;
    line-height: 1.8 !important;
  }

  .markdown-editor :global(.vditor-reset h1),
  .markdown-editor :global(.vditor-reset h2),
  .markdown-editor :global(.vditor-reset h3),
  .markdown-editor :global(.vditor-reset h4),
  .markdown-editor :global(.vditor-reset h5),
  .markdown-editor :global(.vditor-reset h6) {
    color: var(--text-primary) !important;
    font-family: 'LXGW WenKai', sans-serif !important;
    border-bottom: none !important;
  }

  .markdown-editor :global(.vditor-reset code:not(.hljs)) {
    color: var(--text-primary) !important;
    background: var(--bg-hover) !important;
  }

  .markdown-editor :global(.vditor-reset a) {
    color: var(--text-accent) !important;
  }

  .markdown-editor :global(.vditor-reset blockquote) {
    color: var(--text-secondary) !important;
  }

  .markdown-editor :global(.vditor-reset hr) {
    border-color: var(--border-color) !important;
  }

  .markdown-editor :global(.vditor-reset table),
  .markdown-editor :global(.vditor-reset th),
  .markdown-editor :global(.vditor-reset td) {
    border-color: var(--border-color) !important;
    color: var(--text-primary) !important;
  }

  :global(.vditor-ir pre.vditor-reset ::selection),
  :global(.vditor-reset ::selection) {
    background: var(--accent-color) !important;
    color: var(--text-inverse) !important;
  }

  .markdown-editor :global(.vditor-ir__marker) {
    color: var(--text-faint) !important;
    font-weight: normal !important;
    font-style: normal !important;
  }

  /* Kill dark-mode specific borders/backgrounds */
  :global(.vditor--dark .vditor-content),
  :global(.vditor--dark .vditor-ir),
  :global(.vditor--dark .vditor-reset) {
    background: transparent !important;
    border: none !important;
    box-shadow: none !important;
  }

  /* Direct kill on .vditor itself (belt and suspenders) */
  :global(.vditor) {
    border: none !important;
    outline: none !important;
    box-shadow: none !important;
    --border-color: transparent !important;
  }
</style>
