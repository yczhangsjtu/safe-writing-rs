<script lang="ts">
  let {
    message,
    confirmText = 'Save',
    cancelText = 'Discard',
    onConfirm,
    onCancel,
    onDiscard
  }: {
    message: string;
    confirmText?: string;
    cancelText?: string;
    onConfirm: () => void;
    onCancel: () => void;
    onDiscard?: () => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
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
    <p class="dialog-message">{message}</p>
    <div class="dialog-actions">
      <button class="btn-cancel" onclick={onCancel}>Cancel</button>
      {#if onDiscard}
        <button class="btn-discard" onclick={onDiscard}>{cancelText}</button>
      {/if}
      <button class="btn-confirm" onclick={onConfirm}>{confirmText}</button>
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
    min-width: 300px;
    box-shadow: var(--shadow-lg);
  }

  .dialog-message {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    margin-bottom: var(--spacing-md);
    line-height: 1.5;
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

  .btn-discard {
    background: var(--danger-color);
    color: var(--text-inverse);
  }

  .btn-discard:hover {
    opacity: 0.9;
  }

  .btn-confirm {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .btn-confirm:hover {
    opacity: 0.9;
  }
</style>