<script lang="ts">
  let {
    title,
    placeholder,
    onSubmit,
    onCancel
  }: {
    title: string;
    placeholder: string;
    onSubmit: (name: string) => void;
    onCancel: () => void;
  } = $props();

  let name = $state('');

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (name.trim()) {
        onSubmit(name.trim());
      }
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      onCancel();
    }
  }

  function handleSubmit() {
    if (name.trim()) {
      onSubmit(name.trim());
    } else {
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
    <p class="dialog-title">{title}</p>
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="text"
      bind:value={name}
      placeholder={placeholder}
      autofocus
    />
    <div class="dialog-actions">
      <button class="btn-cancel" onclick={onCancel}>Cancel</button>
      <button class="btn-create" onclick={handleSubmit}>Create</button>
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
    min-width: 280px;
    box-shadow: var(--shadow-lg);
  }

  .dialog-title {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    margin-bottom: var(--spacing-md);
    font-weight: 500;
  }

  input {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    margin-bottom: var(--spacing-md);
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

  .btn-create {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .btn-create:hover {
    opacity: 0.9;
  }
</style>