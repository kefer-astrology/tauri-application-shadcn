<script lang="ts">
  import { layout } from '$lib/state/layout';

  function selectContext(c: string) {
    layout.selectedContext = c;
  }
</script>

<div class="w-full h-full flex items-center justify-center">
  <nav class="flex items-center gap-3 px-3 py-1.5 rounded-md">
    {#each layout.contexts as c}
      {#if layout.selectedContext === c}
        <button
          class="px-2 py-1.5 text-sm font-semibold text-white underline underline-offset-4 rounded-md hover:bg-white/10 transition-colors"
          aria-current="page"
          onclick={() => selectContext(c)}
        >{c}</button>
      {:else}
        <button
          class="px-2 py-1.5 text-sm text-white rounded-md hover:bg-white/10 transition-colors"
          onclick={() => selectContext(c)}
        >{c}</button>
      {/if}
    {/each}
  </nav>
</div>
