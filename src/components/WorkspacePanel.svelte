<script lang="ts">
  import { workspace } from '../lib/stores';
  import * as api from '../lib/tauri';
  import type { Character, Relationship, KeyValueEntry } from '../types';

  let selectedCharacter = $state<Character | null>(null);
  let selectedRelationship = $state<Relationship | null>(null);
  let selectedKv = $state<KeyValueEntry | null>(null);

  // Character form state
  let charName = $state('');
  let charDescription = $state('');
  let charAliases = $state('');
  let charTraits = $state('');
  let charNotes = $state('');

  // Relationship form state
  let relCharA = $state<number>(0);
  let relCharB = $state<number>(0);
  let relType = $state('');
  let relDescription = $state('');

  // KV form state
  let kvKey = $state('');
  let kvValue = $state('');
  let kvCategory = $state('general');
  let kvNotes = $state('');

  let activeTab = $state<'characters' | 'relationships' | 'kv'>('characters');

  async function loadWorkspace() {
    try {
      const ws = await api.getWorkspace();
      workspace.set(ws);
    } catch (e) {
      console.error('Failed to load workspace:', e);
    }
  }

  async function handleAddCharacter() {
    if (!charName.trim()) return;
    try {
      await api.addCharacter(charName, charDescription);
      charName = '';
      charDescription = '';
      await loadWorkspace();
    } catch (e) {
      console.error('Failed to add character:', e);
    }
  }

  async function handleUpdateCharacter() {
    if (!selectedCharacter) return;
    try {
      const aliases = charAliases.split(',').map(a => a.trim()).filter(a => a);
      const traits = charTraits.split(',').map(t => t.trim()).filter(t => t);
      await api.updateCharacter(
        selectedCharacter.id,
        charName || undefined,
        charDescription || undefined,
        aliases.length > 0 ? aliases : undefined,
        traits.length > 0 ? traits : undefined,
        charNotes || undefined
      );
      selectedCharacter = null;
      await loadWorkspace();
    } catch (e) {
      console.error('Failed to update character:', e);
    }
  }

  async function handleDeleteCharacter(id: number) {
    try {
      await api.removeCharacter(id);
      selectedCharacter = null;
      await loadWorkspace();
    } catch (e) {
      console.error('Failed to delete character:', e);
    }
  }

  function selectCharacter(c: Character) {
    selectedCharacter = c;
    charName = c.name;
    charDescription = c.description;
    charAliases = c.aliases.join(', ');
    charTraits = c.traits.join(', ');
    charNotes = c.notes;
  }

  async function handleAddRelationship() {
    if (relCharA === relCharB || !relType.trim()) return;
    try {
      await api.addRelationship(relCharA, relCharB, relType, relDescription);
      relType = '';
      relDescription = '';
      await loadWorkspace();
    } catch (e) {
      console.error('Failed to add relationship:', e);
    }
  }

  async function handleDeleteRelationship(id: number) {
    try {
      await api.removeRelationship(id);
      selectedRelationship = null;
      await loadWorkspace();
    } catch (e) {
      console.error('Failed to delete relationship:', e);
    }
  }

  async function handleAddKv() {
    if (!kvKey.trim() || !kvValue.trim()) return;
    try {
      await api.setKv(kvKey, kvValue, kvCategory, kvNotes);
      kvKey = '';
      kvValue = '';
      kvNotes = '';
      await loadWorkspace();
    } catch (e) {
      console.error('Failed to set KV:', e);
    }
  }

  async function handleDeleteKv(key: string) {
    try {
      await api.deleteKv(key);
      selectedKv = null;
      await loadWorkspace();
    } catch (e) {
      console.error('Failed to delete KV:', e);
    }
  }

  function selectKv(entry: KeyValueEntry) {
    selectedKv = entry;
    kvKey = entry.key;
    kvValue = entry.value;
    kvCategory = entry.category;
    kvNotes = entry.notes;
  }

  function getCharacterName(id: number): string {
    const c = $workspace.characters.find(c => c.id === id);
    return c ? c.name : `Unknown(${id})`;
  }

  $effect(() => {
    if ($workspace.characters.length > 0) {
      // Initialize relationship selectors
      relCharA = $workspace.characters[0].id;
      relCharB = $workspace.characters[0].id;
    }
  });
</script>

<div class="workspace-panel">
  <div class="tabs">
    <button class="tab" class:active={activeTab === 'characters'} onclick={() => activeTab = 'characters'}>
      Characters
    </button>
    <button class="tab" class:active={activeTab === 'relationships'} onclick={() => activeTab = 'relationships'}>
      Relationships
    </button>
    <button class="tab" class:active={activeTab === 'kv'} onclick={() => activeTab = 'kv'}>
      KV Store
    </button>
  </div>

  {#if activeTab === 'characters'}
    <div class="section">
      <div class="list">
        {#each $workspace.characters as c}
          <div class="item" class:selected={selectedCharacter?.id === c.id} onclick={() => selectCharacter(c)}>
            <span class="name">{c.name}</span>
            <button class="btn-delete" onclick={() => handleDeleteCharacter(c.id)}>×</button>
          </div>
        {:else}
          <span class="empty">No characters</span>
        {/each}
      </div>

      <div class="form">
        <input type="text" placeholder="Name" bind:value={charName} />
        <textarea placeholder="Description" bind:value={charDescription} rows="2"></textarea>
        <input type="text" placeholder="Aliases (comma separated)" bind:value={charAliases} />
        <input type="text" placeholder="Traits (comma separated)" bind:value={charTraits} />
        <textarea placeholder="Notes" bind:value={charNotes} rows="2"></textarea>

        {#if selectedCharacter}
          <button class="btn-primary" onclick={handleUpdateCharacter}>Update</button>
          <button class="btn-secondary" onclick={() => selectedCharacter = null}>Cancel</button>
        {:else}
          <button class="btn-primary" onclick={handleAddCharacter}>Add Character</button>
        {/if}
      </div>
    </div>
  {:else if activeTab === 'relationships'}
    <div class="section">
      <div class="list">
        {#each $workspace.relationships as r}
          <div class="item" onclick={() => selectedRelationship = r}>
            <span class="name">{getCharacterName(r.character_a_id)} ↔ {getCharacterName(r.character_b_id)}</span>
            <span class="type">{r.relationship_type}</span>
            <button class="btn-delete" onclick={() => handleDeleteRelationship(r.id)}>×</button>
          </div>
        {:else}
          <span class="empty">No relationships</span>
        {/each}
      </div>

      <div class="form">
        <select bind:value={relCharA}>
          {#each $workspace.characters as c}
            <option value={c.id}>{c.name}</option>
          {/each}
        </select>
        <select bind:value={relCharB}>
          {#each $workspace.characters as c}
            <option value={c.id}>{c.name}</option>
          {/each}
        </select>
        <input type="text" placeholder="Type (e.g., friend, family)" bind:value={relType} />
        <textarea placeholder="Description" bind:value={relDescription} rows="2"></textarea>
        <button class="btn-primary" onclick={handleAddRelationship}>Add Relationship</button>
      </div>
    </div>
  {:else if activeTab === 'kv'}
    <div class="section">
      <div class="list">
        {#each $workspace.key_value_store as entry}
          <div class="item" class:selected={selectedKv?.key === entry.key} onclick={() => selectKv(entry)}>
            <span class="key">{entry.key}</span>
            <span class="category">{entry.category}</span>
            <button class="btn-delete" onclick={() => handleDeleteKv(entry.key)}>×</button>
          </div>
        {:else}
          <span class="empty">No entries</span>
        {/each}
      </div>

      <div class="form">
        <input type="text" placeholder="Key" bind:value={kvKey} />
        <textarea placeholder="Value" bind:value={kvValue} rows="2"></textarea>
        <input type="text" placeholder="Category" bind:value={kvCategory} />
        <textarea placeholder="Notes" bind:value={kvNotes} rows="1"></textarea>
        <button class="btn-primary" onclick={handleAddKv}>Set Entry</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .workspace-panel {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--card-radius);
    height: 100%;
    overflow: hidden;
  }

  .tabs {
    display: flex;
    gap: 8px;
    padding: var(--spacing-xs);
    background: var(--bg-hover);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-sm);
  }

  .tab {
    padding: 8px 16px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .tab:hover {
    background: rgba(var(--accent-color-rgb), 0.1);
    color: var(--text-primary);
  }

  .tab.active {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .section {
    display: flex;
    gap: var(--spacing-md);
    flex: 1;
    overflow: hidden;
  }

  .list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: 6px 10px;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .item.selected {
    background: var(--accent-color);
    color: var(--text-inverse);
  }

  .name {
    flex: 1;
  }

  .type, .category {
    color: var(--text-muted);
    font-size: 12px;
  }

  .key {
    font-weight: 500;
  }

  .btn-delete {
    padding: 0 4px;
    border: none;
    background: transparent;
    color: var(--danger-color);
    cursor: pointer;
    font-size: 14px;
  }

  .empty {
    color: var(--text-faint);
    font-size: var(--font-size-sm);
    text-align: center;
    padding: var(--spacing-md);
  }

  .form {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm);
    background: rgba(var(--accent-color-rgb), 0.1);
    border-radius: var(--radius-md);
  }

  .form input, .form textarea, .form select {
    width: 100%;
    padding: 6px 10px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
  }

  .form input:focus, .form textarea:focus, .form select:focus {
    outline: none;
    border-color: var(--accent-color);
  }

  .btn-primary {
    padding: 8px 16px;
    border: none;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .btn-secondary {
    padding: 8px 16px;
    border: 1px solid var(--border-color);
    background: transparent;
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }
</style>