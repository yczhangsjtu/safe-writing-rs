<script lang="ts">
  import { onMount } from 'svelte';
  import { Editor, rootCtx, defaultValueCtx } from '@milkdown/kit/core';
  import { commonmark } from '@milkdown/kit/preset/commonmark';
  import { gfm } from '@milkdown/kit/preset/gfm';
  import { history } from '@milkdown/kit/plugin/history';
  import { listener, listenerCtx } from '@milkdown/kit/plugin/listener';
  import { clipboard } from '@milkdown/kit/plugin/clipboard';
  import { trailing } from '@milkdown/kit/plugin/trailing';
  import { upload, uploadConfig } from '@milkdown/kit/plugin/upload';
  import { replaceAll } from '@milkdown/kit/utils';
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
  let editor: Editor | undefined;
  let editorFocused = $state(false);
  let initialized = $state(false);
  let prevContentDigests = $state('');
  let imageMap: Map<string, string> = new Map();
  let processing = $state(false);

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

  function digestsToDataUri(md: string): string {
    if (imageMap.size === 0) return md;
    return md.replace(
      /!\[([^\]]*)\]\(([a-fA-F0-9]{64})\)/g,
      (m, alt, digest) => {
        const b64 = imageMap.get(digest.toLowerCase());
        return b64 ? `![${alt}](data:image/png;base64,${b64})` : m;
      }
    );
  }

  function dataUriToDigests(md: string): string {
    return md.replace(
      /!\[([^\]]*)\]\(data:image\/[^;]+;base64,([A-Za-z0-9+/=]+)\)/g,
      (m, alt, b64) => {
        for (const [digest, data] of imageMap) {
          if (data === b64) return `![${alt}](${digest})`;
        }
        return m;
      }
    );
  }

  onMount(async () => {
    console.log('[MarkdownEditor] onMount, editorRoot:', editorRoot);
    if (!editorRoot) return;

    await refreshImageMap();
    console.log('[MarkdownEditor] images loaded:', imageMap.size);

    const displayContent = digestsToDataUri(content);
    prevContentDigests = content;

    try {
      editor = Editor.make()
        .config((ctx) => {
          ctx.set(rootCtx, editorRoot!);
          ctx.set(defaultValueCtx, displayContent);
          ctx.update(uploadConfig.key, (prev) => ({
            ...prev,
            uploader: async (files, schema) => {
              const imageType = schema.nodes['image'];
              if (!imageType) return [];
              const nodes = [];
              for (let i = 0; i < files.length; i++) {
                const file = files.item(i);
                if (!file || !file.type.startsWith('image/')) continue;
                try {
                  const buffer = await file.arrayBuffer();
                  const bytes = new Uint8Array(buffer);
                  const b64 = await fileToBase64(file);
                  const digest = await api.insertImage(Array.from(bytes));
                  imageMap.set(digest, b64);
                  const node = imageType.createAndFill({
                    src: `data:image/png;base64,${b64}`,
                    alt: '',
                    title: '',
                  });
                  if (node) nodes.push(node);
                } catch (e) {
                  console.error('[MarkdownEditor] upload failed:', e);
                }
              }
              return nodes;
            },
          }));
        })
        .use(commonmark)
        .use(gfm)
        .use(history)
        .use(clipboard)
        .use(listener)
        .use(trailing)
        .use(upload);

      editor.config((ctx) => {
        const mgr = ctx.get(listenerCtx);
        mgr.markdownUpdated((_ctx, markdown, prev) => {
          if (markdown === prev || processing) return;
          const forStorage = dataUriToDigests(markdown);
          if (forStorage !== prevContentDigests) {
            prevContentDigests = forStorage;
            onContentChange(forStorage);
          }
        });
        mgr.focus(() => { editorFocused = true; });
        mgr.blur(() => { editorFocused = false; });
      });

      await editor.create();
      console.log('[MarkdownEditor] created OK, root children:', editorRoot.children.length);
      initialized = true;
      applyFontStyles();
    } catch (e) {
      console.error('[MarkdownEditor] init error:', e);
    }
  });

  function applyFontStyles() {
    if (!editorRoot) return;
    const pm = editorRoot.querySelector('.ProseMirror') as HTMLElement;
    if (pm) {
      pm.style.fontSize = `${$config.font_size}px`;
      pm.style.fontFamily = "'LXGW WenKai', sans-serif";
    }
  }

  $effect(() => {
    if (!editor || !initialized) return;
    if (content === undefined || content === null) return;
    refreshImageMap().then(() => {
      if (content !== prevContentDigests && !editorFocused) {
        prevContentDigests = content;
        const displayContent = digestsToDataUri(content);
        processing = true;
        editor!.action(replaceAll(displayContent));
        processing = false;
      }
    });
  });

  $effect(() => {
    if ($config.font_size && initialized) applyFontStyles();
  });

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault();
      onSave();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div bind:this={editorRoot} class="markdown-editor"></div>

<style>
  .markdown-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .markdown-editor :global(.milkdown) {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .markdown-editor :global(.ProseMirror) {
    flex: 1;
    outline: none;
    overflow-y: auto;
    padding: 0;
    line-height: 1.8;
    word-wrap: break-word;
    white-space: pre-wrap;
  }

  :global(.ProseMirror p) {
    margin: 0 0 0.5em 0;
  }

  .markdown-editor :global(.ProseMirror img) {
    max-width: 100%;
    border-radius: var(--radius-sm);
  }
</style>
