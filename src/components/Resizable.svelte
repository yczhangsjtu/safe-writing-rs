<script lang="ts">
  import { onMount } from 'svelte';

  let {
    width,
    minWidth = 100,
    maxWidth = 500,
    side = 'right',
    onResize,
    onSave
  }: {
    width: number;
    minWidth?: number;
    maxWidth?: number;
    side?: 'left' | 'right';
    onResize?: (width: number) => void;
    onSave?: (width: number) => void;
  } = $props();

  let isDragging = false;
  let startX = 0;
  let startWidth = 0;
  let dragWidth = width; // Track the actual width during drag

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDragging = true;
    startX = e.clientX;
    startWidth = width;
    dragWidth = width;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    const delta = side === 'right' ? e.clientX - startX : startX - e.clientX;
    const newWidth = Math.min(maxWidth, Math.max(minWidth, startWidth + delta));
    dragWidth = newWidth;
    onResize?.(newWidth);
  }

  function handleMouseUp() {
    if (isDragging) {
      isDragging = false;
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      onSave?.(dragWidth);
    }
  }

  onMount(() => {
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  });
</script>

<div class="resize-handle" class:active={isDragging} onmousedown={handleMouseDown} style="left: {side === 'left' ? 0 : 'auto'}; right: {side === 'right' ? 0 : 'auto'};"></div>

<style>
  .resize-handle {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 4px;
    cursor: col-resize;
    z-index: 100;
    transition: background-color 0.15s ease;
  }

  .resize-handle:hover,
  .resize-handle.active {
    background: var(--accent-color);
  }
</style>