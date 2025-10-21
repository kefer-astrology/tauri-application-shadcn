<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  let { title = '', expanded = $bindable(true), editable = true, onEdit = undefined, children = undefined, footer = undefined } = $props();
  const dispatch = createEventDispatcher<{ toggle: boolean }>();

  function toggle() {
    expanded = !expanded;
    dispatch('toggle', expanded);
  }

  function edit() {
    onEdit?.();
  }
</script>

<div class="w-full flex flex-col max-h-full overflow-hidden rounded-md border shadow-sm" aria-expanded={expanded}>
  <!-- Header: #002e60 background, expander on left, title, then edit icon -->
  <div class="flex items-center justify-between px-3 py-2 select-none" style="background-color:#002e60;color:var(--color-primary-foreground,white)">
    <div class="flex items-center gap-2 min-w-0">
      <button class="inline-flex h-7 w-7 items-center justify-center rounded-sm hover:opacity-85 transition-opacity" onclick={toggle} aria-label="Toggle panel">
        {#if expanded}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18 15-6-6-6 6"/></svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
        {/if}
      </button>
      <div class="font-semibold truncate">{title}</div>
    </div>
    {#if editable}
      <button class="inline-flex h-7 w-7 items-center justify-center rounded-sm hover:opacity-85 transition-opacity" onclick={edit} aria-label="Edit widget">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 1 1 3 3L7 19l-4 1 1-4Z"/></svg>
      </button>
    {/if}
  </div>

  {#if expanded}
    <!-- Body -->
    <div class="flex-1 min-h-0 overflow-auto bg-background/50 backdrop-blur-sm">
      <div class="p-3">
        {@render children?.()}
      </div>
    </div>

    <!-- Footer -->
    <div class="px-3 py-2 bg-primary text-primary-foreground select-none">
      {@render footer?.()}
    </div>
  {/if}
</div>
