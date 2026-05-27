<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
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
  let imageMap: Map<string, string> = new Map();          // digest → base64
  let digestToBlob: Map<string, string> = new Map();       // digest → blobUrl
  let blobToDigest: Map<string, string> = new Map();       // blobUrl → digest
  let suppressInput = $state(false);

  function base64ToBytes(b64: string): Uint8Array {
    const binary = atob(b64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return bytes;
  }

  function getOrCreateBlobUrl(digest: string, b64: string): string {
    let url = digestToBlob.get(digest);
    if (!url) {
      const blob = new Blob([base64ToBytes(b64)], { type: 'image/png' });
      url = URL.createObjectURL(blob);
      digestToBlob.set(digest, url);
      blobToDigest.set(url, digest);
    }
    return url;
  }

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

  // Convert digests → blob URLs for display (tiny strings, fast for Vditor)
  function toDisplay(md: string): string {
    if (imageMap.size === 0) return md;
    return md.replace(
      /!\[([^\]]*)\]\(([a-fA-F0-9]{64})\)/g,
      (_m, alt, digest) => {
        const b64 = imageMap.get(digest.toLowerCase());
        return b64 ? `![${alt}](${getOrCreateBlobUrl(digest, b64)})` : _m;
      }
    );
  }

  // Convert blob URLs (and legacy data URIs) → digests for storage
  function toStorage(md: string): string {
    // 1) blob: URLs (current path)
    md = md.replace(
      /!\[([^\]]*)\]\((blob:[^)\s]+)\)/g,
      (_m, alt, blobUrl) => {
        const digest = blobToDigest.get(blobUrl);
        return digest ? `![${alt}](${digest})` : _m;
      }
    );
    // 2) Legacy data: URIs
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
    console.log('[processImageFiles] got', arr.length, 'files:', arr.map(f => ({ name: f.name, type: f.type, size: f.size })));
    let inserted = false;
    for (const file of arr) {
      const isImage = file.type?.startsWith('image/') || file.name?.match(/\.(png|jpg|jpeg|gif|webp|bmp)$/i);
      if (!isImage) {
        console.log('[processImageFiles] skipping non-image:', file.name, file.type);
        continue;
      }
      try {
        // Create blob URL from File — instant, tiny string (~40 bytes)
        const blobUrl = URL.createObjectURL(file);
        // Insert blob URL immediately (user sees image right away, no freeze)
        vditor!.insertValue(`![image](${blobUrl})`);
        inserted = true;
        // In parallel: compute base64 → send to backend → get digest
        const b64 = await fileToBase64(file);
        const digest = await api.insertImage(b64);
        imageMap.set(digest, b64);
        blobToDigest.set(blobUrl, digest);
        digestToBlob.set(digest, blobUrl);
      } catch (e) {
        console.error('[MarkdownEditor] image insert failed:', e);
      }
    }

    if (inserted && vditor) {
      // Save content — blob URLs are converted to digests by toStorage
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
      if (item.kind !== 'file') {
        console.log('[imageFilesFromItems] skipping item kind=', item.kind, 'type=', item.type);
        continue;
      }
      const file = item.getAsFile();
      console.log('[imageFilesFromItems] kind=file type=', item.type, 'getAsFile()=', file ? `${file.name} (${file.type}, ${file.size}b)` : 'null');
      if (file && isImageFile(file)) files.push(file);
    }
    console.log('[imageFilesFromItems] returning', files.length, 'files');
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
        const b64 = img.data; // already base64 from Rust
        // Create blob URL from decoded bytes — renders instantly in Vditor
        const blob = new Blob([base64ToBytes(b64)], { type: 'image/png' });
        const blobUrl = URL.createObjectURL(blob);
        vditor!.insertValue(`![image](${blobUrl})`);

        const digest = await api.insertImage(b64);
        imageMap.set(digest, b64);
        blobToDigest.set(blobUrl, digest);
        digestToBlob.set(digest, blobUrl);
      }
      const stored = toStorage(vditor!.getValue());
      if (stored !== prevStoredContent) {
        prevStoredContent = stored;
        await onContentChange(stored);
      }
    } catch (_) {}
  }

  // Process image URIs from backend (drop or clipboard fallback for Linux)
  async function processImageUris(uris: string[]) {
    try {
      console.log('[processImageUris] input URIs:', uris);
      const imageUris = uris.filter(u => /\.(png|jpg|jpeg|gif|webp|bmp)$/i.test(u));
      console.log('[processImageUris] filtered image URIs:', imageUris);
      if (!imageUris.length) return;
      const images = await api.readImageFiles(imageUris);
      console.log('[processImageUris] readImageFiles returned', images?.length ?? 0, 'images');
      if (!images?.length) return;
      for (const img of images) {
        const b64 = img.data;
        const blob = new Blob([base64ToBytes(b64)], { type: 'image/png' });
        const blobUrl = URL.createObjectURL(blob);
        vditor!.insertValue(`![image](${blobUrl})`);

        const digest = await api.insertImage(b64);
        imageMap.set(digest, b64);
        blobToDigest.set(blobUrl, digest);
        digestToBlob.set(digest, blobUrl);
      }
      const stored = toStorage(vditor!.getValue());
      if (stored !== prevStoredContent) {
        prevStoredContent = stored;
        await onContentChange(stored);
      }
    } catch (_) {}
  }

  // Use capture phase to intercept drop/paste BEFORE Vditor's handlers.
  // Listen on document for dragover/drop to guarantee we fire before Vditor-internal handlers.
  function setupImageHandlers(el: HTMLElement) {
    const isInside = (e: Event) => el.contains(e.target as Node);

    const dragOver = (e: DragEvent) => {
      if (!isInside(e)) {
        return;
      }
      const data = e.dataTransfer;
      if (!data) return;
      console.log('[dragOver] types:', [...data.types], 'items:', data.items?.length ?? 0, 'files:', data.files?.length ?? 0);
      // Accept all drops over the editor — we inspect types in the drop handler.
      e.preventDefault();
      e.stopPropagation();
      data.dropEffect = 'copy';
    };

    const drop = (e: DragEvent) => {
      if (!isInside(e)) {
        return;
      }
      const data = e.dataTransfer;
      if (!data) return;

      console.log('[drop] types:', [...data.types], 'items:', data.items?.length ?? 0, 'files:', data.files?.length ?? 0);
      e.preventDefault();
      e.stopPropagation();

      // 1) Try File objects (kind=file items)
      const files = imageFilesFromTransfer(data);
      console.log('[drop] imageFilesFromTransfer returned', files.length, 'files');
      if (files.length > 0) {
        void processImageFiles(files);
        return;
      }

      // 2) Try getData first (works for text/html on WebKitGTK), then getAsString
      void (async () => {
        const extractUriFromHtml = (html: string): string | null => {
          // href="file://..." (standard link)
          for (const re of [/href="(file:\/\/[^"]+)"/i, /href='(file:\/\/[^']+)'/i]) {
            const m = html.match(re);
            if (m) return m[1];
          }
          // Nautilus/GNOME: <a style="...">file:///path</a> (URI as text content)
          const m = html.match(/>(file:\/\/[^<]+)<\/a>/i);
          return m ? m[1] : null;
        };

        const tryExtractUris = (text: string, source: string): string[] => {
          if (!text) return [];
          // Direct URI-list format
          let uris = text.split(/[\r\n]+/).map(s => s.trim()).filter(s =>
            s.startsWith('file://') && /\.(png|jpg|jpeg|gif|webp|bmp)$/i.test(s)
          );
          // Try extracting from HTML
          if (!uris.length) {
            const uri = extractUriFromHtml(text);
            if (uri && /\.(png|jpg|jpeg|gif|webp|bmp)$/i.test(uri)) uris = [uri];
          }
          if (uris.length > 0) console.log(`[drop] found URIs via ${source}:`, uris);
          return uris;
        };

        // First pass: try getData (works for text/html in WebKitGTK)
        for (const type of data.types) {
          if (type.includes('Files')) continue;
          try {
            const text = data.getData(type);
            console.log(`[drop] getData("${type}") len=${text?.length ?? 0}:`, text ? `head=[${text.substring(0, 150)}] tail=[${text.substring(Math.max(0, text.length - 200))}]` : '(empty)');
            const uris = tryExtractUris(text, `getData(${type})`);
            if (uris.length > 0) {
              void processImageUris(uris);
              return;
            }
          } catch (err) {
            console.log(`[drop] getData("${type}") threw:`, err);
          }
        }

        // Second pass: getAsString with timeout for stubborn types
        if (data.items) {
          for (let i = 0; i < data.items.length; i++) {
            const item = data.items[i];
            if (item.kind !== 'string') continue;
            console.log(`[drop] item[${i}]: kind=${item.kind} type="${item.type}"`);
            try {
              const text = await new Promise<string>((resolve) => {
                let settled = false;
                const timer = setTimeout(() => { if (!settled) { settled = true; resolve(''); } }, 200);
                item.getAsString((s) => { if (!settled) { settled = true; clearTimeout(timer); resolve(s); } });
              });
              console.log(`[drop] getAsString("${item.type}") len=${text?.length ?? 0}:`, text ? `head=[${text.substring(0, 150)}] tail=[${text.substring(Math.max(0, text.length - 200))}]` : '(empty/timeout)');
              if (!text) continue;
              const uris = tryExtractUris(text, `getAsString(${item.type})`);
              if (uris.length > 0) {
                void processImageUris(uris);
                return;
              }
            } catch (err) {
              console.log(`[drop] getAsString threw:`, err);
            }
          }
        }
        console.log('[drop] no image data found');
      })();
    };

    document.addEventListener('dragover', dragOver, true);
    document.addEventListener('drop', drop, true);

    // Cleanup on component destroy
    const cleanup = () => {
      document.removeEventListener('dragover', dragOver, true);
      document.removeEventListener('drop', drop, true);
    };

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

    return cleanup;
  }

  let handlersCleanup: (() => void) | undefined;

  onMount(async () => {
    if (!editorRoot) return;
    handlersCleanup = setupImageHandlers(editorRoot);
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

  onDestroy(() => {
    handlersCleanup?.();
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
