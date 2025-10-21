<!-- src/lib/components/MiddleContent.svelte -->
<script lang="ts">
  import { layout } from '$lib/state/layout';
  import { t, i18n, setLang } from '$lib/i18n/index.svelte';

  // reactive references using runes
  const tab = $derived(layout.selectedTab);
  const ctx = $derived(layout.selectedContext);
  const languageLabel = $derived(() => t('language', {}, 'Language'));
  const viewLabel = $derived(() => {
    return tab === 'Radix'
      ? 'Radix chart area'
      : tab === 'Aspects'
      ? 'Aspects table area'
      : tab === 'Transits'
      ? 'Transits composite area'
      : `${tab} view`;
  });

  // available languages derived from i18n dicts
  const langs = $derived(Object.keys(i18n.dicts));
  function labelFor(code: string) {
    return (
      {
        en: 'English',
        cz: 'Čeština',
        es: 'Español',
        fr: 'Français',
      } as Record<string, string>
    )[code] ?? code.toUpperCase();
  }

  // square sizing logic
  let contentEl: HTMLDivElement | undefined;
  let square = $state(0);

  function recompute() {
    if (!contentEl) return;
    const rect = contentEl.getBoundingClientRect();
    const size = Math.floor(Math.min(rect.width, rect.height));
    square = size > 0 ? size : 0;
  }

  $effect(() => {
    if (!contentEl) return;
    const ro = new ResizeObserver(() => recompute());
    ro.observe(contentEl);
    queueMicrotask(recompute);
    return () => ro.disconnect();
  });
</script>

<div class="w-full h-full p-4">
  <div class="h-full w-full rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col">
    <div class="flex items-baseline justify-between gap-4 mb-2">
      <h2 class="text-lg font-semibold">{tab}</h2>
      <div class="text-sm opacity-80">Context: <span class="font-medium">{ctx}</span></div>
    </div>

    {#if tab === 'Settings'}
      <!-- Language selector lives in Settings -->
      <div class="mb-4 space-y-2">
        <label class="block text-sm font-medium opacity-90" for="settings-lang">{languageLabel}</label>
        <div>
          <select
            id="settings-lang"
            class="h-9 min-w-[180px] rounded-md bg-transparent text-foreground border px-2 hover:bg-white/10"
            bind:value={i18n.lang}
            onchange={(e) => setLang((e.target as HTMLSelectElement).value as any)}
          >
            {#each langs as code}
              <option value={code}>{labelFor(code)}</option>
            {/each}
          </select>
        </div>
      </div>
    {:else}
      <!-- Example translated strings in non-settings views -->
      <div class="mb-3">
        <h2 class="text-base font-semibold">{t('new')}</h2>
        <p class="text-sm opacity-80">{t('new_location')}</p>
      </div>
    {/if}

    <div class="flex-1 min-h-0 flex items-center justify-center" bind:this={contentEl}>
      {#if square > 0}
        <div
          class="rounded-md border border-dashed bg-muted/40 text-muted-foreground flex items-center justify-center"
          style={`width:${square}px;height:${square}px`}
        >
          {viewLabel}
        </div>
      {:else}
        <div class="text-sm opacity-60">Measuring available space…</div>
      {/if}
    </div>
  </div>
</div>