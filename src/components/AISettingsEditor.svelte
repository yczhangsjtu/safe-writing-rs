<script lang="ts">
  import { copilotSettings } from '../lib/stores';
  import * as api from '../lib/tauri';
  import type { CustomSkill, Skill } from '../types';

  interface Props {}

  let {}: Props = $props();

  // Built-in skills (from backend)
  let builtinSkills = $state<Skill[]>([
    {
      id: 'define_background',
      name: '确定故事背景',
      description: '通过问答引导用户定义故事的世界观、时代背景和环境设定',
      system_prompt_addition: `You are guiding the user to define the story background. Ask questions about the world, time period, and setting. Use update_ai_settings to record the background information.`,
      first_message: '您好！让我们一起来确定您故事的背景。请问这是一个什么样的世界？'
    },
    {
      id: 'define_characters',
      name: '设定人物',
      description: '引导用户创建主要角色，包括名字、性格、外貌和背景',
      system_prompt_addition: `You are guiding the user to define characters. Ask about names, personalities, appearances, backgrounds. Use create_character to add characters to workspace.`,
      first_message: '您好！让我们来设定您故事中的人物。请问谁是主角？'
    },
    {
      id: 'define_relationships',
      name: '设定关系',
      description: '建立角色之间的关系，包括友情、爱情、敌对等',
      system_prompt_addition: `You are helping define relationships between characters. Use list_characters to see existing characters, then use create_relationship to record connections.`,
      first_message: '您好！让我们来设定角色之间的关系。'
    },
    {
      id: 'outline_plot',
      name: '规划情节大纲',
      description: '帮助用户设计故事的主线情节和关键转折点',
      system_prompt_addition: `You are helping outline the story plot. Use get_ai_settings and list_characters to understand context. Use set_kv to store plot points with category "plot".`,
      first_message: '您好！让我们来规划您的故事情节。'
    },
    {
      id: 'writing_session',
      name: '写作会话',
      description: '专注于续写当前章节，根据背景和人物信息创作',
      system_prompt_addition: `You are in a writing session. Read current passage with read_passage. Check background with get_ai_settings. Use continue_writing to append content.`,
      first_message: '您好！让我们开始写作。'
    }
  ]);

  // ===== Local state =====
  let localBackground = $state('');
  let localSettings = $state('');
  let localNotes = $state('');
  let localRequirements = $state('');
  let localWritingStyle = $state('');
  let localCustomPrompt = $state('');
  let localUseCustomPrompt = $state(false);
  let localCustomSkills = $state<CustomSkill[]>([]);
  let dirty = $state(false);

  // Snapshots
  let snapshotBackground = $state('');
  let snapshotSettings = $state('');
  let snapshotNotes = $state('');
  let snapshotRequirements = $state('');
  let snapshotWritingStyle = $state('');
  let snapshotCustomPrompt = $state('');
  let snapshotUseCustomPrompt = $state(false);
  let snapshotCustomSkills = $state<CustomSkill[]>([]);

  function takeSnapshot(s: typeof $copilotSettings) {
    snapshotBackground = s.background;
    snapshotSettings = s.settings;
    snapshotNotes = s.notes;
    snapshotRequirements = s.requirements;
    snapshotWritingStyle = s.writing_style;
    snapshotCustomPrompt = s.custom_system_prompt;
    snapshotUseCustomPrompt = s.use_custom_prompt;
    snapshotCustomSkills = s.custom_skills ? [...s.custom_skills] : [];
  }

  // Reload from store only when no local edits are in flight
  $effect(() => {
    const s = $copilotSettings;
    if (!dirty) {
      localBackground = s.background;
      localSettings = s.settings;
      localNotes = s.notes;
      localRequirements = s.requirements;
      localWritingStyle = s.writing_style;
      localCustomPrompt = s.custom_system_prompt;
      localUseCustomPrompt = s.use_custom_prompt;
      localCustomSkills = s.custom_skills ? [...s.custom_skills] : [];
      takeSnapshot(s);
    }
  });

  // ===== Edit handlers — each calls syncToBackend immediately =====
  function markDirty() {
    dirty = true;
  }

  function handleBackgroundChange(e: Event) {
    localBackground = (e.target as HTMLTextAreaElement).value;
    markDirty();
    syncToBackend();
  }

  function handleSettingsChange(e: Event) {
    localSettings = (e.target as HTMLTextAreaElement).value;
    markDirty();
    syncToBackend();
  }

  function handleNotesChange(e: Event) {
    localNotes = (e.target as HTMLTextAreaElement).value;
    markDirty();
    syncToBackend();
  }

  function handleRequirementsChange(e: Event) {
    localRequirements = (e.target as HTMLTextAreaElement).value;
    markDirty();
    syncToBackend();
  }

  function handleWritingStyleChange(e: Event) {
    localWritingStyle = (e.target as HTMLTextAreaElement).value;
    markDirty();
    syncToBackend();
  }

  function handleCustomPromptChange(e: Event) {
    localCustomPrompt = (e.target as HTMLTextAreaElement).value;
    markDirty();
    syncToBackend();
  }

  function handleUseCustomPromptChange(e: Event) {
    localUseCustomPrompt = (e.target as HTMLInputElement).checked;
    // When enabling custom prompt, if it's empty, fill with default
    if (localUseCustomPrompt && !localCustomPrompt.trim()) {
      localCustomPrompt = defaultPromptPreview;
    }
    markDirty();
    syncToBackend();
  }

  async function handleResetToDefault() {
    try {
      const newSettings = await api.resetCopilotSettings();
      copilotSettings.set(newSettings);
      localBackground = newSettings.background;
      localSettings = newSettings.settings;
      localNotes = newSettings.notes;
      localRequirements = newSettings.requirements;
      localWritingStyle = newSettings.writing_style;
      localCustomPrompt = newSettings.custom_system_prompt;
      localUseCustomPrompt = newSettings.use_custom_prompt;
      localCustomSkills = newSettings.custom_skills || [];
      takeSnapshot(newSettings);
      dirty = false;
    } catch (e) {
      console.error('Failed to reset settings:', e);
    }
  }

  // Custom Skills handlers - for overriding built-in skills
  function handleAddOverride(skillId: string) {
    const builtin = builtinSkills.find(s => s.id === skillId);
    if (!builtin) return;

    const override: CustomSkill = {
      id: `override_${skillId}`,
      name: builtin.name,
      description: builtin.description,
      system_prompt_addition: builtin.system_prompt_addition,
      first_message: builtin.first_message,
      replaces_builtin: skillId,
    };
    localCustomSkills = [...localCustomSkills, override];
    markDirty();
    syncToBackend();
  }

  function handleRemoveOverride(skillId: string) {
    localCustomSkills = localCustomSkills.filter(s => s.replaces_builtin !== skillId);
    markDirty();
    syncToBackend();
  }

  function handleOverridePromptChange(skillId: string, value: string) {
    const skill = localCustomSkills.find(s => s.replaces_builtin === skillId);
    if (skill) {
      skill.system_prompt_addition = value;
      markDirty();
      syncToBackend();
    }
  }

  function handleOverrideFirstMsgChange(skillId: string, value: string) {
    const skill = localCustomSkills.find(s => s.replaces_builtin === skillId);
    if (skill) {
      skill.first_message = value;
      markDirty();
      syncToBackend();
    }
  }

  // New custom skills handlers
  function handleAddNewSkill() {
    const newSkill: CustomSkill = {
      id: `custom_${Date.now()}`,
      name: '',
      description: '',
      system_prompt_addition: '',
      first_message: '',
      replaces_builtin: undefined,
    };
    localCustomSkills = [...localCustomSkills, newSkill];
    markDirty();
    syncToBackend();
  }

  function handleCustomSkillNameChange(index: number, value: string) {
    localCustomSkills[index].name = value;
    markDirty();
    syncToBackend();
  }

  function handleCustomSkillDescChange(index: number, value: string) {
    localCustomSkills[index].description = value;
    markDirty();
    syncToBackend();
  }

  function handleCustomSkillPromptChange(index: number, value: string) {
    localCustomSkills[index].system_prompt_addition = value;
    markDirty();
    syncToBackend();
  }

  function handleCustomSkillFirstMsgChange(index: number, value: string) {
    localCustomSkills[index].first_message = value;
    markDirty();
    syncToBackend();
  }

  function handleRemoveCustomSkill(index: number) {
    localCustomSkills = localCustomSkills.filter((_, i) => i !== index);
    markDirty();
    syncToBackend();
  }

  // ===== Sync local state to backend — directly updates copilot_settings in memory =====
  async function syncToBackend() {
    const store = $copilotSettings;

    const settings = {
      background: localBackground !== snapshotBackground ? localBackground : store.background,
      settings: localSettings !== snapshotSettings ? localSettings : store.settings,
      notes: localNotes !== snapshotNotes ? localNotes : store.notes,
      requirements: localRequirements !== snapshotRequirements ? localRequirements : store.requirements,
      writing_style: localWritingStyle !== snapshotWritingStyle ? localWritingStyle : store.writing_style,
      custom_system_prompt: localCustomPrompt !== snapshotCustomPrompt ? localCustomPrompt : store.custom_system_prompt,
      use_custom_prompt: localUseCustomPrompt !== snapshotUseCustomPrompt ? localUseCustomPrompt : store.use_custom_prompt,
      custom_skills: localCustomSkills.length !== snapshotCustomSkills.length ||
        localCustomSkills.some((s, i) => JSON.stringify(s) !== JSON.stringify(snapshotCustomSkills[i]))
        ? localCustomSkills
        : store.custom_skills,
    };

    await api.updateCopilotSettings(settings);

    copilotSettings.update(s => ({
      ...s,
      background: settings.background,
      settings: settings.settings,
      notes: settings.notes,
      requirements: settings.requirements,
      writing_style: settings.writing_style,
      custom_system_prompt: settings.custom_system_prompt,
      use_custom_prompt: settings.use_custom_prompt,
      custom_skills: settings.custom_skills,
    }));

    dirty = false;
  }

  // Exported save — called by Editor's Cmd+S handler to flush before encryptAndSave
  async function doSave() {
    await syncToBackend();
  }
  export { doSave as save };

  const defaultPromptPreview = `You are a professional writing assistant. Your role is to help create high-quality content while respecting established context and guidelines.

Follow these principles:
1. Consistency - Maintain established characters, settings, and plot elements
2. Quality - Write engaging, polished prose
3. Respect - Honor the author's vision and preferences
4. Tools - Use available tools proactively to understand context before writing`;
</script>

<div class="ai-settings-editor">
  <div class="settings-header">
    <h2 class="settings-title">AI Writing Settings</h2>
    <p class="settings-subtitle">Configure writing guidance and system prompts for the AI assistant</p>
  </div>

  <!-- Writing Guidance Section -->
  <div class="settings-section">
    <div class="section-label">Story Background</div>
    <textarea
      value={localBackground}
      oninput={handleBackgroundChange}
      rows="3"
      placeholder="Describe the story background, world setting, or context..."
      class="settings-textarea"
    ></textarea>
  </div>

  <div class="settings-section">
    <div class="section-label">World Settings</div>
    <textarea
      value={localSettings}
      oninput={handleSettingsChange}
      rows="3"
      placeholder="Define world rules, magic systems, technology levels, etc..."
      class="settings-textarea"
    ></textarea>
  </div>

  <div class="settings-section">
    <div class="section-label">Important Notes</div>
    <textarea
      value={localNotes}
      oninput={handleNotesChange}
      rows="3"
      placeholder="Key things the AI should remember or avoid..."
      class="settings-textarea"
    ></textarea>
  </div>

  <div class="settings-section">
    <div class="section-label">Requirements</div>
    <textarea
      value={localRequirements}
      oninput={handleRequirementsChange}
      rows="3"
      placeholder="Specific requirements for output (length, format, style constraints)..."
      class="settings-textarea"
    ></textarea>
  </div>

  <div class="settings-section">
    <div class="section-label">Writing Style</div>
    <textarea
      value={localWritingStyle}
      oninput={handleWritingStyleChange}
      rows="3"
      placeholder="Preferred writing style, tone, voice, literary devices..."
      class="settings-textarea"
    ></textarea>
  </div>

  <!-- System Prompt Section -->
  <div class="settings-section">
    <div class="section-header-row">
      <div class="section-label">System Prompt</div>
      <button class="btn-reset" onclick={handleResetToDefault}>
        Reset to Default
      </button>
    </div>

    <div class="prompt-toggle">
      <label class="toggle-label">
        <input
          type="checkbox"
          checked={localUseCustomPrompt}
          onchange={handleUseCustomPromptChange}
          class="toggle-checkbox"
        />
        <span class="toggle-text">Use custom system prompt</span>
      </label>
    </div>

    {#if localUseCustomPrompt}
      <textarea
        value={localCustomPrompt}
        oninput={handleCustomPromptChange}
        rows="6"
        placeholder="Enter your custom system prompt..."
        class="settings-textarea"
      ></textarea>
    {:else}
      <div class="default-prompt-preview">
        <div class="preview-label">Default System Prompt:</div>
        <pre class="preview-content">{defaultPromptPreview}</pre>
      </div>
    {/if}
  </div>

  <!-- Skills Section -->
  <div class="settings-section">
    <div class="section-label">Skills</div>
    <p class="section-hint">Skills are specialized modes for specific writing tasks. Agent can activate them via activate_skill tool.</p>

    <!-- Built-in Skills -->
    <div class="builtin-skills-list">
      {#each builtinSkills as skill}
        {@const customOverride = localCustomSkills.find(s => s.replaces_builtin === skill.id)}
        <div class="builtin-skill-item">
          <div class="skill-header-row">
            <span class="skill-name">{skill.name}</span>
            <span class="skill-id">({skill.id})</span>
            {#if customOverride}
              <span class="override-badge">Custom Override</span>
              <button class="btn-small" onclick={() => handleRemoveOverride(skill.id)}>Remove Override</button>
            {:else}
              <button class="btn-small" onclick={() => handleAddOverride(skill.id)}>Override</button>
            {/if}
          </div>
          <p class="skill-desc">{skill.description}</p>

          {#if customOverride}
            <!-- Custom override editor -->
            <div class="override-editor">
              <div class="override-label">Custom System Prompt Addition:</div>
              <textarea
                value={customOverride.system_prompt_addition}
                oninput={(e) => handleOverridePromptChange(skill.id, (e.target as HTMLTextAreaElement).value)}
                rows="4"
                placeholder="Override the skill's system prompt..."
                class="settings-textarea"
              ></textarea>
              <div class="override-label">Custom First Message:</div>
              <input
                type="text"
                value={customOverride.first_message}
                oninput={(e) => handleOverrideFirstMsgChange(skill.id, (e.target as HTMLInputElement).value)}
                placeholder="Override the skill's greeting message..."
                class="settings-input"
              />
            </div>
          {:else}
            <!-- Show default content (collapsed) -->
            <details class="skill-details">
              <summary>View default prompt addition</summary>
              <pre class="skill-default-content">{skill.system_prompt_addition}</pre>
            </details>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Add new custom skill -->
    <div class="new-skill-section">
      <div class="section-header-row">
        <span class="section-label-small">New Custom Skills</span>
        <button class="btn-add" onclick={handleAddNewSkill}>+ Add New</button>
      </div>
      {#each localCustomSkills.filter(s => !s.replaces_builtin) as skill, i}
        {@const actualIndex = localCustomSkills.findIndex(s => s === skill)}
        <div class="skill-item">
          <div class="skill-header">
            <input
              type="text"
              value={skill.name}
              oninput={(e) => handleCustomSkillNameChange(actualIndex, (e.target as HTMLInputElement).value)}
              placeholder="Skill name"
              class="settings-input skill-name-input"
            />
            <button class="btn-remove" onclick={() => handleRemoveCustomSkill(actualIndex)}>×</button>
          </div>
          <input
            type="text"
            value={skill.description}
            oninput={(e) => handleCustomSkillDescChange(actualIndex, (e.target as HTMLInputElement).value)}
            placeholder="Brief description"
            class="settings-input"
          />
          <textarea
            value={skill.system_prompt_addition}
            oninput={(e) => handleCustomSkillPromptChange(actualIndex, (e.target as HTMLTextAreaElement).value)}
            rows="4"
            placeholder="System prompt addition for this skill..."
            class="settings-textarea"
          ></textarea>
          <input
            type="text"
            value={skill.first_message}
            oninput={(e) => handleCustomSkillFirstMsgChange(actualIndex, (e.target as HTMLInputElement).value)}
            placeholder="First message when skill activates"
            class="settings-input"
          />
        </div>
      {/each}
    </div>
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
    gap: var(--spacing-md);
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

  .prompt-toggle {
    padding: var(--spacing-sm) 0;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    cursor: pointer;
  }

  .toggle-checkbox {
    width: 18px;
    height: 18px;
    accent-color: var(--accent-color);
  }

  .toggle-text {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .default-prompt-preview {
    padding: var(--spacing-sm);
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color-faint);
  }

  .preview-label {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin-bottom: var(--spacing-xs);
  }

  .preview-content {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    white-space: pre-wrap;
    line-height: 1.4;
    margin: 0;
    font-family: inherit;
  }

  .btn-reset {
    padding: 6px 12px;
    border: none;
    background: var(--bg-hover);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all 0.15s ease;
  }

  .btn-reset:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .btn-add {
    padding: 6px 12px;
    border: none;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .btn-add:hover {
    opacity: 0.9;
  }

  .section-hint {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin: 0;
  }

  .section-label-small {
    font-size: var(--font-size-sm);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .builtin-skills-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .builtin-skill-item {
    padding: var(--spacing-sm);
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color-faint);
  }

  .skill-header-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .skill-name {
    font-weight: 500;
    color: var(--text-primary);
  }

  .skill-id {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .skill-desc {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin: var(--spacing-xs) 0;
  }

  .override-badge {
    font-size: var(--font-size-xs);
    padding: 2px 8px;
    background: var(--accent-color);
    color: var(--text-inverse);
    border-radius: var(--radius-sm);
  }

  .btn-small {
    padding: 4px 8px;
    border: none;
    background: var(--bg-input);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
  }

  .btn-small:hover {
    background: var(--bg-hover-active);
    color: var(--text-primary);
  }

  .skill-details {
    margin-top: var(--spacing-sm);
  }

  .skill-details summary {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    cursor: pointer;
  }

  .skill-default-content {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    white-space: pre-wrap;
    line-height: 1.4;
    margin: var(--spacing-sm) 0;
    font-family: inherit;
    background: var(--bg-input);
    padding: var(--spacing-sm);
    border-radius: var(--radius-sm);
  }

  .override-editor {
    margin-top: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .override-label {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .new-skill-section {
    border-top: 1px solid var(--border-color-faint);
    padding-top: var(--spacing-md);
  }

  .skills-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .skill-item {
    padding: var(--spacing-sm);
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color-faint);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .skill-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .skill-name-input {
    flex: 1;
  }

  .btn-remove {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 14px;
  }

  .btn-remove:hover {
    color: var(--danger-color);
  }

  .empty-hint {
    font-size: var(--font-size-sm);
    color: var(--text-faint);
    margin: 0;
    padding: var(--spacing-sm) 0;
  }
</style>