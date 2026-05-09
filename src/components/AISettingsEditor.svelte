<script lang="ts">
  import { copilotSettings } from '../lib/stores';
  import * as api from '../lib/tauri';
  import type { FavoritePrompt } from '../types';

  interface Props {}

  let {}: Props = $props();

  // ===== Local state (editing buffer, same pattern as Editor.svelte) =====
  let localSystemPrompt = $state('');
  let localBuffers = $state<string[]>(Array(10).fill(''));
  let localFavorites = $state<FavoritePrompt[]>([]);
  let dirty = $state(false);

  // Snapshot of store values at last load — used to detect which fields the user edited
  let snapshotSystemPrompt = $state('');
  let snapshotBuffers = $state<string[]>(Array(10).fill(''));
  let snapshotFavorites = $state<FavoritePrompt[]>([]);

  function takeSnapshot(s: typeof $copilotSettings) {
    snapshotSystemPrompt = s.system_prompt;
    snapshotBuffers = [...s.buffers];
    snapshotFavorites = s.favorite_prompts.map(f => ({ name: f.name, prompt: f.prompt }));
  }

  // Reload from store only when no local edits are in flight
  $effect(() => {
    const s = $copilotSettings;
    if (!dirty) {
      localSystemPrompt = s.system_prompt;
      localBuffers = [...s.buffers];
      localFavorites = s.favorite_prompts.map(f => ({ name: f.name, prompt: f.prompt }));
      takeSnapshot(s);
    }
  });

  // ===== Edit handlers — each calls syncToBackend immediately =====
  function markDirty() {
    dirty = true;
  }

  function handleSystemPromptChange(e: Event) {
    localSystemPrompt = (e.target as HTMLTextAreaElement).value;
    markDirty();
    syncToBackend();
  }

  function handleBufferChange(index: number, value: string) {
    localBuffers[index] = value.slice(0, 200);
    markDirty();
    syncToBackend();
  }

  function handleClearBuffer(index: number) {
    localBuffers[index] = '';
    markDirty();
    syncToBackend();
  }

  function handleAddFavorite() {
    localFavorites = [...localFavorites, { name: '', prompt: '' }];
    markDirty();
    syncToBackend();
  }

  function handleFavoriteNameChange(index: number, value: string) {
    localFavorites[index].name = value;
    markDirty();
    syncToBackend();
  }

  function handleFavoritePromptChange(index: number, value: string) {
    localFavorites[index].prompt = value;
    markDirty();
    syncToBackend();
  }

  function handleRemoveFavorite(index: number) {
    localFavorites = localFavorites.filter((_, i) => i !== index);
    markDirty();
    syncToBackend();
  }

  // ===== Sync local state to backend — directly updates copilot_settings in memory =====
  // No XML round-trip; serialization only happens at file save time (encrypt_and_save).
  async function syncToBackend() {
    const store = $copilotSettings;

    // Merge: fields the user touched → use local; untouched → use store (preserves CopilotPanel changes)
    const finalSystemPrompt = localSystemPrompt !== snapshotSystemPrompt
      ? localSystemPrompt
      : store.system_prompt;

    const finalBuffers = localBuffers.map((buf, i) =>
      buf !== snapshotBuffers[i] ? buf : (store.buffers[i] || '')
    );

    const favsModified = localFavorites.length !== snapshotFavorites.length ||
      localFavorites.some((f, i) =>
        f.name !== snapshotFavorites[i]?.name || f.prompt !== snapshotFavorites[i]?.prompt
      );
    const finalFavorites = favsModified
      ? localFavorites
      : store.favorite_prompts.map(f => ({ name: f.name, prompt: f.prompt }));

    await api.updateCopilotSettings({
      system_prompt: finalSystemPrompt,
      buffers: finalBuffers,
      favorite_prompts: finalFavorites.map(f => ({ name: f.name, prompt: f.prompt })),
      messages: store.messages, // preserved by backend anyway
    });

    // Keep the store in sync so CopilotPanel sees changes immediately
    copilotSettings.update(s => ({
      ...s,
      system_prompt: finalSystemPrompt,
      buffers: [...finalBuffers],
      favorite_prompts: finalFavorites.map(f => ({ name: f.name, prompt: f.prompt })),
    }));

    dirty = false;
  }

  // Exported save — called by Editor's Cmd+S handler to flush before encryptAndSave
  async function doSave() {
    await syncToBackend();
  }
  export { doSave as save };
</script>

<div class="ai-settings-editor">
  <div class="settings-header">
    <h2 class="settings-title">AI Settings</h2>
    <p class="settings-subtitle">Configure system prompt, context buffers, and favorite prompts for the AI Copilot</p>
  </div>

  <!-- System Prompt Section -->
  <div class="settings-section">
    <div class="section-label">System Prompt</div>
    <textarea
      value={localSystemPrompt}
      oninput={handleSystemPromptChange}
      rows="4"
      placeholder="You are a helpful writing assistant."
      class="settings-textarea"
    ></textarea>
  </div>

  <!-- Buffers Section -->
  <div class="settings-section">
    <div class="section-label">Buffers <span class="hint">#0 – #9</span></div>
    <div class="buffer-grid">
      {#each localBuffers as buf, i}
        <div class="buffer-row">
          <span class="buffer-index">#{i}</span>
          <input
            type="text"
            value={buf}
            oninput={(e) => handleBufferChange(i, (e.target as HTMLInputElement).value)}
            placeholder="Buffer content..."
            class="settings-input buffer-input"
            maxlength="200"
          />
          {#if buf}
            <button
              class="btn-action danger"
              onclick={() => handleClearBuffer(i)}
              title="Clear"
            >×</button>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- Favorite Prompts Section -->
  <div class="settings-section">
    <div class="section-header-row">
      <span class="section-label">Favorite Prompts</span>
      <button class="btn-add" onclick={handleAddFavorite}>
        <span class="material-icons icon-sm">add</span> Add
      </button>
    </div>
    {#if localFavorites.length > 0}
      <div class="favorites-list">
        {#each localFavorites as fav, i}
          <div class="favorite-row">
            <div class="favorite-fields">
              <input
                type="text"
                value={fav.name}
                oninput={(e) => handleFavoriteNameChange(i, (e.target as HTMLInputElement).value)}
                placeholder="Short name"
                class="settings-input name-input"
              />
              <input
                type="text"
                value={fav.prompt}
                oninput={(e) => handleFavoritePromptChange(i, (e.target as HTMLInputElement).value)}
                placeholder="Prompt text..."
                class="settings-input prompt-input"
              />
            </div>
            <button
              class="btn-action danger"
              onclick={() => handleRemoveFavorite(i)}
              title="Remove"
            >×</button>
          </div>
        {/each}
      </div>
    {:else}
      <p class="empty-hint">No favorite prompts. Add one to quickly access it in the Copilot panel.</p>
    {/if}
  </div>
</div>

<style>
  .ai-settings-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: var(--spacing-xl);
    max-width: 700px;
    margin: 0 auto;
    width: 100%;
    gap: var(--spacing-lg);
  }

  .settings-header {
    margin-bottom: var(--spacing-sm);
  }

  .settings-title {
    font-size: 22px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 4px 0;
  }

  .settings-subtitle {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color-faint);
  }

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .section-header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .hint {
    font-size: 12px;
    color: var(--text-faint);
    font-weight: 400;
  }

  .settings-textarea {
    width: 100%;
    padding: var(--spacing-sm);
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    resize: vertical;
    font-size: var(--font-size-sm);
    line-height: 1.5;
    font-family: inherit;
  }

  .settings-textarea:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .settings-input {
    padding: 6px 10px;
    border: 1px solid var(--border-color-faint);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-family: inherit;
  }

  .settings-input:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .buffer-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .buffer-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .buffer-index {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
    width: 24px;
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .buffer-input {
    flex: 1;
  }

  .btn-action {
    width: 28px;
    height: 28px;
    border: none;
    background: var(--bg-hover);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    padding: 0;
    flex-shrink: 0;
    transition: all 0.15s ease;
  }

  .btn-action:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .btn-action.danger:hover {
    color: var(--danger-color);
  }

  .btn-add {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border: none;
    background: var(--bg-hover);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .btn-add:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .icon-sm {
    font-size: 14px;
    line-height: 1;
  }

  .favorites-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .favorite-row {
    display: flex;
    align-items: flex-start;
    gap: 6px;
  }

  .favorite-fields {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .name-input {
    width: 100%;
  }

  .prompt-input {
    width: 100%;
  }

  .empty-hint {
    font-size: var(--font-size-sm);
    color: var(--text-faint);
    margin: 0;
    padding: var(--spacing-sm) 0;
  }

  .material-icons {
    font-family: 'Material Icons';
    font-weight: normal;
    font-style: normal;
    display: inline-block;
    line-height: 1;
    text-transform: none;
    letter-spacing: normal;
    word-wrap: normal;
    white-space: nowrap;
    direction: ltr;
    -webkit-font-smoothing: antialiased;
  }
</style>