<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '../lib/tauri';
  import type { ImageInfo } from '../types';

  let images = $state<ImageInfo[]>([]);
  let referencedDigests = $state<Set<string>>(new Set());
  let loading = $state(true);
  let copiedDigest = $state<string | null>(null);

  // Delete confirmation
  let deleteTarget = $state<{ digest: string; refs: string[] } | null>(null);
  let showCleanupConfirm = $state(false);
  let unusedCount = $state(0);

  async function loadData() {
    loading = true;
    try {
      const [imgs, refs] = await Promise.all([
        api.getImages(),
        api.findReferencedDigests(),
      ]);
      images = imgs;
      referencedDigests = new Set(refs.map(d => d.toLowerCase()));
      // Count unused
      unusedCount = imgs.filter(i => !referencedDigests.has(i.digest.toLowerCase())).length;
    } catch (_) {
      images = [];
    }
    loading = false;
  }

  async function copyReference(digest: string) {
    try {
      await navigator.clipboard.writeText(`![image](${digest})`);
      copiedDigest = digest;
      setTimeout(() => { if (copiedDigest === digest) copiedDigest = null; }, 2000);
    } catch {
      // Fallback for environments without clipboard API
      const el = document.createElement('textarea');
      el.value = `![image](${digest})`;
      document.body.appendChild(el);
      el.select();
      document.execCommand('copy');
      document.body.removeChild(el);
      copiedDigest = digest;
      setTimeout(() => { if (copiedDigest === digest) copiedDigest = null; }, 2000);
    }
  }

  function confirmDelete(digest: string) {
    const refs = Array.from(referencedDigests);
    const lower = digest.toLowerCase();
    // Find which passages reference this image (we don't have passage info here,
    // so just mark it as referenced)
    deleteTarget = { digest, refs: refs.includes(lower) ? ['referenced'] : [] };
  }

  async function doDelete() {
    if (!deleteTarget) return;
    const digest = deleteTarget.digest;
    try {
      await api.deleteImage(digest);
      images = images.filter(i => i.digest !== digest);
      deleteTarget = null;
      // Refresh reference data after deletion
      const refs = await api.findReferencedDigests();
      referencedDigests = new Set(refs.map(d => d.toLowerCase()));
      unusedCount = images.filter(i => !referencedDigests.has(i.digest.toLowerCase())).length;
    } catch (_) {}
  }

  async function cleanupUnused() {
    showCleanupConfirm = false;
    const unused = images.filter(i => !referencedDigests.has(i.digest.toLowerCase()));
    for (const img of unused) {
      try { await api.deleteImage(img.digest); } catch (_) {}
    }
    await loadData();
  }

  function formatSize(bytes: number | undefined): string {
    if (!bytes) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="gallery-container">
  <div class="gallery-header">
    <h2 class="gallery-title">Image Gallery</h2>
    <div class="header-actions">
      <span class="count-info">{images.length} images ({unusedCount} unused)</span>
      {#if unusedCount > 0}
        <button class="btn-cleanup" onclick={() => showCleanupConfirm = true}>
          Clean up unused
        </button>
      {/if}
      <button class="btn-refresh" onclick={loadData}>
        <span class="material-icons icon">refresh</span>
      </button>
    </div>
  </div>

  {#if loading}
    <div class="loading">
      <p>Loading...</p>
    </div>
  {:else if images.length === 0}
    <div class="empty">
      <p>No images in this file.</p>
      <p class="hint">Paste or drag images into the editor to add them.</p>
    </div>
  {:else}
    <div class="image-grid">
      {#each images as img (img.digest)}
        {@const isRef = referencedDigests.has(img.digest.toLowerCase())}
        <div class="image-card" class:referenced={isRef}>
          <div class="thumbnail-wrap">
            {#if img.data}
              <img
                class="thumbnail"
                src="data:image/png;base64,{img.data}"
                alt={img.digest}
                loading="lazy"
              />
            {:else}
              <div class="no-thumb">[no data]</div>
            {/if}
          </div>
          <div class="card-info">
            <div class="digest-row">
              <code class="digest" title={img.digest}>{img.digest.substring(0, 8)}…{img.digest.substring(56)}</code>
              <button
                class="btn-copy"
                title="Copy reference"
                onclick={() => copyReference(img.digest)}
              >
                {copiedDigest === img.digest ? '✓ Copied!' : 'Copy'}
              </button>
            </div>
            {#if isRef}
              <span class="badge-ref" title="Referenced in passages">Used</span>
            {:else}
              <span class="badge-unused">Unused</span>
            {/if}
            <button
              class="btn-delete"
              title="Delete image"
              onclick={() => confirmDelete(img.digest)}
            >Delete</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Delete confirmation dialog -->
{#if deleteTarget}
  <div class="dialog-overlay" onclick={() => deleteTarget = null}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      {#if deleteTarget.refs.length > 0}
        <p class="dialog-warning">⚠ This image is referenced in your passages.</p>
        <p class="dialog-hint">Deleting it will leave broken image links.</p>
      {/if}
      <p class="dialog-title">Delete this image?</p>
      <code class="dialog-digest">{deleteTarget.digest.substring(0, 12)}…</code>
      <div class="dialog-actions">
        <button class="btn-danger" onclick={doDelete}>Delete</button>
        <button onclick={() => deleteTarget = null}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<!-- Clean up confirmation -->
{#if showCleanupConfirm}
  <div class="dialog-overlay" onclick={() => showCleanupConfirm = false}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <p class="dialog-title">Delete all unused images?</p>
      <p class="dialog-hint">{unusedCount} image(s) will be permanently removed.</p>
      <div class="dialog-actions">
        <button class="btn-danger" onclick={cleanupUnused}>Delete all</button>
        <button onclick={() => showCleanupConfirm = false}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .gallery-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: var(--spacing-lg);
  }

  .gallery-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-lg);
    flex-shrink: 0;
  }

  .gallery-title {
    font-size: var(--font-size-base);
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .count-info {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .btn-cleanup {
    padding: 6px 12px;
    border: 1px solid var(--border-color);
    background: var(--bg-card);
    color: var(--warning-color, #e6a817);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s;
  }

  .btn-cleanup:hover {
    background: var(--bg-hover);
  }

  .btn-refresh {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-refresh:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .loading, .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .hint {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
    margin-top: var(--spacing-sm);
  }

  .image-grid {
    flex: 1;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--spacing-md);
    align-content: start;
  }

  .image-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
    transition: border-color 0.15s;
    display: flex;
    flex-direction: column;
  }

  .image-card:hover {
    border-color: var(--accent-color);
  }

  .image-card.referenced {
    border-color: var(--success-color, #4caf50);
  }

  .thumbnail-wrap {
    aspect-ratio: 1;
    overflow: hidden;
    background: var(--bg-hover);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumbnail {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .no-thumb {
    color: var(--text-faint);
    font-size: var(--font-size-sm);
  }

  .card-info {
    padding: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .digest-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .digest {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--bg-hover);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .btn-copy {
    padding: 2px 8px;
    border: 1px solid var(--border-color);
    background: var(--bg-card);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 11px;
    white-space: nowrap;
    transition: all 0.15s;
  }

  .btn-copy:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .badge-ref {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
    background: rgba(76, 175, 80, 0.15);
    color: var(--success-color, #4caf50);
    display: inline-block;
    width: fit-content;
  }

  .badge-unused {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
    background: var(--bg-hover);
    color: var(--text-faint);
    display: inline-block;
    width: fit-content;
  }

  .btn-delete {
    padding: 2px 8px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s;
    align-self: flex-end;
    margin-top: 4px;
  }

  .btn-delete:hover {
    background: rgba(248, 71, 71, 0.1);
    color: var(--danger-color);
  }

  /* Dialog overlay */
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(2px);
  }

  .dialog {
    background: var(--bg-modal);
    border-radius: var(--radius-md);
    padding: var(--spacing-xl);
    max-width: 360px;
    width: 90%;
    text-align: center;
    box-shadow: var(--shadow-lg);
  }

  .dialog-warning {
    color: var(--warning-color, #e6a817);
    font-size: var(--font-size-sm);
    margin-bottom: var(--spacing-sm);
    font-weight: 600;
  }

  .dialog-title {
    font-size: var(--font-size-base);
    color: var(--text-primary);
    margin-bottom: var(--spacing-sm);
  }

  .dialog-hint {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-bottom: var(--spacing-md);
  }

  .dialog-digest {
    display: block;
    font-size: 12px;
    color: var(--text-muted);
    background: var(--bg-hover);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-md);
  }

  .dialog-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: center;
  }

  .dialog-actions button {
    padding: 6px 16px;
    border: none;
    background: var(--bg-hover);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s;
  }

  .dialog-actions button:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .btn-danger {
    background: var(--danger-color) !important;
    color: var(--text-inverse) !important;
  }

  .btn-danger:hover {
    opacity: 0.9 !important;
  }
</style>
