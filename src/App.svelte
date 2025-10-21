<script lang="ts">
  import TopBar from '$lib/components/TopBar.svelte';
  import ExpandablePanel from '$lib/components/ExpandablePanel.svelte';
  import MiddleContent from '$lib/components/MiddleContent.svelte';
  import BottomTabs from '$lib/components/BottomTabs.svelte';
  import { layout, type Mode, addContext } from '$lib/state/layout';
  import OpenExportDialog from '$lib/components/OpenExportDialog.svelte';

  let rightExpanded = $state(true);
  // Left column has three panels with independent states
  let leftTopExpanded = $state(true);
  let leftMiddleExpanded = $state(true);
  // Third panel folded by default
  let leftBottomExpanded = $state(false);

  const mode = $derived(layout.mode as Mode);

  // New Radix input state
  let newContextName = $state('');
  function submitNewContext(e?: Event) {
    e?.preventDefault?.();
    const n = newContextName.trim();
    if (!n) return;
    addContext(n);
    newContextName = '';
  }
</script>

<!-- Root layout: full viewport height, three rows by percentages -->
<div class="h-screen w-screen grid grid-rows-[15%_75%_10%] bg-gradient-to-br from-[#274f73] to-[#242460] text-foreground select-none box-border overflow-x-hidden">
  <!-- Top: 15% height -->
  <header class="row-span-1">
    <TopBar logoText="Kefer" />
  </header>

  <!-- Middle: 75% height -->
  {#if mode === 'new_radix' || mode === 'settings'}
    <!-- Left 20% + middle stretched to 80% -->
    <section class="row-span-1 grid gap-y-3 px-3 pb-3 overflow-hidden w-full" style:grid-template-columns="minmax(0,20%) minmax(0,80%)">
      <!-- Left single panel -->
      <div class="h-full min-w-0 px-1 flex flex-col gap-2 min-h-0">
        <div class="min-h-0 flex-1">
          <ExpandablePanel title={mode === 'settings' ? 'Settings' : 'New Radix'} editable={false}>
            {#snippet children()}
              {#if mode === 'new_radix'}
                <div class="text-sm opacity-85">Use the center panel to create a new radix.</div>
                <div class="mt-4">
                  <div class="text-sm font-medium opacity-85 mb-2">Existing contexts</div>
                  <ul class="space-y-1 max-h-40 overflow-auto pr-1">
                    {#each layout.contexts as c}
                      <li class="flex items-center justify-between text-sm">
                        <span class:font-semibold={layout.selectedContext === c}>{c}</span>
                        {#if layout.selectedContext === c}
                          <span class="text-xs opacity-70">selected</span>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                </div>
              {:else}
                <div class="space-y-2 text-sm">
                  <p>Application settings</p>
                  <div class="h-32 rounded border border-dashed bg-muted/40"></div>
                </div>
              {/if}
            {/snippet}
          </ExpandablePanel>
        </div>
      </div>

      <!-- Middle content spans remaining width -->
      <div class="h-full min-w-0 px-1">
        {#if mode === 'new_radix'}
          <div class="h-full w-full rounded-md border bg-card text-card-foreground shadow-sm p-4 flex flex-col items-start justify-start">
            <h2 class="text-lg font-semibold mb-3">New Radix</h2>
            <form class="space-y-3 w-full max-w-md" onsubmit={submitNewContext}>
              <div class="space-y-1">
                <label class="block text-sm opacity-85" for="ctxNameCenter">New context name</label>
                <input
                  id="ctxNameCenter"
                  type="text"
                  class="w-full h-9 px-3 rounded-md bg-background text-foreground border"
                  bind:value={newContextName}
                  placeholder="e.g. John Doe"
                />
              </div>
              <div class="flex gap-2">
                <button type="submit" class="px-3 py-1.5 rounded-md bg-primary text-primary-foreground hover:opacity-90">Add</button>
                <button type="button" class="px-3 py-1.5 rounded-md bg-transparent border hover:bg-white/10" onclick={() => (newContextName = '')}>Clear</button>
              </div>
            </form>
          </div>
        {:else}
          <MiddleContent />
        {/if}
      </div>
    </section>
  {:else if mode === 'radix_table'}
    <!-- Left 20% (1 panel) + middle stretched to 80% -->
    <section class="row-span-1 grid gap-y-3 px-3 pb-3 overflow-hidden w-full" style:grid-template-columns="minmax(0,20%) minmax(0,80%)">
      <div class="h-full min-w-0 px-1 flex flex-col gap-2 min-h-0">
        <div class="min-h-0" class:flex-1={leftTopExpanded}>
          <ExpandablePanel title="Table Tools" bind:expanded={leftTopExpanded}>
            {#snippet children()}
              <div class="space-y-2 text-sm">
                <p>Table filters and helpers.</p>
                <div class="h-24 rounded border border-dashed bg-muted/40"></div>
              </div>
            {/snippet}
          </ExpandablePanel>
        </div>
      </div>
      <div class="h-full min-w-0 px-1">
        <MiddleContent />
      </div>
    </section>
  {:else}
    <!-- radix_view and radix_transits: fixed split 20% / 60% / 20% -->
    <section class="row-span-1 grid gap-y-3 px-3 pb-3 overflow-hidden w-full" style:grid-template-columns="minmax(0,20%) minmax(0,60%) minmax(0,20%)">
      <!-- Left column: stack three panels -->
      <div class="h-full min-w-0 px-1 flex flex-col gap-2 min-h-0">
        <!-- Panel 1: title is current context name -->
        <div class="min-h-0" class:flex-1={leftTopExpanded}>
          <ExpandablePanel title={layout.selectedContext} bind:expanded={leftTopExpanded}>
            {#snippet children()}
              <div class="space-y-2 text-sm">
                <p>Primary context tools.</p>
                <div class="h-32 rounded border border-dashed bg-muted/40"></div>
              </div>
            {/snippet}
          </ExpandablePanel>
        </div>
        <!-- Panel 2: Astrolab -->
        <div class="min-h-0" class:flex-1={leftMiddleExpanded}>
          <ExpandablePanel title="Astrolab" bind:expanded={leftMiddleExpanded}>
            {#snippet children()}
              <div class="space-y-2 text-sm">
                <p>Astrolab utilities.</p>
                <div class="h-24 rounded border border-dashed bg-muted/40"></div>
              </div>
            {/snippet}
          </ExpandablePanel>
        </div>
        <!-- Panel 3: Transits (folded by default) -->
        <div class="min-h-0" class:flex-1={leftBottomExpanded}>
          <ExpandablePanel title="Transits" bind:expanded={leftBottomExpanded}>
            {#snippet children()}
              <div class="space-y-2 text-sm">
                <p>Transits tools.</p>
                <div class="h-24 rounded border border-dashed bg-muted/40"></div>
              </div>
            {/snippet}
          </ExpandablePanel>
        </div>
      </div>

      <!-- Middle content -->
      <div class="h-full min-w-0 px-1">
        <MiddleContent />
      </div>

      <!-- Right panel -->
      <div class="h-full min-w-0 px-1">
        <ExpandablePanel title="Right Panel" bind:expanded={rightExpanded}>
          {#snippet children()}
            <div class="space-y-2 text-sm">
              <p>Expandable content (right). Put properties, logs, etc.</p>
              <div class="h-40 rounded border border-dashed bg-muted/40"></div>
              <div class="h-24 rounded border border-dashed bg-muted/40"></div>
            </div>
          {/snippet}
        </ExpandablePanel>
      </div>
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
