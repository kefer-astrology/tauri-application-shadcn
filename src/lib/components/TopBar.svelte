<script lang="ts">
  import { layout, type Tab, setMode, showOpenExportOverlay } from '$lib/state/layout';
  import { t } from '$lib/i18n/index.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let { logoText = 'Kefer' } = $props();

  type Action = {
    id: 'new' | 'load' | 'export' | 'radix' | 'aspects' | 'transits' | 'settings' | 'about' | 'help' | 'exit';
    onClick: () => void | Promise<void>;
    icon: 'doc' | 'folder' | 'download' | 'radix' | 'aspects' | 'transits' | 'gear' | 'info' | 'help' | 'power';
  };

  function setTab(tab: Tab) {
    layout.selectedTab = tab;
  }

  // Map specific buttons to translation keys provided in JSON
  function labelKeyFor(id: Action['id']): string {
    switch (id) {
      case 'new': return 'new';
      case 'load': return 'load';
      case 'export': return 'share';
      case 'transits': return 'transits';
      case 'aspects': return 'aspects';
      case 'radix': return 'new_type_radix';
      case 'settings': return 'settings';
      case 'about': return 'settings_about';
      case 'exit': return 'exit';
      default: return id; // fallback to flat keys if present
    }
  }

  const actions: Action[] = [
    { id: 'new', onClick: () => { setMode('new_radix'); }, icon: 'doc' },
    { id: 'load', onClick: () => { showOpenExportOverlay(true); }, icon: 'folder' },
    { id: 'export', onClick: () => { showOpenExportOverlay(true); }, icon: 'download' },
    { id: 'radix', onClick: () => { setMode('radix_view'); setTab('Radix'); }, icon: 'radix' },
    { id: 'aspects', onClick: () => { setMode('radix_table'); setTab('Aspects'); }, icon: 'aspects' },
    { id: 'transits', onClick: () => { setMode('radix_transits'); setTab('Transits'); }, icon: 'transits' },
    { id: 'settings', onClick: () => { setMode('settings'); setTab('Settings'); }, icon: 'gear' },
    { id: 'about', onClick: () => setTab('About'), icon: 'info' },
    { id: 'help', onClick: () => {/* TODO: implement */}, icon: 'help' },
    { id: 'exit', onClick: async () => { try { await getCurrentWindow().close(); } catch (e) { console.error(e); } }, icon: 'power' },
  ];

  function onIconClick(a: Action) {
    a.onClick();
  }
</script>

<!-- Top bar: grid with logo 15% width and icons area filling the rest -->
<div class="w-full h-full grid grid-cols-[15%_auto] items-center px-4">
  <!-- Logo area -->
  <div class="h-full flex items-center">
    <div class="text-xl font-bold tracking-wide select-none text-white">{logoText}</div>
  </div>
  <!-- Icons area only -->
  <div class="h-full grid content-center">
    <div class="grid grid-cols-10 gap-2 p-1 place-items-center">
      {#each actions as a}
        <button
          class="h-12 w-12 flex items-center justify-center rounded-md bg-transparent text-white hover:bg-white/10 transition-colors"
          onclick={() => onIconClick(a)}
          aria-label={t(labelKeyFor(a.id))}
          title={t(labelKeyFor(a.id))}
        >
          {#if a.icon === 'doc'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <path d="M14 2v6h6"/>
            </svg>
          {:else if a.icon === 'folder'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 7h5l2 3h11v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
            </svg>
          {:else if a.icon === 'download'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <path d="M7 10l5 5 5-5"/>
              <path d="M12 15V3"/>
            </svg>
          {:else if a.icon === 'radix'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="8" />
              <path d="M12 4v16M4 12h16"/>
            </svg>
          {:else if a.icon === 'aspects'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 6h16M4 12h16M4 18h16"/>
            </svg>
          {:else if a.icon === 'transits'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 3v18h18"/>
              <path d="M6 15l4-4 3 3 5-5"/>
            </svg>
          {:else if a.icon === 'gear'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"/>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09c-.66 0-1.26.39-1.51 1z"/>
            </svg>
          {:else if a.icon === 'info'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4"/>
              <path d="M12 8h.01"/>
            </svg>
          {:else if a.icon === 'help'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 9a3 3 0 1 1 4 2.83c-.9.34-1.5 1.2-1.5 2.17V15"/>
              <path d="M12 19h.01"/>
              <circle cx="12" cy="12" r="10"/>
            </svg>
          {:else if a.icon === 'power'}
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2v10"/>
              <path d="M5.5 7a8 8 0 1 0 13 0"/>
            </svg>
          {/if}
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  button { min-height: 36px; }
</style>
