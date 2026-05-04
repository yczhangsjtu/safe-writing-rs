<script lang="ts">
  let {
    oldPassword = '',
    onSubmit,
    onCancel
  }: {
    oldPassword?: string;
    onSubmit: (oldPassword: string, newPassword: string) => void;
    onCancel: () => void;
  } = $props();

  let newPassword = $state('');
  let confirmPassword = $state('');
  let errorMsg = $state('');

  function handleSubmit() {
    errorMsg = '';

    if (!oldPassword && passwordInput) {
      errorMsg = 'Old password required';
      return;
    }

    if (!newPassword) {
      errorMsg = 'New password required';
      return;
    }

    if (newPassword !== confirmPassword) {
      errorMsg = 'Passwords do not match';
      return;
    }

    onSubmit(oldPassword || passwordInput, newPassword);
  }

  // For when oldPassword is not provided (e.g., changing password after file is already open)
  let passwordInput = $state('');

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
    <h2>Change Password</h2>

    {#if errorMsg}
      <p class="error">{errorMsg}</p>
    {/if}

    <div class="form">
      {#if !oldPassword}
        <label>
          <span>Old Password</span>
          <input type="password" bind:value={passwordInput} placeholder="Enter old password" />
        </label>
      {/if}

      <label>
        <span>New Password</span>
        <input type="password" bind:value={newPassword} placeholder="Enter new password" />
      </label>

      <label>
        <span>Confirm New Password</span>
        <input type="password" bind:value={confirmPassword} placeholder="Confirm new password" />
      </label>

      <div class="actions">
        <button class="btn-primary" onclick={handleSubmit}>Change</button>
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
    font-size: var(--font-size-xs);
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
    font-size: var(--font-size-xs);
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
    font-size: var(--font-size-xs);
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
</style>