// Runes global state (no stores, no classes)
// This file has a .svelte.ts extension so the Svelte plugin transforms runes.

export const tabs = ['Radix', 'Aspects', 'Transits', 'Settings', 'About'] as const;
export type Tab = (typeof tabs)[number];

// Initial demo contexts
const initialContexts = ['Pavel Malina', 'Pavlína Wolf', 'Josef Martinec'] as const;

export type Mode =
  | 'new_radix'
  | 'radix_view'
  | 'radix_table'
  | 'radix_transits'
  | 'settings';

export const layout = $state({
  selectedTab: tabs[0] as Tab,
  selectedContext: initialContexts[0] as string,
  contexts: [...initialContexts] as string[],
  leftExpanded: true,
  rightExpanded: true,
  mode: 'radix_view' as Mode,
  prevMode: 'radix_view' as Mode,
  overlay: {
    openExport: false,
  },
});

export function setMode(next: Mode) {
  if (layout.mode !== next) {
    layout.prevMode = layout.mode as Mode;
    layout.mode = next;
  }
}

export function showOpenExportOverlay(show: boolean) {
  layout.overlay.openExport = show;
}

export function addContext(name: string) {
  const n = name?.trim();
  if (!n) return;
  if (!layout.contexts.includes(n)) layout.contexts = [...layout.contexts, n];
  layout.selectedContext = n;
}
