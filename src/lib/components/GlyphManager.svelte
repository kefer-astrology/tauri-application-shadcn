<!-- Glyph Manager Component - For uploading and managing custom glyphs -->
<script lang="ts">
  import { glyphs, setCustomGlyph, getGlyph } from '$lib/stores/glyphs.svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as Dialog from '$lib/components/ui/dialog/index.js';
  
  let open = $state(false);
  let selectedGlyphId = $state<string | null>(null);
  let uploadError = $state<string | null>(null);
  
  // Get all glyphs grouped by type
  const glyphsByType = $derived({
    planet: Object.values(glyphs).filter(g => g.type === 'planet'),
    zodiac: Object.values(glyphs).filter(g => g.type === 'zodiac'),
    custom: Object.values(glyphs).filter(g => g.isCustom),
  });
  
  function handleFileUpload(event: Event, glyphId: string) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    
    // Validate it's an SVG
    if (!file.name.endsWith('.svg') && !file.type.includes('svg')) {
      uploadError = 'Please upload an SVG file';
      return;
    }
    
    uploadError = null;
    
    // Read file content
    const reader = new FileReader();
    reader.onload = (e) => {
      const svgContent = e.target?.result as string;
      if (svgContent) {
        const glyph = getGlyph(glyphId);
        setCustomGlyph(
          glyphId,
          glyph?.name || glyphId,
          svgContent,
          glyph?.type || 'custom',
          file.name
        );
        selectedGlyphId = null;
      }
    };
    reader.onerror = () => {
      uploadError = 'Failed to read file';
    };
    reader.readAsText(file);
  }
  
  function resetGlyph(glyphId: string) {
    // Reset to default (remove from custom glyphs)
    const defaultGlyphs = {
      sun: { id: 'sun', name: 'Sun', type: 'planet' as const, svg: '☉', isCustom: false },
      moon: { id: 'moon', name: 'Moon', type: 'planet' as const, svg: '☽', isCustom: false },
      mercury: { id: 'mercury', name: 'Mercury', type: 'planet' as const, svg: '☿', isCustom: false },
      venus: { id: 'venus', name: 'Venus', type: 'planet' as const, svg: '♀', isCustom: false },
      mars: { id: 'mars', name: 'Mars', type: 'planet' as const, svg: '♂', isCustom: false },
      jupiter: { id: 'jupiter', name: 'Jupiter', type: 'planet' as const, svg: '♃', isCustom: false },
      saturn: { id: 'saturn', name: 'Saturn', type: 'planet' as const, svg: '♄', isCustom: false },
      uranus: { id: 'uranus', name: 'Uranus', type: 'planet' as const, svg: '♅', isCustom: false },
      neptune: { id: 'neptune', name: 'Neptune', type: 'planet' as const, svg: '♆', isCustom: false },
      pluto: { id: 'pluto', name: 'Pluto', type: 'planet' as const, svg: '♇', isCustom: false },
      aries: { id: 'aries', name: 'Aries', type: 'zodiac' as const, svg: '♈', isCustom: false },
      taurus: { id: 'taurus', name: 'Taurus', type: 'zodiac' as const, svg: '♉', isCustom: false },
      gemini: { id: 'gemini', name: 'Gemini', type: 'zodiac' as const, svg: '♊', isCustom: false },
      cancer: { id: 'cancer', name: 'Cancer', type: 'zodiac' as const, svg: '♋', isCustom: false },
      leo: { id: 'leo', name: 'Leo', type: 'zodiac' as const, svg: '♌', isCustom: false },
      virgo: { id: 'virgo', name: 'Virgo', type: 'zodiac' as const, svg: '♍', isCustom: false },
      libra: { id: 'libra', name: 'Libra', type: 'zodiac' as const, svg: '♎', isCustom: false },
      scorpio: { id: 'scorpio', name: 'Scorpio', type: 'zodiac' as const, svg: '♏', isCustom: false },
      sagittarius: { id: 'sagittarius', name: 'Sagittarius', type: 'zodiac' as const, svg: '♐', isCustom: false },
      capricorn: { id: 'capricorn', name: 'Capricorn', type: 'zodiac' as const, svg: '♑', isCustom: false },
      aquarius: { id: 'aquarius', name: 'Aquarius', type: 'zodiac' as const, svg: '♒', isCustom: false },
      pisces: { id: 'pisces', name: 'Pisces', type: 'zodiac' as const, svg: '♓', isCustom: false },
    };
    
    if (defaultGlyphs[glyphId as keyof typeof defaultGlyphs]) {
      glyphs[glyphId] = defaultGlyphs[glyphId as keyof typeof defaultGlyphs];
      // Remove from localStorage
      try {
        const stored = localStorage.getItem('custom_glyphs');
        if (stored) {
          const customGlyphs = JSON.parse(stored) as Record<string, any>;
          delete customGlyphs[glyphId];
          localStorage.setItem('custom_glyphs', JSON.stringify(customGlyphs));
        }
      } catch (e) {
        console.warn('Failed to update custom glyphs:', e);
      }
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Trigger>
    <Button variant="outline" size="sm">Manage Glyphs</Button>
  </Dialog.Trigger>
  <Dialog.Content class="max-w-2xl max-h-[80vh] overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Glyph Management</Dialog.Title>
      <Dialog.Description>
        Upload custom SVG glyphs to replace default symbols. SVG files only.
      </Dialog.Description>
    </Dialog.Header>
    
    {#if uploadError}
      <div class="text-sm text-destructive mb-4">{uploadError}</div>
    {/if}
    
    <!-- Planets -->
    <div class="mb-6">
      <h3 class="text-sm font-semibold mb-3">Planets</h3>
      <div class="grid grid-cols-2 gap-2">
        {#each glyphsByType.planet as glyph}
          <div class="flex items-center justify-between p-2 border rounded">
            <div class="flex items-center gap-2">
              <span class="text-2xl">{glyph.svg}</span>
              <span class="text-sm">{glyph.name}</span>
              {#if glyph.isCustom}
                <span class="text-xs text-muted-foreground">(custom)</span>
              {/if}
            </div>
            <div class="flex gap-1">
              <label class="cursor-pointer">
                <input
                  type="file"
                  accept=".svg,image/svg+xml"
                  class="hidden"
                  onchange={(e) => handleFileUpload(e, glyph.id)}
                />
                <Button variant="ghost" size="sm" type="button">Upload</Button>
              </label>
              {#if glyph.isCustom}
                <Button variant="ghost" size="sm" onclick={() => resetGlyph(glyph.id)}>Reset</Button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
    
    <!-- Zodiac Signs -->
    <div class="mb-6">
      <h3 class="text-sm font-semibold mb-3">Zodiac Signs</h3>
      <div class="grid grid-cols-2 gap-2">
        {#each glyphsByType.zodiac as glyph}
          <div class="flex items-center justify-between p-2 border rounded">
            <div class="flex items-center gap-2">
              <span class="text-2xl">{glyph.svg}</span>
              <span class="text-sm">{glyph.name}</span>
              {#if glyph.isCustom}
                <span class="text-xs text-muted-foreground">(custom)</span>
              {/if}
            </div>
            <div class="flex gap-1">
              <label class="cursor-pointer">
                <input
                  type="file"
                  accept=".svg,image/svg+xml"
                  class="hidden"
                  onchange={(e) => handleFileUpload(e, glyph.id)}
                />
                <Button variant="ghost" size="sm" type="button">Upload</Button>
              </label>
              {#if glyph.isCustom}
                <Button variant="ghost" size="sm" onclick={() => resetGlyph(glyph.id)}>Reset</Button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
    
    <Dialog.Footer>
      <Button onclick={() => open = false}>Close</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
