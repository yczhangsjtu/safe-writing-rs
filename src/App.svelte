<script lang="ts">
  import { onMount } from 'svelte';
  import { theme, files, currentFile, passages, currentPassageIndex, config, isLoading, error, success, copilotVisible, isDirty, copilotSettings } from './lib/stores';
  import * as api from './lib/tauri';
  import { listen } from '@tauri-apps/api/event';
  import Sidebar from './components/Sidebar.svelte';
  import Editor from './components/Editor.svelte';
  import PasswordDialog from './components/PasswordDialog.svelte';
  import ChangePasswordDialog from './components/ChangePasswordDialog.svelte';
  import CopilotPanel from './components/CopilotPanel.svelte';
  import ThemeToggle from './components/ThemeToggle.svelte';

  let showPasswordDialog = $state(false);
  let passwordDialogMode: 'new' | 'decrypt' = $state('decrypt');
  let pendingFilename = $state('');
  let pendingCiphertext = $state('');
  let initialized = $state(false);
  let initError = $state('');
  let showChangePasswordDialog = $state(false);
  let lastCurrentFile = $state<string | null>(null);

  onMount(async () => {
    console.log('App mounted, initializing...');
    try {
      // Load config
      const cfg = await api.getConfig();
      config.set(cfg);
      theme.set(cfg.theme as 'light' | 'dark');
      document.documentElement.setAttribute('data-theme', cfg.theme);

      // Load files list
      const fileList = await api.listFiles();
      files.set(fileList);
      initialized = true;
      console.log('Initialization complete');
    } catch (e: any) {
      initError = e?.message || String(e);
      console.error('Init error:', e);
    }

    // Listen for state-changed events from backend
    const unlistenState = listen<api.AppStateResponse>('state-changed', (event) => {
      const state = event.payload;

      // Check if file actually changed (opened/closed)
      const fileChanged = (lastCurrentFile === null && state.current_file !== null) ||
                          (lastCurrentFile !== null && state.current_file === null);

      currentFile.set(state.current_file);
      lastCurrentFile = state.current_file;
      passages.set(state.passages);
      currentPassageIndex.set(state.current_passage_index);
      isDirty.set(state.is_dirty);

      // Update copilot persistent settings (preserve runtime messages)
      copilotSettings.update(s => {
        s.system_prompt = state.copilot_settings.system_prompt;
        s.buffers = state.copilot_settings.buffers;
        s.favorite_prompts = state.copilot_settings.favorite_prompts;
        // Only reset messages when file changes
        if (fileChanged) {
          s.messages = [];
        }
        return s;
      });
    });

    return async () => {
      (await unlistenState)();
    };
  });

  async function handleFileSelect(filename: string) {
    // Close current file first
    if ($currentFile) {
      await api.closeFile();
    }

    try {
      isLoading.set(true);

      // Check if this is a new file
      const isNewFile = !$files.includes(filename);

      if (isNewFile) {
        passwordDialogMode = 'new';
        pendingFilename = filename;
        pendingCiphertext = '';
        showPasswordDialog = true;
      } else {
        const result = await api.openFile(filename);
        if (result.is_new) {
          passwordDialogMode = 'new';
          pendingFilename = filename;
          showPasswordDialog = true;
        } else if (result.ciphertext) {
          passwordDialogMode = 'decrypt';
          pendingFilename = filename;
          pendingCiphertext = result.ciphertext;
          showPasswordDialog = true;
        }
      }
    } catch (e: any) {
      error.set(`Failed to open file: ${e?.message || e}`);
    } finally {
      isLoading.set(false);
    }
  }

  async function handlePasswordSubmit(password: string) {
    try {
      isLoading.set(true);
      showPasswordDialog = false;

      if (passwordDialogMode === 'new') {
        await api.createFile(pendingFilename, password);
        // Refresh file list
        const fileList = await api.listFiles();
        files.set(fileList);
        // Decrypt (will trigger state-changed event)
        await api.decryptFile(pendingFilename, '', password);
      } else {
        await api.decryptFile(pendingFilename, pendingCiphertext, password);
      }

      success.set('File opened successfully');
      setTimeout(() => success.set(null), 3000);
    } catch (e: any) {
      error.set(`Failed: ${e?.message || e}`);
    } finally {
      isLoading.set(false);
    }
  }

  async function handleSave() {
    try {
      isLoading.set(true);
      await api.encryptAndSave(); // Will trigger state-changed event
      success.set('Saved successfully');
      setTimeout(() => success.set(null), 3000);
    } catch (e: any) {
      error.set(`Failed to save: ${e?.message || e}`);
    } finally {
      isLoading.set(false);
    }
  }

  async function handleLock() {
    await api.closeFile(); // Will trigger state-changed event
  }

  function handleThemeChange(newTheme: 'light' | 'dark') {
    theme.set(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    api.updateConfig({ theme: newTheme });
  }

  function handleChangePassword() {
    showChangePasswordDialog = true;
  }

  async function handlePasswordChange(oldPassword: string, newPassword: string) {
    try {
      isLoading.set(true);
      showChangePasswordDialog = false;
      await api.changePassword(oldPassword, newPassword);
      success.set('Password changed successfully');
      setTimeout(() => success.set(null), 3000);
    } catch (e: any) {
      error.set(`Failed to change password: ${e?.message || e}`);
    } finally {
      isLoading.set(false);
    }
  }
</script>

<div class="app-container">
  {#if !initialized}
    <div class="loading-screen">
      {#if initError}
        <p class="error-text">{initError}</p>
      {:else}
        <p class="loading-text">Loading...</p>
      {/if}
    </div>
  {:else}
    <header class="app-header">
      <div class="header-left">
        <span class="app-title">Safe Writing</span>
        {#if $config.data_dir}
          <span class="data-dir">{ $config.data_dir }</span>
        {/if}
      </div>
      <div class="header-right">
        <ThemeToggle currentTheme={$theme} onThemeChange={handleThemeChange} />
      </div>
    </header>

    <main class="app-main">
      <Sidebar
        filesProp={$files}
        currentFile={$currentFile}
        onFileSelect={handleFileSelect}
        onSave={handleSave}
        isDirtyProp={$isDirty}
        onChangePassword={handleChangePassword}
      />

      {#if $currentFile}
        <Editor
          passagesProp={$passages}
          currentIndex={$currentPassageIndex}
          isDirtyProp={$isDirty}
          onSave={handleSave}
          onLock={handleLock}
        />
      {:else}
        <div class="empty-state">
          <p>Select a file to begin</p>
        </div>
      {/if}

      {#if $copilotVisible && $currentFile}
        <CopilotPanel />
      {/if}
    </main>

    {#if showPasswordDialog}
      <PasswordDialog
        mode={passwordDialogMode}
        filename={pendingFilename}
        onSubmit={handlePasswordSubmit}
        onCancel={() => showPasswordDialog = false}
      />
    {/if}

    {#if showChangePasswordDialog}
      <ChangePasswordDialog
        onSubmit={handlePasswordChange}
        onCancel={() => showChangePasswordDialog = false}
      />
    {/if}

    {#if $isLoading}
      <div class="loading-overlay">
        <div class="spinner"></div>
      </div>
    {/if}

    {#if $error}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="error-toast" onclick={() => error.set(null)}>
        {$error}
      </div>
    {/if}

    {#if $success}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="success-toast" onclick={() => success.set(null)}>
        {$success}
      </div>
    {/if}
  {/if}
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .loading-screen {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
  }

  .loading-text {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .error-text {
    color: var(--danger-color);
    font-size: var(--font-size-sm);
  }

  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--bg-sidebar);
    border-bottom: 1px solid var(--border-color-faint);
    height: 40px;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .app-title {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .data-dir {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
  }

  .header-right {
    display: flex;
    gap: var(--spacing-sm);
  }

  .app-main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-editor);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .loading-overlay {
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
    backdrop-filter: blur(2px);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent-color);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-toast, .success-toast {
    position: fixed;
    bottom: var(--spacing-lg);
    right: var(--spacing-lg);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-modal);
    border-radius: var(--radius-md);
    cursor: pointer;
    z-index: 1000;
    font-size: var(--font-size-xs);
    box-shadow: var(--shadow-md);
    animation: slideIn 0.2s ease;
  }

  .error-toast {
    border: 1px solid var(--danger-color);
    color: var(--danger-color);
  }

  .success-toast {
    border: 1px solid var(--success-color);
    color: var(--success-color);
  }
</style>