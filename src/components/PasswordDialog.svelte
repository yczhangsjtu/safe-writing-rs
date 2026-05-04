<script lang="ts">
  let {
    mode,
    filename,
    onSubmit,
    onCancel,
    onChangePassword
  }: {
    mode: 'new' | 'decrypt';
    filename: string;
    onSubmit: (password: string) => void;
    onCancel: () => void;
    onChangePassword?: () => void;
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
        <!-- svelte-ignore a11y_autofocus -->
        <input type="password" bind:value={password} placeholder="Enter password" autofocus />
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

      {#if mode === 'decrypt' && onChangePassword}
        <button class="btn-change-password" onclick={onChangePassword}>
          Change Password
        </button>
      {/if}
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
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .dialog {
    background: var(--bg-modal);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xl);
    min-width: 280px;
    max-width: 360px;
    box-shadow: var(--shadow-lg);
    animation: slideIn 0.2s ease;
  }

  .dialog h2 {
    margin-bottom: var(--spacing-lg);
    text-align: center;
    color: var(--text-primary);
    font-size: var(--font-size-lg);
    font-weight: 500;
  }

  .error {
    margin-bottom: var(--spacing-md);
    padding: var(--spacing-sm);
    background: rgba(220, 38, 38, 0.1);
    border-radius: var(--radius-sm);
    text-align: center;
    color: var(--danger-color);
    font-size: var(--font-size-sm);
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  label span {
    font-size: var(--font-size-sm);
    color: var(--text-faint);
  }

  input {
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

  .actions {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-lg);
  }

  .actions button {
    flex: 1;
    padding: var(--spacing-sm);
    border: none;
    background: var(--bg-hover);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .actions button:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .btn-primary {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .btn-primary:hover {
    background: var(--accent-color-hover);
    color: var(--text-inverse);
  }

  .btn-change-password {
    width: 100%;
    padding: var(--spacing-sm);
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
    text-align: center;
  }

  .btn-change-password:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
</style>