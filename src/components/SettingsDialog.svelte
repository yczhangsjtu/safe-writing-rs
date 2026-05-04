<script lang="ts">
  let {
    dataDir,
    onSubmit,
    onCancel
  }: {
    dataDir: string;
    onSubmit: (newDir: string) => void;
    onCancel: () => void;
  } = $props();

  let newDir = $state('');

  $effect(() => {
    newDir = dataDir;
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (newDir.trim()) {
        onSubmit(newDir.trim());
      }
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="dialog-overlay" onclick={onCancel}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <p class="dialog-title">Settings</p>
    <label>
      <span class="label-text">Working Directory</span>
      <input
        type="text"
        bind:value={newDir}
        placeholder="Enter directory path..."
      />
    </label>
    <div class="dialog-actions">
      <button class="btn-cancel" onclick={onCancel}>Cancel</button>
      <button class="btn-save" onclick={() => onSubmit(newDir.trim())}>Save</button>
    </div>
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
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
    padding: var(--spacing-lg);
    min-width: 400px;
    box-shadow: var(--shadow-lg);
  }

  .dialog-title {
    font-size: var(--font-size-md);
    color: var(--text-primary);
    margin-bottom: var(--spacing-md);
    font-weight: 500;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  .label-text {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  input {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
  }

  input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .dialog-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
  }

  .dialog-actions button {
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    transition: all 0.15s ease;
  }

  .btn-cancel {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .btn-cancel:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .btn-save {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .btn-save:hover {
    opacity: 0.9;
  }
</style>