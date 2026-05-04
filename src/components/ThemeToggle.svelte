<script lang="ts">
  export let currentTheme: 'light' | 'dark';

  export function onThemeChange(theme: 'light' | 'dark') {}
</script>

<button class="theme-toggle" title="Toggle theme" onclick={() => onThemeChange(currentTheme === 'dark' ? 'light' : 'dark')}>
  {#if currentTheme === 'dark'}☀{:else}🌙{/if}
</button>

<style>
  .theme-toggle {
    width: 36px;
    height: 36px;
    border: none;
    background: var(--bg-button);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 18px;
  }

  .theme-toggle:hover {
    background: var(--bg-button-hover);
  }
</style>