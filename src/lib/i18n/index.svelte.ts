// Runes-based i18n (Svelte 5)
// - Uses $state for current language and dictionaries
// - Uses $derived for the active dictionary
// - Exported t(key) returns the current translated string
// - This file must be Svelte-compiled (.svelte.ts)

import en from './en.json';
import cz from './cz.json';
import es from './es.json';
import fr from './fr.json';

export type Messages = Record<string, any>;
export type Dictionaries = Record<string, Messages>;

// Helper: nested path lookup 'a.b.c' (supports both flat and nested keys)
function getPath(obj: any, path: string): any {
  if (!obj) return undefined;
  if (!path) return obj;
  return path.split('.').reduce((acc, part) => (acc && acc[part] != null ? acc[part] : undefined), obj);
}

export const i18n = $state<{ lang: string; dicts: Dictionaries }>({
  lang: 'en',
  dicts: { en, cz, es, fr },
});

// active dictionary reacts to i18n.lang
const active = $derived(i18n.dicts[i18n.lang] ?? {});

// t() reads from the current active dict; reactivity flows via $derived above
export function t(key: string, vars?: Record<string, any>, fallback?: string) {
  const raw = getPath(active, key) ?? active[key];
  if (typeof raw === 'string') return interpolate(raw, vars);
  return fallback ?? key;
}

// minimal template interpolation: "Hello {name}" -> vars.name
function interpolate(template: string, vars?: Record<string, any>) {
  if (!template || !vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, k) => (k in vars ? String(vars[k]) : `{${k}}`));
}

export type Lang = keyof typeof i18n.dicts;
export function setLang(l: Lang) {
  if (!l) return;
  if (i18n.dicts[l]) {
    i18n.lang = l;
    try {
      localStorage.setItem('lang', l);
    } catch (_) {}
  }
}

// Restore persisted language if available
try {
  const saved = localStorage.getItem('lang');
  if (saved && saved in i18n.dicts) i18n.lang = saved;
} catch (_) {}
