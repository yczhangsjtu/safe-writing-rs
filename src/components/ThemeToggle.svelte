<script lang="ts">
  let {
    currentTheme,
    onThemeChange
  }: {
    currentTheme: 'light' | 'dark';
    onThemeChange: (theme: 'light' | 'dark') => void;
  } = $props();
</script>

<button
  class="theme-toggle"
  title="Toggle theme"
  onclick={() => onThemeChange(currentTheme === 'dark' ? 'light' : 'dark')}
>
  <span class="material-icons icon">{#if currentTheme === 'dark'}light_mode{:else}dark_mode{/if}</span>
</button>

<style>
  .theme-toggle {
    width: 26px;
    height: 26px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
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

  .icon {
    font-size: 14px;
    line-height: 1;
  }

  .theme-toggle:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
</style>