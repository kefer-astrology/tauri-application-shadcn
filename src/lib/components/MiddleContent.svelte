<!-- src/lib/components/MiddleContent.svelte -->
<script lang="ts">
  import { layout } from '$lib/state/layout';
  import { t, i18n, setLang } from '$lib/i18n/index.svelte';
  import * as Select from '$lib/components/ui/select/index.js';
  import { presets, preset, applyPreset } from '$lib/state/theme.svelte';
  import { showOpenExportOverlay } from '$lib/state/layout';
  import RadixChart from '$lib/components/RadixChart.svelte';
  import AspectGrid from '$lib/components/AspectGrid.svelte';

  // reactive references using runes
  const tab = $derived(layout.selectedTab);
  const ctx = $derived(layout.selectedContext);
  const languageLabel = $derived.by(() => t('language', {}, 'Language'));
  const viewLabel = $derived(() => {
    return tab === 'Radix'
      ? 'Radix chart area'
      : tab === 'Aspects'
      ? 'Aspects table area'
      : tab === 'Transits'
      ? 'Transits composite area'
      : `${tab} view`;
  });

  // Languages as items: { value, label }
  const languages = $derived(
    Object.keys(i18n.dicts).map((code) => ({
      value: code,
      label:
        ({ en: 'English', cz: 'Čeština', es: 'Español', fr: 'Français' } as Record<string, string>)[code] ?? code.toUpperCase()
    }))
  );

  // Language select value + trigger content (doc-compliant)
  // Make langValue reactive to i18n.lang changes
  let langValue = $state(String(i18n.lang));
  const langTriggerContent = $derived(
    languages.find((l) => l.value === langValue)?.label ?? 'Select language'
  );

  // Sync langValue -> i18n.lang (when user changes select)
  $effect(() => {
    if (langValue !== i18n.lang) {
      setLang(langValue as any);
    }
  });

  // Sync i18n.lang -> langValue (when language changes elsewhere)
  $effect(() => {
    if (i18n.lang !== langValue) {
      langValue = String(i18n.lang);
    }
  });

  // Presets as items and trigger content
  const presetItems = presets.map((p) => ({ value: p.id, label: p.name }));
  let presetValue = $state(String(preset.id));
  const presetTriggerContent = $derived(
    presetItems.find((p) => p.value === presetValue)?.label ?? 'Select preset'
  );

  $effect(() => {
    // Only update if the value actually changed
    if (presetValue !== preset.id) {
      applyPreset(presetValue);
    }
  });

  // Toolbar state/actions
  let searchQuery = $state('');
  function openChart() { showOpenExportOverlay(true); }
  function selectContext(c: string) { layout.selectedContext = c; }

  // square sizing logic
  let contentEl = $state<HTMLDivElement | undefined>(undefined);
  let square = $state(0);

  function recompute() {
    if (!contentEl) return;
    const rect = contentEl.getBoundingClientRect();
    const size = Math.floor(Math.min(rect.width, rect.height));
    square = size > 0 ? size : 0;
  }

  $effect(() => {
    const el = contentEl;
    if (!el) return;
    const ro = new ResizeObserver(() => recompute());
    ro.observe(el);
    queueMicrotask(recompute);
    return () => ro.disconnect();
  });
</script>

<div class="w-full h-full p-4">
  <div class="h-full w-full rounded-md border border-transparent bg-transparent p-4 flex flex-col">
    {#if tab === 'Radix'}
      <!-- Radix: Only SVG -->
      <div class="flex-1 min-h-0 flex items-center justify-center" bind:this={contentEl}>
        {#if square > 0}
          <RadixChart size={square} />
        {:else}
          <div class="text-sm opacity-60">Measuring available space…</div>
        {/if}
      </div>
    {:else if tab === 'Aspects'}
      <!-- Aspects: Aspect grid SVG in a box -->
      <div class="flex-1 min-h-0 flex items-center justify-center p-4">
        <div class="h-full w-full rounded-md border bg-card text-card-foreground shadow-sm flex items-center justify-center" bind:this={contentEl}>
          {#if square > 0}
            <AspectGrid size={square} />
          {:else}
            <div class="text-sm opacity-60">Measuring available space…</div>
          {/if}
        </div>
      </div>
    {:else if tab === 'Settings'}
      <!-- Language selector lives in Settings -->
      <div class="mb-4 space-y-2">
        <label class="block text-sm font-medium opacity-90" for="settings-lang">{languageLabel}</label>
        <div class="min-w-[220px]">
          <Select.Root type="single" name="appLanguage" bind:value={langValue}>
            <Select.Trigger class="w-[220px]" id="settings-lang">
              {langTriggerContent}
            </Select.Trigger>
            <Select.Content>
              <Select.Group>
                <Select.Label>Languages</Select.Label>
                {#each languages as lang (lang.value)}
                  <Select.Item value={lang.value} label={lang.label}>
                    {lang.label}
                  </Select.Item>
                {/each}
              </Select.Group>
            </Select.Content>
          </Select.Root>
        </div>
      </div>

      <!-- Color preset selector (from imported preset files) -->
      <div class="mb-4 space-y-2">
        <label class="block text-sm font-medium opacity-90" for="settings-preset">Color preset</label>
        <div class="min-w-[220px]">
          <Select.Root type="single" name="appPreset" bind:value={presetValue}>
            <Select.Trigger class="w-[220px]" id="settings-preset">
              {presetTriggerContent}
            </Select.Trigger>
            <Select.Content>
              <Select.Group>
                <Select.Label>Themes</Select.Label>
                {#each presetItems as item (item.value)}
                  <Select.Item value={item.value} label={item.label}>
                    {item.label}
                  </Select.Item>
                {/each}
              </Select.Group>
            </Select.Content>
          </Select.Root>
        </div>
      </div>
    {:else}
      <!-- Other tabs: Full layout with toolbar -->
      <div class="flex items-baseline justify-between gap-4 mb-2">
        <h2 class="text-lg font-semibold">{tab}</h2>
        <div class="text-sm opacity-80">Context: <span class="font-medium">{ctx}</span></div>
      </div>

      <!-- Top toolbar: search + open chart button -->
      <div class="flex items-center gap-2 mb-3">
        <input
          type="text"
          class="h-9 px-3 rounded-md bg-background text-foreground border min-w-[220px]"
          placeholder="Search…"
          bind:value={searchQuery}
        />
        <button type="button" class="px-3 py-1.5 rounded-md bg-transparent border hover:bg-white/10 text-sm">
          Search
        </button>
        <button type="button" class="px-3 py-1.5 rounded-md bg-primary text-primary-foreground text-sm" onclick={openChart}>
          Open chart
        </button>
      </div>

      <!-- Example translated strings in non-settings views -->
      <div class="mb-3">
        <h2 class="text-base font-semibold">{t('new')}</h2>
        <p class="text-sm opacity-80">{t('new_location')}</p>
      </div>

      <!-- Opened contexts list -->
      <div class="mb-3">
        <div class="text-sm font-medium opacity-85 mb-1">Opened contexts</div>
        <ul class="space-y-1 max-h-40 overflow-auto pr-1">
          {#each layout.contexts as c}
            <li class="flex items-center justify-between text-sm">
              <button class="text-left hover:underline" onclick={() => selectContext(c)}>
                <span class:font-semibold={layout.selectedContext === c}>{c}</span>
              </button>
              {#if layout.selectedContext === c}
                <span class="text-xs opacity-70">selected</span>
              {/if}
            </li>
          {/each}
        </ul>
      </div>

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
    {/if}
  </div>
</div>