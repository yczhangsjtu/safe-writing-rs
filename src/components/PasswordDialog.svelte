<script lang="ts">
  let {
    mode,
    filename,
    onSubmit,
    onCancel
  }: {
    mode: 'new' | 'decrypt';
    filename: string;
    onSubmit: (password: string) => void;
    onCancel: () => void;
  } = $props();

  let password = $state('');
  let confirmPassword = $state('');
  let errorMsg = $state('');

  function handleSubmit() {
    errorMsg = '';

    if (!password) {
      errorMsg = 'Password required';
      return;
    }

    if (mode === 'new') {
      if (password !== confirmPassword) {
        errorMsg = 'Passwords do not match';
        return;
      }
    }

    onSubmit(password);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSubmit();
    }
    if (e.key === 'Escape') {
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
    <h2>
      {#if mode === 'new'}
        Create New File
      {:else}
        Unlock: {filename}
      {/if}
    </h2>

    {#if errorMsg}
      <p class="error">{errorMsg}</p>
    {/if}

    <div class="form">
      <label>
        <span>Password</span>
        <input type="password" bind:value={password} placeholder="Enter password" />
      </label>

      {#if mode === 'new'}
        <label>
          <span>Confirm Password</span>
          <input type="password" bind:value={confirmPassword} placeholder="Confirm password" />
        </label>
      {/if}

      <div class="actions">
        <button class="btn-primary" onclick={handleSubmit}>
          {#if mode === 'new'}Create{:else}Unlock{/if}
        </button>
        <button onclick={onCancel}>Cancel</button>
      </div>
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
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--bg-primary);
    border-radius: 12px;
    padding: 24px;
    min-width: 300px;
    max-width: 400px;
  }

  .dialog h2 {
    margin-bottom: 16px;
    text-align: center;
    color: var(--text-primary);
  }

  .error {
    margin-bottom: 12px;
    padding: 8px;
    background: rgba(231, 76, 60, 0.1);
    border-radius: 4px;
    text-align: center;
    color: var(--danger-color);
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  label span {
    font-size: 14px;
    color: var(--text-secondary);
  }

  input {
    padding: 10px 12px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: 4px;
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }

  .actions button {
    flex: 1;
    padding: 10px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-primary {
    background: var(--accent-color);
    color: white;
  }

  .actions button:hover {
    opacity: 0.9;
  }
</style>