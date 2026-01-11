<script lang="ts">
  import TopBar from '$lib/components/TopBar.svelte';
  import ExpandablePanel from '$lib/components/ExpandablePanel.svelte';
  import MiddleContent from '$lib/components/MiddleContent.svelte';
  import BottomTabs from '$lib/components/BottomTabs.svelte';
  import OpenExportDialog from '$lib/components/OpenExportDialog.svelte';
  import TimeNavigationPanel from '$lib/components/TimeNavigationPanel.svelte';
  import GlyphManager from '$lib/components/GlyphManager.svelte';
  import { layout, type Mode, addContext, showOpenExportOverlay, loadChartsFromWorkspace, updateChartComputation, getSelectedChart, type ChartData, setMode } from '$lib/state/layout';
  import { invoke } from '@tauri-apps/api/core';
  import { reapplyCurrentPreset, preset, presets, applyPreset } from '$lib/state/theme.svelte';
  import { timeNavigation } from '$lib/stores/timeNavigation.svelte';
  import { t, i18n, setLang } from '$lib/i18n/index.svelte';
  import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
  import * as Accordion from '$lib/components/ui/accordion/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { getGlyphContent } from '$lib/stores/glyphs.svelte';
  import BodySelector from '$lib/components/BodySelector.svelte';
  import * as Dialog from '$lib/components/ui/dialog/index.js';
  import { onMount } from 'svelte';

  let rightExpanded = $state(true);
  // Left column has three panels with independent states
  let leftTopExpanded = $state(true);
  let leftMiddleExpanded = $state(true);
  // Third panel folded by default
  let leftBottomExpanded = $state(false);

  const mode = $derived(layout.mode as Mode);

  // New Radix form state
  let newChartType = $state<'NATAL' | 'EVENT' | 'HORARY' | 'COMPOSITE'>('NATAL');
  let newContextName = $state('');
  let newDate = $state('');
  let newTime = $state('');
  let newLocation = $state('');
  let newLatitude = $state('');
  let newLongitude = $state('');
  let newHouseSystem = $state('Placidus');
  let newZodiacType = $state('Tropical');
  let newTags = $state('');
  let advancedExpanded = $state<string | undefined>(undefined);
  
  // Open Chart mode state
  let openMode = $state<'my_radixes' | 'database'>('my_radixes');
  let searchQuery = $state('');
  
  // Mock data for horoscopes table
  const horoscopes = $state([
    { name: 'John Doe', chartType: 'NATAL', dateTime: '1990-01-15 10:30', place: 'Prague', tags: 'personal, important' },
    { name: 'Jane Smith', chartType: 'EVENT', dateTime: '2020-05-20 14:00', place: 'Brno', tags: 'work' },
    { name: 'Test Chart', chartType: 'HORARY', dateTime: '2024-01-01 12:00', place: 'London', tags: 'test' }
  ]);
  
  // Export mode state
  let exportType = $state<'print' | 'pdf' | 'png'>('print');
  let exportIncludeLocation = $state(true);
  let exportIncludeAspects = $state(true);
  let exportIncludeInfo = $state(true);
  
  // Info mode state
  let selectedInfoItem = $state<string | undefined>(undefined);
  
  // Transits mode state
  let selectedTransitsSection = $state<string | undefined>('obecne');
  let transitingBodies = $state<string[]>(['sun', 'moon', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune', 'pluto']);
  let transitedBodies = $state<string[]>(['sun', 'moon', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune', 'pluto']);
  let selectedAspects = $state<string[]>(['conjunction', 'square', 'trine', 'opposition']);
  
  // Settings mode state
  let selectedSettingsSection = $state<string | undefined>('jazyk');
  let settingsChanged = $state(false);
  
  // Language and preset state for settings
  const languages = $derived(
    Object.keys(i18n.dicts).map((code) => ({
      value: code,
      label:
        ({ en: 'English', cz: 'Čeština', es: 'Español', fr: 'Français' } as Record<string, string>)[code] ?? code.toUpperCase()
    }))
  );
  let langValue = $state(String(i18n.lang));
  const langTriggerContent = $derived(
    languages.find((l) => l.value === langValue)?.label ?? 'Select language'
  );
  
  const presetItems = presets.map((p) => ({ value: p.id, label: p.name }));
  let presetValue = $state(String(preset.id));
  const presetTriggerContent = $derived(
    presetItems.find((p) => p.value === presetValue)?.label ?? 'Select preset'
  );
  
  // Sync language changes
  $effect(() => {
    if (langValue !== i18n.lang) {
      setLang(langValue as any);
      settingsChanged = true;
    }
  });
  
  // Sync preset changes
  $effect(() => {
    if (presetValue !== String(preset.id)) {
      applyPreset(presetValue);
      settingsChanged = true;
    }
  });
  
  // Info items structure
  const infoItems = $state([
    {
      id: 'positive_dominances',
      label: 'Převahy pozitivní',
      children: [
        { id: 'dominance_mode_quality', label: 'Převaha modu/kvality znamení' },
        { id: 'dominance_element', label: 'Převaha živlu' },
        { id: 'dominance_houses', label: 'Převaha v domech' },
        { id: 'dominance_aspects', label: 'Převaha aspektů' }
      ]
    },
    {
      id: 'negative_dynamics',
      label: 'Negativní dynamika',
      children: [
        { id: 'negative_quality_signs', label: 'Kvalita znamení' },
        { id: 'negative_elements', label: 'Živlu' },
        { id: 'negative_houses', label: 'V domech' },
        { id: 'negative_aspects', label: 'Aspektů' }
      ]
    },
    { id: 'quadrant_division', label: 'Rozdělení v kvadrantech' },
    { id: 'sabian_symbols', label: 'Sabiánské symboly' },
    { id: 'detailed_planet_positions', label: 'Detailní informace o poloze planet (starfisher - rozšířené info)' },
    { id: 'horoscope_shape_diagram', label: 'Tvarový diagram horoskopu' },
    { id: 'hemisphere_emphasis', label: 'Zdůraznění hemisféry' },
    { id: 'singleton_hemisphere', label: 'Singlton v hemisféře' },
    { id: 'stellium', label: 'Stellium' },
    { id: 'planetary_configuration', label: 'Planetární konfigurace' },
    { id: 'lunar_phases', label: 'Lunární fáze' },
    { id: 'sun_moon_horizon', label: 'Slunce a Luna (obzor)' },
    { id: 'mercury', label: 'Merkur' },
    { id: 'venus', label: 'Venuše' },
    { id: 'extroversion_introversion_ratio', label: 'Poměr extroverze a introverze' },
    {
      id: 'focal_planets',
      label: 'Ohniskové planety',
      children: [
        { id: 'final_dispositor', label: 'Finální dispozitor' },
        { id: 'horoscope_ruler', label: 'Vládce horoskopu' },
        { id: 'singleton', label: 'Singlton' },
        { id: 'angular_planet', label: 'Rohová planeta' },
        { id: 'by_position', label: 'Polohou' },
        { id: 'unaspect_planets', label: 'Neaspektované planety (žádné hlavní aspekty)' },
        { id: 'focal_planet', label: 'Obráběcí planeta' },
        { id: 'trigger_planet', label: 'Planeta spouštěcí' },
        { id: 'planets_abstract_points', label: 'Planety v kontaktu s abstraktními body horoskopu' }
      ]
    }
  ]);
  
  // Planet positions for Location table (same as RadixChart default)
  // Get planets from selected chart's computed data, or use defaults
  const selectedChart = $derived(getSelectedChart());
  const planets = $derived(() => {
    const computed = selectedChart?.computed?.positions;
    if (computed) {
      // Convert positions to planet data format
      // This is a placeholder - actual conversion depends on Python output format
      const result: Record<string, { degrees: number; sign: string; house: number }> = {};
      for (const [name, position] of Object.entries(computed)) {
        // Assuming position is in degrees (0-360)
        const degrees = position % 360;
        const signIndex = Math.floor(degrees / 30);
        const signs = ['♈', '♉', '♊', '♋', '♌', '♍', '♎', '♏', '♐', '♑', '♒', '♓'];
        result[name] = {
          degrees: degrees % 30,
          sign: signs[signIndex] || '♈',
          house: 1 // TODO: calculate house from position
        };
      }
      return result;
    }
    // Default demo data
    return {
      sun: { degrees: 120, sign: '♌', house: 4 },
      moon: { degrees: 45, sign: '♉', house: 1 },
      mercury: { degrees: 90, sign: '♋', house: 3 },
      venus: { degrees: 200, sign: '♏', house: 7 },
      mars: { degrees: 15, sign: '♈', house: 10 },
      jupiter: { degrees: 40, sign: '♒', house: 2 },
      saturn: { degrees: 190, sign: '♒', house: 1 },
      uranus: { degrees: 260, sign: '♒', house: 3 },
      neptune: { degrees: 310, sign: '♒', house: 5 },
      pluto: { degrees: 25, sign: '♒', house: 11 }
    };
  });
  
  // Chart details (from new chart form or loaded chart)
  const chartDetails = $derived(() => {
    const chart = selectedChart;
    if (chart) {
      // Parse dateTime - can be ISO format (2025-11-01T23:41:00) or space-separated
      const dateTimeParts = chart.dateTime.includes('T') 
        ? chart.dateTime.split('T')
        : chart.dateTime.split(' ');
      const date = dateTimeParts[0] || '';
      const time = dateTimeParts[1]?.split('.')[0] || '';
      
      return {
        chartType: chart.chartType as 'NATAL' | 'EVENT' | 'HORARY' | 'COMPOSITE',
        date,
        time,
        location: chart.location,
        latitude: chart.latitude?.toString() || '',
        longitude: chart.longitude?.toString() || '',
        timezone: chart.timezone || '',
        houseSystem: chart.houseSystem || '—',
        zodiacType: chart.zodiacType || '—',
        engine: chart.engine || '—',
        model: chart.model || '—',
        overrideEphemeris: chart.overrideEphemeris || '—',
        tags: chart.tags.join(', ')
      };
    }
    return {
      chartType: 'NATAL' as 'NATAL' | 'EVENT' | 'HORARY' | 'COMPOSITE',
      date: '',
      time: '',
      location: '',
      latitude: '',
      longitude: '',
      timezone: '',
      houseSystem: '—',
      zodiacType: '—',
      engine: '—',
      model: '—',
      overrideEphemeris: '—',
      tags: ''
    };
  });
  
  // Initialize time navigation when chart is selected
  $effect(() => {
    const chart = selectedChart;
    if (chart && chart.dateTime) {
      try {
        // Parse the chart's event time
        const chartDate = new Date(chart.dateTime);
        if (!isNaN(chartDate.getTime())) {
          // Set the current time to the chart's event time
          timeNavigation.currentTime = chartDate;
          // Set time range around the chart time (default: 1 day before/after)
          const oneDay = 24 * 60 * 60 * 1000;
          timeNavigation.startTime = new Date(chartDate.getTime() - oneDay);
          timeNavigation.endTime = new Date(chartDate.getTime() + oneDay);
        }
      } catch (err) {
        console.error('Failed to parse chart date:', err);
      }
    }
  });
  
  function submitNewContext(e?: Event) {
    e?.preventDefault?.();
    const n = newContextName.trim();
    if (!n) return;
    addContext(n);
    
    // Note: chartDetails is now derived from selectedChart
    // Form data will be saved when creating a new chart via workspace
    
    // Reset form
    newContextName = '';
    newDate = '';
    newTime = '';
    newLocation = '';
    newLatitude = '';
    newLongitude = '';
    newTags = '';
    newChartType = 'NATAL';
    newHouseSystem = 'Placidus';
    newZodiacType = 'Tropical';
  }

  // Ensure current preset is applied at app start and when theme class changes externally
  onMount(() => {
    // Apply once on mount (in case no component called applyPreset yet)
    reapplyCurrentPreset();
    // If the <html> class toggles (e.g., system/theme toggle), re-apply the preset's vars
    const mo = new MutationObserver(() => reapplyCurrentPreset());
    mo.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => mo.disconnect();
  });
</script>

<!-- Root layout: full viewport height, three rows by percentages -->
<div class="h-screen w-screen grid grid-rows-[15%_75%_10%] bg-gradient-to-br from-[#274f73] to-[#242460] text-foreground select-none box-border overflow-x-hidden">
  <!-- Top: 15% height -->
  <header class="row-span-1">
    <TopBar />
  </header>

  <!-- Middle: 75% height -->
  {#if mode === 'new_radix' || mode === 'open' || mode === 'info' || mode === 'dynamic' || mode === 'revolution' || mode === 'favorite' || mode === 'settings' || mode === 'export'}
    <!-- Left 20% + middle stretched to 80% -->
    <section class="row-span-1 grid gap-x-3 gap-y-3 px-3 pb-3 overflow-hidden w-full" style:grid-template-columns="minmax(0,20%) minmax(0,80%)">
      <!-- Left single panel -->
      <div class="h-full min-w-0 flex flex-col gap-2 min-h-0">
        <div class="min-h-0 flex-1">
          <ExpandablePanel 
            title={
              mode === 'settings' ? t('settings', {}, 'Settings')
              : mode === 'open' ? t('open_chart', {}, 'Open Chart')
              : mode === 'info' ? t('info', {}, 'Info')
              : mode === 'dynamic' ? t('dynamic', {}, 'Dynamic')
              : mode === 'revolution' ? t('revolution', {}, 'Revolution')
              : mode === 'favorite' ? t('favorite', {}, 'Favorite')
              : t('new', {}, 'New')
            } 
            editable={false}
          >
            {#snippet children()}
              {#if mode === 'new_radix'}
                {@const chartTypes = [
                  { value: 'NATAL', label: t('new_type_radix', {}, 'Radix') },
                  { value: 'EVENT', label: t('new_type_event', {}, 'Event') },
                  { value: 'HORARY', label: t('new_type_horary', {}, 'Horary') }
                ]}
                <div class="space-y-3">
                  <div class="text-xs font-medium opacity-75 mb-2">{t('new_type', {}, 'Type')}</div>
                  <Breadcrumb.Root>
                    <Breadcrumb.List class="flex flex-col gap-1.5">
                      {#each chartTypes as type, i}
                        <Breadcrumb.Item>
                          {#if newChartType === type.value}
                            <Breadcrumb.Page 
                              class="px-2 py-1.5 text-sm font-semibold underline underline-offset-4 text-foreground"
                            >
                              {type.label}
                            </Breadcrumb.Page>
                          {:else}
                            <Breadcrumb.Link>
                              {#snippet child({ props })}
                                <button
                                  type="button"
                                  class={`${props.class ?? ''} px-2 py-1.5 text-sm text-foreground/80 hover:bg-primary hover:text-primary-foreground transition-colors w-full text-left rounded-md`}
                                  onclick={() => newChartType = type.value as typeof newChartType}
                                >
                                  {type.label}
                                </button>
                              {/snippet}
                            </Breadcrumb.Link>
                          {/if}
                        </Breadcrumb.Item>
                      {/each}
                    </Breadcrumb.List>
                  </Breadcrumb.Root>
                </div>
              {:else if mode === 'open'}
                {@const openModes = [
                  { value: 'my_radixes', label: t('open_mode_my_radixes', {}, 'My Radixes') },
                  { value: 'database', label: t('open_mode_database', {}, 'Persons Database') }
                ]}
                <div class="space-y-3">
                  <Breadcrumb.Root>
                    <Breadcrumb.List class="flex flex-col gap-1.5">
                      {#each openModes as modeItem}
                        <Breadcrumb.Item>
                          {#if openMode === modeItem.value}
                            <Breadcrumb.Page 
                              class="px-2 py-1.5 text-sm font-semibold underline underline-offset-4 text-foreground"
                            >
                              {modeItem.label}
                            </Breadcrumb.Page>
                          {:else}
                            <Breadcrumb.Link>
                              {#snippet child({ props })}
                                <button
                                  type="button"
                                  class={`${props.class ?? ''} px-2 py-1.5 text-sm text-foreground/80 hover:bg-primary hover:text-primary-foreground transition-colors w-full text-left rounded-md`}
                                  onclick={() => openMode = modeItem.value as typeof openMode}
                                >
                                  {modeItem.label}
                                </button>
                              {/snippet}
                            </Breadcrumb.Link>
                          {/if}
                        </Breadcrumb.Item>
                      {/each}
                    </Breadcrumb.List>
                  </Breadcrumb.Root>
                </div>
              {:else if mode === 'export'}
                {@const exportTypes = [
                  { value: 'print', label: t('export_type_print', {}, 'Print') },
                  { value: 'pdf', label: t('export_type_pdf', {}, 'Export PDF') },
                  { value: 'png', label: t('export_type_png', {}, 'Export PNG') }
                ]}
                <div class="space-y-3">
                  <Breadcrumb.Root>
                    <Breadcrumb.List class="flex flex-col gap-1.5">
                      {#each exportTypes as typeItem}
                        <Breadcrumb.Item>
                          {#if exportType === typeItem.value}
                            <Breadcrumb.Page 
                              class="px-2 py-1.5 text-sm font-semibold underline underline-offset-4 text-foreground"
                            >
                              {typeItem.label}
                            </Breadcrumb.Page>
                          {:else}
                            <Breadcrumb.Link>
                              {#snippet child({ props })}
                                <button
                                  type="button"
                                  class={`${props.class ?? ''} px-2 py-1.5 text-sm text-foreground/80 hover:bg-primary hover:text-primary-foreground transition-colors w-full text-left rounded-md`}
                                  onclick={() => exportType = typeItem.value as typeof exportType}
                                >
                                  {typeItem.label}
                                </button>
                              {/snippet}
                            </Breadcrumb.Link>
                          {/if}
                        </Breadcrumb.Item>
                      {/each}
                    </Breadcrumb.List>
                  </Breadcrumb.Root>
                </div>
              {:else if mode === 'info'}
                <div class="space-y-1 text-sm max-h-full overflow-y-auto pr-1">
                  {#each infoItems as item}
                    {#if item.children}
                      <!-- Item with children (expandable) -->
                      {@const isExpanded = selectedInfoItem === item.id || item.children.some(c => selectedInfoItem === c.id)}
                      {@const hasSelectedChild = item.children.some(c => selectedInfoItem === c.id)}
                      <div class="space-y-0.5">
                        <button
                          type="button"
                          class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                            hasSelectedChild
                              ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                              : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                          }`}
                          onclick={() => {
                            if (hasSelectedChild) {
                              selectedInfoItem = undefined;
                            } else {
                              selectedInfoItem = item.id;
                            }
                          }}
                        >
                          {item.label}
                        </button>
                        {#if isExpanded}
                          <div class="pl-4 space-y-0.5">
                            {#each item.children as child}
                              <button
                                type="button"
                                class={`w-full text-left px-2 py-1 text-xs rounded-md transition-colors ${
                                  selectedInfoItem === child.id
                                    ? 'font-semibold underline underline-offset-2 text-foreground bg-primary/10'
                                    : 'text-foreground/70 hover:bg-primary/50 hover:text-primary-foreground'
                                }`}
                                onclick={() => selectedInfoItem = selectedInfoItem === child.id ? undefined : child.id}
                              >
                                {child.label}
                              </button>
                            {/each}
                          </div>
                        {/if}
                      </div>
                    {:else}
                      <!-- Single item -->
                      <button
                        type="button"
                        class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                          selectedInfoItem === item.id
                            ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                            : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                        }`}
                        onclick={() => selectedInfoItem = selectedInfoItem === item.id ? undefined : item.id}
                      >
                        {item.label}
                      </button>
                    {/if}
                  {/each}
                </div>
              {:else if mode === 'settings'}
                <div class="space-y-1 text-sm max-h-full overflow-y-auto pr-1">
                  <!-- Jazyk -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedSettingsSection === 'jazyk'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedSettingsSection = 'jazyk'}
                  >
                    Jazyk
                  </button>
                  
                  <!-- Lokace -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedSettingsSection === 'lokace'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedSettingsSection = 'lokace'}
                  >
                    Lokace
                  </button>
                  
                  <!-- Systém domů -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedSettingsSection === 'system_domu'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedSettingsSection = 'system_domu'}
                  >
                    Systém domů
                  </button>
                  
                  <!-- Nastavení aspektů -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedSettingsSection === 'nastaveni_aspektu'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedSettingsSection = 'nastaveni_aspektu'}
                  >
                    Nastavení aspektů
                  </button>
                  
                  <!-- Vzhled -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedSettingsSection === 'vzhled'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedSettingsSection = 'vzhled'}
                  >
                    Vzhled
                  </button>
                  
                  <!-- Manuál -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedSettingsSection === 'manual'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedSettingsSection = 'manual'}
                  >
                    Manuál
                  </button>
                </div>
              {:else}
                <div class="text-sm opacity-85">{t('mode_view_description', { mode: t(mode, {}, mode) }, 'Use the center panel for {mode} view.')}</div>
                <div class="mt-4">
                  <div class="text-sm font-medium opacity-85 mb-2">{t('list_items', {}, 'Contexts')}</div>
                  <ul class="space-y-1 max-h-40 overflow-auto pr-1">
                    {#each layout.contexts as c}
                      <li class="flex items-center justify-between text-sm">
                        <span class:font-semibold={layout.selectedContext === c}>{c}</span>
                        {#if layout.selectedContext === c}
                          <span class="text-xs opacity-70">{t('selected', {}, 'selected')}</span>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                </div>
              {/if}
            {/snippet}
          </ExpandablePanel>
        </div>
      </div>

      <!-- Middle content spans remaining width -->
      <div class="h-full min-w-0">
        {#if mode === 'new_radix'}
          <div class="h-full w-full rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col overflow-y-auto">
            <h2 class="text-lg font-semibold mb-4">{t('new', {}, 'New')}</h2>
            <form class="space-y-4 w-full max-w-2xl" onsubmit={submitNewContext}>
              <!-- Name -->
              <div class="space-y-1">
                <label class="block text-sm font-medium opacity-85" for="ctxNameCenter">
                  {t('new_name', {}, 'Name')}
                </label>
                <input
                  id="ctxNameCenter"
                  type="text"
                  class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                  bind:value={newContextName}
                  placeholder={t('new_context_placeholder', {}, 'e.g. John Doe')}
                />
              </div>
              
              <!-- Date and Time -->
              <div class="grid grid-cols-2 gap-4">
                <div class="space-y-1">
                  <label class="block text-sm font-medium opacity-85" for="new-date">
                    {t('new_date', {}, 'Date')}
                  </label>
                  <input
                    id="new-date"
                    type="date"
                    class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                    bind:value={newDate}
                  />
                </div>
                <div class="space-y-1">
                  <label class="block text-sm font-medium opacity-85" for="new-time">
                    {t('new_time', {}, 'Time')}
                  </label>
                  <input
                    id="new-time"
                    type="time"
                    class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                    bind:value={newTime}
                  />
                </div>
              </div>
              
              <!-- Location -->
              <div class="space-y-1">
                <label class="block text-sm font-medium opacity-85" for="new-location">
                  {t('new_location', {}, 'Location')}
                </label>
                <div class="flex gap-2">
                  <input
                    id="new-location"
                    type="text"
                    class="flex-1 h-9 px-3 rounded-md bg-background text-foreground border"
                    bind:value={newLocation}
                    placeholder={t('new_location_search', {}, 'Search')}
                  />
                  <button 
                    type="button" 
                    class="px-3 py-1.5 rounded-md bg-transparent border hover:bg-white/10 text-sm"
                    title={t('new_location_search', {}, 'Search')}
                  >
                    🔍
                  </button>
                </div>
              </div>
              
              <!-- Tags -->
              <div class="space-y-1">
                <label class="block text-sm font-medium opacity-85" for="new-tags">
                  {t('new_tags', {}, 'Tags')}
                </label>
                <input
                  id="new-tags"
                  type="text"
                  class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                  bind:value={newTags}
                  placeholder="e.g. personal, important"
                />
              </div>
              
              <!-- Advanced Settings -->
              <Accordion.Root bind:value={advancedExpanded} type="single" collapsible>
                <Accordion.Item value="advanced">
                  <Accordion.Trigger class="text-sm font-medium opacity-85">
                    {t('new_advanced', {}, 'Advanced')}
                  </Accordion.Trigger>
                  <Accordion.Content>
                    <div class="space-y-3 pt-2">
                      <div class="space-y-1">
                        <div class="block text-xs font-medium opacity-75">
                          {t('new_advanced_coords', {}, 'Coords')}
                        </div>
                        <div class="grid grid-cols-2 gap-2">
                          <input
                            type="text"
                            class="w-full h-8 px-2 rounded-md bg-background text-foreground border text-xs"
                            placeholder="Latitude"
                            bind:value={newLatitude}
                          />
                          <input
                            type="text"
                            class="w-full h-8 px-2 rounded-md bg-background text-foreground border text-xs"
                            placeholder="Longitude"
                            bind:value={newLongitude}
                          />
                        </div>
                      </div>
                      <div class="space-y-1">
                        <div class="block text-xs font-medium opacity-75">
                          {t('house_system', {}, 'House System')}
                        </div>
                        <select
                          class="w-full h-8 px-2 rounded-md bg-background text-foreground border text-xs"
                          bind:value={newHouseSystem}
                        >
                          <option value="Placidus">Placidus</option>
                          <option value="Whole Sign">Whole Sign</option>
                          <option value="Campanus">Campanus</option>
                          <option value="Koch">Koch</option>
                          <option value="Equal">Equal</option>
                          <option value="Regiomontanus">Regiomontanus</option>
                          <option value="Vehlow">Vehlow</option>
                          <option value="Porphyry">Porphyry</option>
                          <option value="Alcabitius">Alcabitius</option>
                        </select>
                      </div>
                      <div class="space-y-1">
                        <div class="block text-xs font-medium opacity-75">
                          {t('zodiac_type', {}, 'Zodiac Type')}
                        </div>
                        <select
                          class="w-full h-8 px-2 rounded-md bg-background text-foreground border text-xs"
                          bind:value={newZodiacType}
                        >
                          <option value="Tropical">Tropical</option>
                          <option value="Sidereal">Sidereal</option>
                        </select>
                      </div>
                      <div class="space-y-1">
                        <div class="block text-xs font-medium opacity-75">
                          {t('new_advanced_date', {}, 'Date')}
                        </div>
                        <div class="flex gap-2">
                          <button type="button" class="flex-1 px-2 py-1 text-xs rounded border hover:bg-white/10">
                            {t('new_advanced_date_gregorian', {}, 'Gregorian')}
                          </button>
                          <button type="button" class="flex-1 px-2 py-1 text-xs rounded border hover:bg-white/10">
                            {t('new_advanced_date_julian', {}, 'Julian')}
                          </button>
                        </div>
                      </div>
                      <div class="space-y-1">
                        <div class="block text-xs font-medium opacity-75">
                          {t('new_advanced_timezone', {}, 'Timezone')}
                        </div>
                        <input
                          type="text"
                          class="w-full h-8 px-2 rounded-md bg-background text-foreground border text-xs"
                          placeholder="UTC offset"
                        />
                      </div>
                      <div class="space-y-1">
                        <div class="block text-xs font-medium opacity-75">
                          {t('new_notes', {}, 'Notes')}
                        </div>
                        <textarea
                          class="w-full min-h-20 px-2 py-1 rounded-md bg-background text-foreground border text-xs resize-none"
                          placeholder="Additional notes..."
                        ></textarea>
                      </div>
                    </div>
                  </Accordion.Content>
                </Accordion.Item>
              </Accordion.Root>
              
              <!-- Submit buttons -->
              <div class="flex gap-2 pt-2">
                <button type="submit" class="px-4 py-2 rounded-md bg-primary text-primary-foreground hover:opacity-90">
                  {t('add', {}, 'Add')}
                </button>
                <button 
                  type="button" 
                  class="px-4 py-2 rounded-md bg-transparent border hover:bg-white/10"
                  onclick={() => {
                    newContextName = '';
                    newDate = '';
                    newTime = '';
                    newLocation = '';
                    newTags = '';
                    newChartType = 'NATAL';
                  }}
                >
                  {t('clear', {}, 'Clear')}
                </button>
              </div>
            </form>
          </div>
        {:else if mode === 'open'}
          <div class="h-full w-full rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col overflow-hidden">
            {#if openMode === 'my_radixes'}
              <!-- My Radixes: Table view -->
              <div class="flex flex-col h-full">
                <!-- Top toolbar: Open buttons + Search -->
                <div class="flex items-center gap-2 mb-4">
                  <Button 
                    class="px-4 py-2"
                    onclick={async () => {
                      try {
                        const folderPath = await invoke<string | null>('open_folder_dialog');
                        if (folderPath) {
                          const workspace = await invoke<{
                            path: string;
                            owner: string;
                            active_model: string | null;
                            charts: Array<{
                              id: string;
                              name: string;
                              chart_type: string;
                              date_time: string;
                              location: string;
                              tags: string[];
                            }>;
                          }>('load_workspace', { workspacePath: folderPath });
                          
                          // Load full chart data to get all settings
                          const charts: ChartData[] = [];
                          for (const ch of workspace.charts) {
                            // Get full chart data from Rust
                            try {
                              const fullChart = await invoke<{
                                id: string;
                                subject: {
                                  id: string;
                                  name: string;
                                  event_time: string | null;
                                  location: {
                                    name: string;
                                    latitude: number;
                                    longitude: number;
                                    timezone: string;
                                  };
                                };
                                config: {
                                  mode: string;
                                  house_system: string | null;
                                  zodiac_type: string;
                                  engine: string | null;
                                  model: string | null;
                                  override_ephemeris: string | null;
                                };
                                tags: string[];
                              }>('get_chart_details', {
                                workspacePath: folderPath,
                                chartId: ch.id
                              });
                              
                              charts.push({
                                id: fullChart.id,
                                name: fullChart.subject.name,
                                chartType: fullChart.config.mode,
                                dateTime: fullChart.subject.event_time || '',
                                location: fullChart.subject.location.name,
                                latitude: fullChart.subject.location.latitude,
                                longitude: fullChart.subject.location.longitude,
                                timezone: fullChart.subject.location.timezone,
                                houseSystem: fullChart.config.house_system,
                                zodiacType: fullChart.config.zodiac_type,
                                engine: fullChart.config.engine,
                                model: fullChart.config.model,
                                overrideEphemeris: fullChart.config.override_ephemeris,
                                tags: fullChart.tags,
                              });
                            } catch (err) {
                              console.error(`Failed to load full chart data for ${ch.id}:`, err);
                              // Fallback to summary data
                              charts.push({
                                id: ch.id,
                                name: ch.name,
                                chartType: ch.chart_type,
                                dateTime: ch.date_time,
                                location: ch.location,
                                tags: ch.tags,
                              });
                            }
                          }
                          
                          loadChartsFromWorkspace(charts);
                          layout.workspacePath = workspace.path;
                          
                          // Trigger computation for all charts
                          for (const chart of charts) {
                            try {
                              const result = await invoke<{
                                positions: Record<string, number>;
                                aspects: any[];
                                chart_id: string;
                              }>('compute_chart', {
                                workspacePath: workspace.path,
                                chartId: chart.id
                              });
                              
                              updateChartComputation(chart.id, {
                                positions: result.positions,
                                aspects: result.aspects
                              });
                            } catch (err) {
                              console.error(`Failed to compute chart ${chart.id}:`, err);
                            }
                          }
                        }
                      } catch (err) {
                        console.error('Failed to load workspace:', err);
                      }
                    }}
                  >
                    {t('open_workspace', {}, 'Open Workspace')}
                  </Button>
                  <Button 
                    variant="outline"
                    class="px-4 py-2"
                    onclick={async () => {
                      // TODO: Implement single radix file opening
                      console.log('Open Radix - to be implemented');
                    }}
                  >
                    {t('open_radix', {}, 'Open Radix')}
                  </Button>
                  <Input
                    type="text"
                    class="flex-1 max-w-md"
                    bind:value={searchQuery}
                    placeholder={t('search_fulltext', {}, 'Fulltext search')}
                  />
                </div>
                
                <!-- Table -->
                <div class="flex-1 overflow-auto">
                  <table class="w-full border-collapse text-sm">
                    <thead class="sticky top-0 bg-background border-b">
                      <tr>
                        <th class="text-left p-2 font-semibold opacity-85">{t('table_name', {}, 'Name')}</th>
                        <th class="text-left p-2 font-semibold opacity-85">{t('table_chart_type', {}, 'Chart Type')}</th>
                        <th class="text-left p-2 font-semibold opacity-85">{t('table_date_time', {}, 'Date & Time')}</th>
                        <th class="text-left p-2 font-semibold opacity-85">{t('table_place', {}, 'Place')}</th>
                        <th class="text-left p-2 font-semibold opacity-85">{t('table_tags', {}, 'Tags')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#each layout.contexts as chart}
                        <tr 
                          class="border-b hover:bg-accent/50 transition-colors cursor-pointer"
                          onclick={() => {
                            layout.selectedContext = chart.name;
                            setMode('radix_view');
                          }}
                        >
                          <td class="p-2">{chart.name}</td>
                          <td class="p-2 opacity-75">
                            {chart.chartType === 'NATAL' ? t('new_type_radix', {}, 'Radix')
                              : chart.chartType === 'EVENT' ? t('new_type_event', {}, 'Event')
                              : chart.chartType === 'HORARY' ? t('new_type_horary', {}, 'Horary')
                              : chart.chartType}
                          </td>
                          <td class="p-2 opacity-75">{chart.dateTime}</td>
                          <td class="p-2 opacity-75">{chart.location}</td>
                          <td class="p-2 opacity-75">{chart.tags.join(', ')}</td>
                        </tr>
                      {/each}
                      {#if layout.contexts.length === 0}
                        <tr>
                          <td colspan="5" class="p-4 text-center opacity-60">
                            {t('no_charts_loaded', {}, 'No charts loaded. Click "Open Workspace" to load charts.')}
                          </td>
                        </tr>
                      {/if}
                    </tbody>
                  </table>
                </div>
              </div>
            {:else}
              <!-- Persons Database: Placeholder -->
              <div class="flex-1 flex items-center justify-center">
                <div class="text-center space-y-2 opacity-60">
                  <p class="text-lg font-medium">{t('open_mode_database', {}, 'Persons Database')}</p>
                  <p class="text-sm">{t('database_placeholder', {}, 'Custom layout for fetching specific persons')}</p>
                </div>
              </div>
            {/if}
          </div>
        {:else if mode === 'export'}
          <div class="h-full w-full rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col overflow-y-auto">
            <h2 class="text-lg font-semibold mb-4">{t('export', {}, 'Export')}</h2>
            <div class="space-y-4 w-full max-w-2xl">
              <div class="text-sm font-medium opacity-85 mb-3">
                {t('export_include', {}, 'Include in export')}
              </div>
              
              <!-- Include Options -->
              <div class="space-y-3">
                <label class="flex items-center gap-3 cursor-pointer group">
                  <input
                    type="checkbox"
                    bind:checked={exportIncludeLocation}
                    class="w-4 h-4 rounded border border-foreground/30 bg-background text-primary focus:ring-2 focus:ring-primary focus:ring-offset-2"
                  />
                  <span class="text-sm opacity-85 group-hover:opacity-100 transition-opacity">
                    {t('export_include_location', {}, 'Location')}
                  </span>
                </label>
                
                <label class="flex items-center gap-3 cursor-pointer group">
                  <input
                    type="checkbox"
                    bind:checked={exportIncludeAspects}
                    class="w-4 h-4 rounded border border-foreground/30 bg-background text-primary focus:ring-2 focus:ring-primary focus:ring-offset-2"
                  />
                  <span class="text-sm opacity-85 group-hover:opacity-100 transition-opacity">
                    {t('export_include_aspects', {}, 'Aspects')}
                  </span>
                </label>
                
                <label class="flex items-center gap-3 cursor-pointer group">
                  <input
                    type="checkbox"
                    bind:checked={exportIncludeInfo}
                    class="w-4 h-4 rounded border border-foreground/30 bg-background text-primary focus:ring-2 focus:ring-primary focus:ring-offset-2"
                  />
                  <span class="text-sm opacity-85 group-hover:opacity-100 transition-opacity">
                    {t('export_include_info', {}, 'Info')}
                  </span>
                </label>
              </div>
              
              <!-- Export Button -->
              <div class="pt-4 border-t mt-6">
                <Button 
                  class="w-full sm:w-auto px-6 py-2"
                  onclick={() => showOpenExportOverlay(true)}
                >
                  {t('export', {}, 'Export')}
                </Button>
              </div>
            </div>
          </div>
        {:else if mode === 'info' || mode === 'dynamic' || mode === 'revolution' || mode === 'favorite'}
          <div class="h-full w-full rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col items-start justify-start">
            <h2 class="text-lg font-semibold mb-3">{t(mode, {}, mode.charAt(0).toUpperCase() + mode.slice(1))}</h2>
            <div class="text-sm opacity-85">{t('mode_view_placeholder', { mode: t(mode, {}, mode) }, 'Content for {mode} view will be displayed here.')}</div>
          </div>
        {:else if mode === 'settings'}
          <div class="h-full min-w-0 rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col overflow-hidden">
            <div class="flex-1 min-h-0 overflow-y-auto">
              {#if selectedSettingsSection === 'jazyk'}
                <!-- Jazyk (Language) -->
                <h3 class="text-sm font-semibold mb-4">Jazyk</h3>
                <div class="space-y-4 max-w-md">
                  <div class="space-y-2">
                    <label class="block text-sm font-medium opacity-90" for="settings-lang">{t('language', {}, 'Language')}</label>
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
                </div>
              {:else if selectedSettingsSection === 'lokace'}
                <!-- Lokace (Location) -->
                <h3 class="text-sm font-semibold mb-4">Lokace</h3>
                <div class="space-y-4 max-w-md">
                  <div class="space-y-2">
                    <div class="block text-sm font-medium opacity-90">Výchozí lokace</div>
                    <input
                      type="text"
                      class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                      placeholder="Zadejte výchozí lokaci..."
                    />
                  </div>
                  <div class="space-y-2">
                    <div class="block text-sm font-medium opacity-90">Zeměpisná šířka</div>
                    <input
                      type="text"
                      class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                      placeholder="Latitude"
                    />
                  </div>
                  <div class="space-y-2">
                    <div class="block text-sm font-medium opacity-90">Zeměpisná délka</div>
                    <input
                      type="text"
                      class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                      placeholder="Longitude"
                    />
                  </div>
                </div>
              {:else if selectedSettingsSection === 'system_domu'}
                <!-- Systém domů (House System) -->
                <h3 class="text-sm font-semibold mb-4">Systém domů</h3>
                <div class="space-y-4 max-w-md">
                  <div class="space-y-2">
                    <div class="block text-sm font-medium opacity-90">{t('house_system', {}, 'House System')}</div>
                    <select class="w-full h-9 px-3 rounded-md bg-background text-foreground border">
                      <option value="Placidus">Placidus</option>
                      <option value="Whole Sign">Whole Sign</option>
                      <option value="Campanus">Campanus</option>
                      <option value="Koch">Koch</option>
                      <option value="Equal">Equal</option>
                      <option value="Regiomontanus">Regiomontanus</option>
                      <option value="Vehlow">Vehlow</option>
                      <option value="Porphyry">Porphyry</option>
                      <option value="Alcabitius">Alcabitius</option>
                    </select>
                  </div>
                </div>
              {:else if selectedSettingsSection === 'nastaveni_aspektu'}
                <!-- Nastavení aspektů (Aspect Settings) -->
                <h3 class="text-sm font-semibold mb-4">Nastavení aspektů</h3>
                <div class="space-y-4 max-w-md">
                  <div class="space-y-2">
                    <div class="block text-sm font-medium opacity-90">Výchozí aspekty</div>
                    <div class="space-y-2">
                      {#each [
                        { id: 'conjunction', label: 'Conjunction (☌)', defaultOrb: 8 },
                        { id: 'sextile', label: 'Sextile (*)', defaultOrb: 6 },
                        { id: 'square', label: 'Square (□)', defaultOrb: 8 },
                        { id: 'trine', label: 'Trine (△)', defaultOrb: 8 },
                        { id: 'quincunx', label: 'Quincunx (∠)', defaultOrb: 3 },
                        { id: 'opposition', label: 'Opposition (☍)', defaultOrb: 8 }
                      ] as aspect}
                        <div class="flex items-center justify-between">
                          <label class="flex items-center gap-2 cursor-pointer">
                            <input
                              type="checkbox"
                              class="w-4 h-4 rounded border border-foreground/30 bg-background text-primary focus:ring-2 focus:ring-primary focus:ring-offset-2 cursor-pointer"
                              checked={true}
                            />
                            <span class="text-sm">{aspect.label}</span>
                          </label>
                          <input
                            type="number"
                            class="w-20 h-8 px-2 rounded-md bg-background text-foreground border text-xs"
                            value={aspect.defaultOrb}
                            min="0"
                            max="30"
                            step="0.5"
                          />
                        </div>
                      {/each}
                    </div>
                  </div>
                </div>
              {:else if selectedSettingsSection === 'vzhled'}
                <!-- Vzhled (Appearance) -->
                <h3 class="text-sm font-semibold mb-4">Vzhled</h3>
                <div class="space-y-4 max-w-md">
                  <div class="space-y-2">
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
                  <div class="mt-4">
                    <GlyphManager />
                  </div>
                </div>
              {:else if selectedSettingsSection === 'manual'}
                <!-- Manuál (Manual) -->
                <h3 class="text-sm font-semibold mb-4">Manuál</h3>
                <div class="space-y-4 max-w-2xl">
                  <div class="prose prose-sm dark:prose-invert max-w-none">
                    <p class="text-sm opacity-85">
                      Dokumentace a nápověda k aplikaci bude zobrazena zde.
                    </p>
                  </div>
                </div>
              {/if}
            </div>
            <!-- Cancel/Confirm buttons at bottom -->
            <div class="pt-4 mt-4 border-t border-border/60 flex-shrink-0 flex gap-2">
              <Button 
                variant="outline"
                class="flex-1"
                onclick={() => {
                  // TODO: Reset settings to original values
                  settingsChanged = false;
                }}
              >
                {t('cancel', {}, 'Cancel')}
              </Button>
              <Button 
                class="flex-1"
                onclick={() => {
                  // TODO: Save settings
                  settingsChanged = false;
                }}
              >
                {t('confirm', {}, 'Confirm')}
              </Button>
            </div>
          </div>
        {:else}
          <MiddleContent />
        {/if}
      </div>
    </section>
  {:else if mode === 'radix_table'}
    <!-- Left 20% (1 panel) + middle stretched to 80% -->
    <section class="row-span-1 grid gap-x-3 gap-y-3 px-3 pb-3 overflow-hidden w-full" style:grid-template-columns="minmax(0,20%) minmax(0,80%)">
      <div class="h-full min-w-0 flex flex-col gap-2 min-h-0">
        <div class="min-h-0" class:flex-1={leftTopExpanded}>
          <ExpandablePanel title={t('table_tools', {}, 'Table Tools')} bind:expanded={leftTopExpanded}>
            {#snippet children()}
              <div class="space-y-2 text-sm">
                <p>{t('table_tools_description', {}, 'Table filters and helpers.')}</p>
                <div class="h-24 rounded border border-dashed bg-muted/40"></div>
              </div>
            {/snippet}
          </ExpandablePanel>
        </div>
      </div>
      <div class="h-full min-w-0">
        <MiddleContent />
      </div>
    </section>
  {:else}
    <!-- radix_view and radix_transits: fixed split 20% / 60% / 20% (or 20% / 80% for Aspects, or 20% / 80% for Transits) -->
    {@const isAspectsView = layout.selectedTab === 'Aspects'}
    {@const isTransitsView = mode === 'radix_transits'}
    <section 
      class="row-span-1 grid gap-x-3 gap-y-3 px-3 pb-3 overflow-hidden w-full" 
      style:grid-template-columns={(isAspectsView || isTransitsView) ? "minmax(0,20%) minmax(0,80%)" : "minmax(0,20%) minmax(0,60%) minmax(0,20%)"}
    >
      <!-- Left column: stack two panels (removed Transits panel) -->
      {#if isTransitsView}
        <!-- Transits mode: only show transits selector -->
        <div class="h-full min-w-0 flex flex-col gap-2 min-h-0">
          <div class="min-h-0" class:flex-1={leftMiddleExpanded}>
            <ExpandablePanel title={t('transits', {}, 'Transits')} bind:expanded={leftMiddleExpanded}>
              {#snippet children()}
                <div class="space-y-1 text-sm max-h-full overflow-y-auto pr-1">
                  <!-- Obecné -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedTransitsSection === 'obecne'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedTransitsSection = 'obecne'}
                  >
                    Obecné
                  </button>
                  
                  <!-- Tranzitující tělesa -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedTransitsSection === 'transiting'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedTransitsSection = 'transiting'}
                  >
                    Tranzitující tělesa
                  </button>
                  
                  <!-- Tranzitovaná tělesa -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedTransitsSection === 'transited'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedTransitsSection = 'transited'}
                  >
                    Tranzitovaná tělesa
                  </button>
                  
                  <!-- Použité aspekty -->
                  <button
                    type="button"
                    class={`w-full text-left px-2 py-1.5 text-sm rounded-md transition-colors ${
                      selectedTransitsSection === 'aspects'
                        ? 'font-semibold underline underline-offset-4 text-foreground bg-primary/10'
                        : 'text-foreground/80 hover:bg-primary hover:text-primary-foreground'
                    }`}
                    onclick={() => selectedTransitsSection = 'aspects'}
                  >
                    Použité aspekty
                  </button>
                </div>
              {/snippet}
            </ExpandablePanel>
          </div>
        </div>
      {:else}
        <!-- Normal radix view: show chart details and astrolab -->
        <div class="h-full min-w-0 flex flex-col gap-2 min-h-0">
          <!-- Panel 1: title is current context name -->
          <div class="min-h-0" class:flex-1={leftTopExpanded}>
            <ExpandablePanel title={layout.selectedContext} bind:expanded={leftTopExpanded}>
              {#snippet children()}
                <div class="space-y-2.5 text-sm">
                  <!-- Chart Type -->
                  <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('new_type', {}, 'Type')}
                    </div>
                    <div class="opacity-85 text-right text-xs">
                      {chartDetails.chartType === 'NATAL' ? t('new_type_radix', {}, 'Radix')
                        : chartDetails.chartType === 'EVENT' ? t('new_type_event', {}, 'Event')
                        : chartDetails.chartType === 'HORARY' ? t('new_type_horary', {}, 'Horary')
                        : t('new_type_composite', {}, 'Composite')}
                    </div>
                  </div>
                  
                  <!-- Date -->
                  <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('new_date', {}, 'Date')}
                    </div>
                    <div class="opacity-85 font-mono text-xs text-right">
                      {chartDetails.date || '—'}
                    </div>
                  </div>
                  
                  <!-- Time -->
                  <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('new_time', {}, 'Time')}
                    </div>
                    <div class="opacity-85 font-mono text-xs text-right">
                      {chartDetails.time || '—'}
                    </div>
                  </div>
                  
                  <!-- Location -->
                  <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('new_location', {}, 'Location')}
                    </div>
                    <div class="opacity-85 text-xs text-right space-y-0.5 min-w-0 flex-1">
                      {#if chartDetails.location}
                        <div class="break-words">{chartDetails.location}</div>
                      {/if}
                      {#if chartDetails.latitude && chartDetails.longitude}
                        <div class="font-mono opacity-75 text-[10px]">
                          {chartDetails.latitude}, {chartDetails.longitude}
                        </div>
                      {/if}
                      {#if !chartDetails.location && !chartDetails.latitude}
                        <div class="opacity-60">—</div>
                      {/if}
                    </div>
                  </div>
                  
                  <!-- House System -->
                  <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('house_system', {}, 'House System')}
                    </div>
                    <div class="opacity-85 text-xs text-right">
                      {chartDetails.houseSystem}
                    </div>
                  </div>
                  
                  <!-- Zodiac Type -->
                  <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('zodiac_type', {}, 'Zodiac Type')}
                    </div>
                    <div class="opacity-85 text-xs text-right">
                      {chartDetails.zodiacType}
                    </div>
                  </div>
                  
                  <!-- Engine -->
                  <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('engine', {}, 'Engine')}
                    </div>
                    <div class="opacity-85 text-xs text-right">
                      {chartDetails.engine}
                    </div>
                  </div>
                  
                  <!-- Model -->
                  {#if chartDetails.model && chartDetails.model !== '—'}
                    <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                      <div class="text-xs font-medium opacity-75 flex-shrink-0">
                        {t('model', {}, 'Model')}
                      </div>
                      <div class="opacity-85 text-xs text-right">
                        {chartDetails.model}
                      </div>
                    </div>
                  {/if}
                  
                  <!-- Override Ephemeris -->
                  {#if chartDetails.overrideEphemeris && chartDetails.overrideEphemeris !== '—'}
                    <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                      <div class="text-xs font-medium opacity-75 flex-shrink-0">
                        {t('override_ephemeris', {}, 'Override Ephemeris')}
                      </div>
                      <div class="opacity-85 text-xs text-right break-words min-w-0 flex-1">
                        {chartDetails.overrideEphemeris}
                      </div>
                    </div>
                  {/if}
                  
                  <!-- Timezone -->
                  {#if chartDetails.timezone}
                    <div class="flex items-start justify-between gap-3 py-1 border-b border-border/40">
                      <div class="text-xs font-medium opacity-75 flex-shrink-0">
                        {t('timezone', {}, 'Timezone')}
                      </div>
                      <div class="opacity-85 text-xs text-right">
                        {chartDetails.timezone}
                      </div>
                    </div>
                  {/if}
                  
                  <!-- Tags -->
                  <div class="flex items-start justify-between gap-3 py-1">
                    <div class="text-xs font-medium opacity-75 flex-shrink-0">
                      {t('new_tags', {}, 'Tags')}
                    </div>
                    <div class="opacity-85 text-xs text-right break-words min-w-0 flex-1">
                      {chartDetails.tags || '—'}
                    </div>
                  </div>
                </div>
              {/snippet}
            </ExpandablePanel>
          </div>
          <!-- Panel 2: Astrolab -->
          <div class="min-h-0" class:flex-1={leftMiddleExpanded}>
            <ExpandablePanel title={t('astrolabe', {}, 'Astrolab')} bind:expanded={leftMiddleExpanded}>
              {#snippet children()}
                <TimeNavigationPanel />
              {/snippet}
            </ExpandablePanel>
          </div>
        </div>
      {/if}

      <!-- Middle content -->
      {#if mode === 'radix_transits' && selectedTransitsSection}
        <div class="h-full min-w-0 rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col overflow-hidden">
          <div class="flex-1 min-h-0 overflow-y-auto">
            {#if selectedTransitsSection === 'obecne'}
              <h3 class="text-sm font-semibold mb-4">Obecné nastavení tranzitů</h3>
              <div class="space-y-4 max-w-md">
                <div class="space-y-2">
                  <div class="text-sm font-medium">Z graf</div>
                  <select class="w-full h-9 px-3 rounded-md bg-background text-foreground border">
                    <option>Vyberte graf...</option>
                  </select>
                </div>
                <div class="space-y-2">
                  <div class="text-sm font-medium">Do grafu</div>
                  <select class="w-full h-9 px-3 rounded-md bg-background text-foreground border">
                    <option>Vyberte graf...</option>
                  </select>
                </div>
                <div class="space-y-2">
                  <div class="text-sm font-medium">Časové rozmezí</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input type="date" class="h-9 px-3 rounded-md bg-background text-foreground border" />
                    <input type="date" class="h-9 px-3 rounded-md bg-background text-foreground border" />
                  </div>
                </div>
              </div>
            {:else if selectedTransitsSection === 'transiting'}
              <h3 class="text-sm font-semibold mb-3">Tranzitující tělesa</h3>
              <BodySelector bind:selectedBodies={transitingBodies} />
            {:else if selectedTransitsSection === 'transited'}
              <h3 class="text-sm font-semibold mb-3">Tranzitovaná tělesa</h3>
              <BodySelector bind:selectedBodies={transitedBodies} />
            {:else if selectedTransitsSection === 'aspects'}
              <h3 class="text-sm font-semibold mb-3">Použité aspekty</h3>
              <div class="space-y-2">
                {#each [
                  { id: 'conjunction', label: 'Conjunction (☌)' },
                  { id: 'sextile', label: 'Sextile (*)' },
                  { id: 'square', label: 'Square (□)' },
                  { id: 'trine', label: 'Trine (△)' },
                  { id: 'quincunx', label: 'Quincunx (∠)' },
                  { id: 'opposition', label: 'Opposition (☍)' }
                ] as aspect}
                  <label class="flex items-center gap-2 cursor-pointer group hover:opacity-80 transition-opacity">
                    <input
                      type="checkbox"
                      class="w-4 h-4 rounded border border-foreground/30 bg-background text-primary focus:ring-2 focus:ring-primary focus:ring-offset-2 cursor-pointer"
                      checked={selectedAspects.includes(aspect.id)}
                      onchange={() => {
                        if (selectedAspects.includes(aspect.id)) {
                          selectedAspects = selectedAspects.filter(id => id !== aspect.id);
                        } else {
                          selectedAspects = [...selectedAspects, aspect.id];
                        }
                      }}
                    />
                    <span class="text-sm">{aspect.label}</span>
                  </label>
                {/each}
              </div>
            {/if}
          </div>
          <!-- Calculate button at bottom -->
          <div class="pt-4 mt-4 border-t border-border/60 flex-shrink-0">
            <Button 
              class="w-full"
              onclick={() => {
                // TODO: Implement transit calculation
                console.log('Calculate transits', {
                  transitingBodies,
                  transitedBodies,
                  selectedAspects
                });
              }}
            >
              {t('calculate', {}, 'Calculate')}
            </Button>
          </div>
        </div>
      {:else}
        <div class="h-full min-w-0">
          <MiddleContent />
        </div>
      {/if}

      <!-- Right panel (hidden for Aspects view and Transits view) -->
      {#if !isAspectsView && !isTransitsView}
        <div class="h-full min-w-0">
          <ExpandablePanel title={t('right_panel', {}, 'Location')} bind:expanded={rightExpanded}>
            {#snippet children()}
              {#if mode === 'radix_view'}
                <!-- Location table: object glyph / location (degrees glyph minutes) -->
                <div class="overflow-auto">
                  <table class="w-full text-sm border-collapse">
                    <thead class="sticky top-0 bg-background border-b">
                      <tr>
                        <th class="text-left p-2 font-semibold opacity-85">{t('table_object', {}, 'Object')}</th>
                        <th class="text-left p-2 font-semibold opacity-85">{t('table_location', {}, 'Location')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#each Object.entries(planets) as [planetName, planetData]}
                        {@const planetGlyph = getGlyphContent(planetName)}
                        {@const signSymbolMap: Record<string, string> = {
                          '♈': 'aries', '♉': 'taurus', '♊': 'gemini', '♋': 'cancer',
                          '♌': 'leo', '♍': 'virgo', '♎': 'libra', '♏': 'scorpio',
                          '♐': 'sagittarius', '♑': 'capricorn', '♒': 'aquarius', '♓': 'pisces'
                        }}
                        {@const signName = signSymbolMap[planetData.sign] || planetData.sign.toLowerCase()}
                        {@const signGlyph = getGlyphContent(signName)}
                        {@const signDeg = planetData.degrees % 30}
                        {@const minutes = Math.floor((signDeg % 1) * 60)}
                        <tr class="border-b hover:bg-accent/50 transition-colors">
                          <td class="p-2">
                            {#if planetGlyph.type === 'svg'}
                              <span class="inline-block" style="width: 1.2em; height: 1.2em; vertical-align: middle;">
                                {@html planetGlyph.content}
                              </span>
                            {:else}
                              <span class="text-lg">{planetGlyph.content || planetName.charAt(0).toUpperCase()}</span>
                            {/if}
                          </td>
                          <td class="p-2 opacity-75 font-mono text-xs">
                            {Math.floor(signDeg)}° 
                            {#if signGlyph.type === 'svg'}
                              <span class="inline-block" style="width: 1em; height: 1em; vertical-align: middle;">
                                {@html signGlyph.content}
                              </span>
                            {:else}
                              {signGlyph.content || planetData.sign}
                            {/if}
                            {' '}{minutes}'
                          </td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              {:else}
                <div class="space-y-2 text-sm">
                  <p>{t('right_panel_description', {}, 'Expandable content (right). Put properties, logs, etc.')}</p>
                  <div class="h-40 rounded border border-dashed bg-muted/40"></div>
                </div>
              {/if}
            {/snippet}
          </ExpandablePanel>
        </div>
      {/if}
    </section>
  {/if}

  <!-- Bottom: 10% height -->
  <footer class="row-span-1">
    <BottomTabs />
  </footer>

  {#if layout.overlay.openExport}
    <OpenExportDialog />
  {/if}
</div>
