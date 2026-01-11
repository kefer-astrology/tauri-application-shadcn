// Glyph Management Store - Svelte 5 runes-based
// Manages default and custom glyphs for planets, zodiac signs, etc.

export interface GlyphDefinition {
  id: string;
  name: string;
  type: 'planet' | 'zodiac' | 'aspect' | 'custom';
  svg: string; // SVG content or path
  isCustom: boolean;
  customPath?: string; // Path to custom uploaded file
}

// Default glyphs (using Unicode symbols as fallback, can be replaced with SVG)
const defaultGlyphs: Record<string, GlyphDefinition> = {
  // Planets
  sun: { id: 'sun', name: 'Sun', type: 'planet', svg: '☉', isCustom: false },
  moon: { id: 'moon', name: 'Moon', type: 'planet', svg: '☽', isCustom: false },
  mercury: { id: 'mercury', name: 'Mercury', type: 'planet', svg: '☿', isCustom: false },
  venus: { id: 'venus', name: 'Venus', type: 'planet', svg: '♀', isCustom: false },
  mars: { id: 'mars', name: 'Mars', type: 'planet', svg: '♂', isCustom: false },
  jupiter: { id: 'jupiter', name: 'Jupiter', type: 'planet', svg: '♃', isCustom: false },
  saturn: { id: 'saturn', name: 'Saturn', type: 'planet', svg: '♄', isCustom: false },
  uranus: { id: 'uranus', name: 'Uranus', type: 'planet', svg: '♅', isCustom: false },
  neptune: { id: 'neptune', name: 'Neptune', type: 'planet', svg: '♆', isCustom: false },
  pluto: { id: 'pluto', name: 'Pluto', type: 'planet', svg: '♇', isCustom: false },
  
  // Zodiac signs
  aries: { id: 'aries', name: 'Aries', type: 'zodiac', svg: '♈', isCustom: false },
  taurus: { id: 'taurus', name: 'Taurus', type: 'zodiac', svg: '♉', isCustom: false },
  gemini: { id: 'gemini', name: 'Gemini', type: 'zodiac', svg: '♊', isCustom: false },
  cancer: { id: 'cancer', name: 'Cancer', type: 'zodiac', svg: '♋', isCustom: false },
  leo: { id: 'leo', name: 'Leo', type: 'zodiac', svg: '♌', isCustom: false },
  virgo: { id: 'virgo', name: 'Virgo', type: 'zodiac', svg: '♍', isCustom: false },
  libra: { id: 'libra', name: 'Libra', type: 'zodiac', svg: '♎', isCustom: false },
  scorpio: { id: 'scorpio', name: 'Scorpio', type: 'zodiac', svg: '♏', isCustom: false },
  sagittarius: { id: 'sagittarius', name: 'Sagittarius', type: 'zodiac', svg: '♐', isCustom: false },
  capricorn: { id: 'capricorn', name: 'Capricorn', type: 'zodiac', svg: '♑', isCustom: false },
  aquarius: { id: 'aquarius', name: 'Aquarius', type: 'zodiac', svg: '♒', isCustom: false },
  pisces: { id: 'pisces', name: 'Pisces', type: 'zodiac', svg: '♓', isCustom: false },
};

// Store for all glyphs (default + custom)
export const glyphs = $state<Record<string, GlyphDefinition>>({ ...defaultGlyphs });

// Get glyph by ID
export function getGlyph(id: string): GlyphDefinition | undefined {
  return glyphs[id];
}

// Get glyph SVG content (handles both inline SVG and file paths)
export function getGlyphSvg(id: string): string {
  const glyph = glyphs[id];
  if (!glyph) return '';
  
  // If it's a custom glyph with a path, we'd need to load it
  // For now, return the SVG content directly
  return glyph.svg;
}

// Helper to check if content is SVG markup
function isSvgMarkup(content: string): boolean {
  return content.trim().startsWith('<svg') || content.trim().startsWith('<?xml');
}

// Get glyph content (handles both Unicode and SVG markup)
export function getGlyphContent(id: string): { type: 'unicode' | 'svg'; content: string } {
  const glyph = glyphs[id];
  if (!glyph) return { type: 'unicode', content: '' };
  
  const svg = glyph.svg;
  if (isSvgMarkup(svg)) {
    return { type: 'svg', content: svg };
  }
  return { type: 'unicode', content: svg };
}

// Add or update a custom glyph
export function setCustomGlyph(id: string, name: string, svg: string, type: 'planet' | 'zodiac' | 'aspect' | 'custom' = 'custom', customPath?: string) {
  glyphs[id] = {
    id,
    name,
    type,
    svg,
    isCustom: true,
    customPath,
  };
  
  // Persist to localStorage
  try {
    const customGlyphs = Object.values(glyphs)
      .filter(g => g.isCustom)
      .reduce((acc, g) => {
        acc[g.id] = g;
        return acc;
      }, {} as Record<string, GlyphDefinition>);
    localStorage.setItem('custom_glyphs', JSON.stringify(customGlyphs));
  } catch (e) {
    console.warn('Failed to save custom glyphs:', e);
  }
}

// Load custom glyphs from localStorage
export function loadCustomGlyphs() {
  try {
    const stored = localStorage.getItem('custom_glyphs');
    if (stored) {
      const customGlyphs = JSON.parse(stored) as Record<string, GlyphDefinition>;
      Object.assign(glyphs, customGlyphs);
    }
  } catch (e) {
    console.warn('Failed to load custom glyphs:', e);
  }
}

// Initialize: load custom glyphs on module load
loadCustomGlyphs();
