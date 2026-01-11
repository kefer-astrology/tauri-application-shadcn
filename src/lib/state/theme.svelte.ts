// Global theming state using Svelte 5 runes ($state, $effect)
// This keeps us consistent with other app state like `layout.svelte.ts`.

export type Theme = 'light' | 'dark';

// Resolve initial theme from localStorage or prefers-color-scheme
function resolveInitialTheme(): Theme {
  try {
    const stored = localStorage.getItem('theme');
    if (stored === 'light' || stored === 'dark') return stored;
  } catch {}
  const prefersDark = typeof window !== 'undefined' && window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
  return prefersDark ? 'dark' : 'light';
}

// Export object state; mutate its properties instead of reassigning the state variable
export const theme = $state<{ value: Theme }>({ value: resolveInitialTheme() });

export function applyTheme(next: Theme) {
  theme.value = next;
  const root = document.documentElement;
  if (next === 'dark') root.classList.add('dark');
  else root.classList.remove('dark');
  try { localStorage.setItem('theme', next); } catch {}
}

export function toggleTheme() {
  applyTheme(theme.value === 'dark' ? 'light' : 'dark');
}

// Apply immediately on module load
applyTheme(theme.value);

// Preset management
export type ThemePreset = {
  id: string;
  name: string;
  light: Record<string, string>;
  dark: Record<string, string>;
};

const PRESET_STORAGE_KEY = 'theme_preset';

import { presetsFromFiles } from '$lib/themes/presets';
export const presets: ThemePreset[] = presetsFromFiles;

function applyVars(vars: Record<string, string>) {
  const root = document.documentElement;
  for (const [k, v] of Object.entries(vars)) {
    root.style.setProperty(`--${k}`, v);
  }
}

function applyForCurrentMode(preset: ThemePreset) {
  const vars = theme.value === 'dark' ? preset.dark : preset.light;
  applyVars(vars);
}

function resolveInitialPresetId(): string {
  try {
    const stored = localStorage.getItem(PRESET_STORAGE_KEY);
    if (stored && presets.some((p) => p.id === stored)) return stored;
  } catch {}
  return presets[0].id;
}

// Export object state for preset; mutate its properties instead of reassigning
export const preset = $state<{ id: string }>({ id: resolveInitialPresetId() });

export function applyPreset(id: string) {
  const found = presets.find((p) => p.id === id) ?? presets[0];
  preset.id = found.id;
  applyForCurrentMode(found);
  try { localStorage.setItem(PRESET_STORAGE_KEY, found.id); } catch {}
}

// Re-apply helper when needed (call from components if you need to force update)
export function reapplyCurrentPreset() {
  const current = presets.find((p) => p.id === preset.id) ?? presets[0];
  applyForCurrentMode(current);
}
