// Platform detection helpers for keyboard shortcuts

export const isMac = typeof window !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;

// Check if the command/control key is pressed (Cmd on Mac, Ctrl on other platforms)
export function isCommandKey(e: KeyboardEvent): boolean {
  return isMac ? e.metaKey : e.ctrlKey;
}

// Get the display name for the command key
export function commandKeyName(): string {
  return isMac ? 'Cmd' : 'Ctrl';
}