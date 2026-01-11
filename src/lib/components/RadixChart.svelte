<!-- Radix Chart Component - Interactive SVG polar chart -->
<script lang="ts">
  import { effectiveTime } from '$lib/stores/timeNavigation.svelte';
  import { glyphs } from '$lib/stores/glyphs.svelte';
  
  // Props
  interface Props {
    size?: number;
    planetPositions?: Record<string, { degrees: number; sign: string; house?: number }>; // planet -> position data
    houseCusps?: number[]; // 12 house cusp positions in degrees (0-360)
    aspects?: Array<{ from: string; to: string; type: 'conjunction' | 'sextile' | 'square' | 'trine' | 'opposition' }>;
    infoBlobs?: Array<{ degrees: number; content: string }>; // 3 information blobs
  }
  
  let { 
    size = 400,
    planetPositions = {},
    houseCusps = [],
    aspects = [],
    infoBlobs = []
  }: Props = $props();
  
  // Default info blobs if not provided (3 blobs at 120-degree intervals)
  const defaultInfoBlobs = $derived(
    infoBlobs.length > 0
      ? infoBlobs
      : [
          { degrees: 0, content: 'Info 1' },
          { degrees: 120, content: 'Info 2' },
          { degrees: 240, content: 'Info 3' }
        ]
  );
  
  // Reactive to time changes
  const currentTime = $derived(effectiveTime());
  
  // Chart dimensions - divided into 10 parts from outer to center
  const centerX = $derived(size / 2);
  const centerY = $derived(size / 2);
  const maxRadius = $derived(size * 0.48); // Maximum chart radius
  
  // Divide radius into 10 parts
  const partSize = $derived(maxRadius / 10);
  
  // Radii from outer to center:
  const radius1 = $derived(maxRadius); // 1 - Outer border
  const radius2 = $derived(maxRadius - partSize); // 2 - Zodiac strip (colored background + sign glyphs)
  const radius3 = $derived(maxRadius - partSize * 2); // 3 - Planet glyph position
  const radius4 = $derived(maxRadius - partSize * 3); // 4 - Precise angle text
  const radius5 = $derived(maxRadius - partSize * 4); // 5 - Sign glyph position
  const radius6 = $derived(maxRadius - partSize * 5); // 6 - Another degree text
  const radius7 = $derived(maxRadius - partSize * 6); // 7 - House numbers strip
  const radius8 = $derived(maxRadius - partSize * 7); // 8 - Dark center start
  const radius9 = $derived(maxRadius - partSize * 8); // 9 - Dark center middle
  const radius10 = $derived(maxRadius - partSize * 9); // 10 - Dark center inner (aspects)
  
  // Convenience radii
  const outerBorderRadius = $derived(radius1);
  const zodiacStripOuter = $derived(radius1);
  const zodiacStripInner = $derived(radius2);
  const planetGlyphRadius = $derived(radius3);
  const angleTextRadius = $derived(radius4);
  const signGlyphRadius = $derived(radius5);
  const degreeTextRadius = $derived(radius6);
  const houseStripOuter = $derived(radius6);
  const houseStripInner = $derived(radius7);
  const centerDarkOuter = $derived(radius7);
  const centerDarkInner = $derived(radius10);
  const aspectRadius = $derived(radius10 * 0.8); // Aspect lines inside dark center
  
  // Zodiac sign colors (alternating pattern)
  const zodiacColors = [
    '#2d5016', // dark green - Pisces
    '#2d5016', // dark green - Aries
    '#6b46c1', // purple - Taurus
    '#3b82f6', // light blue - Gemini
    '#3b82f6', // light blue - Cancer
    '#6b46c1', // purple - Leo
    '#2d5016', // dark green - Virgo
    '#2d5016', // dark green - Libra
    '#6b46c1', // purple - Scorpio
    '#3b82f6', // light blue - Sagittarius
    '#3b82f6', // light blue - Capricorn
    '#2d5016'  // dark green - Aquarius
  ];
  
  // Zodiac sign positions (middle of each 30-degree section)
  const zodiacSigns = [
    { name: 'pisces', degrees: 345 },
    { name: 'aries', degrees: 15 },
    { name: 'taurus', degrees: 45 },
    { name: 'gemini', degrees: 75 },
    { name: 'cancer', degrees: 105 },
    { name: 'leo', degrees: 135 },
    { name: 'virgo', degrees: 165 },
    { name: 'libra', degrees: 195 },
    { name: 'scorpio', degrees: 225 },
    { name: 'sagittarius', degrees: 255 },
    { name: 'capricorn', degrees: 285 },
    { name: 'aquarius', degrees: 315 }
  ];
  
  // Default house cusps (every 30 degrees if not provided)
  const defaultHouseCusps = $derived(
    houseCusps.length === 12 
      ? houseCusps 
      : Array.from({ length: 12 }, (_, i) => i * 30)
  );
  
  // Default planet positions if not provided
  const defaultPlanets = $derived({
    sun: { degrees: 120, sign: '♒', house: 12 },
    moon: { degrees: 45, sign: '♒', house: 10 },
    mercury: { degrees: 100, sign: '♒', house: 8 },
    venus: { degrees: 200, sign: '♒', house: 6 },
    mars: { degrees: 300, sign: '♒', house: 4 },
    jupiter: { degrees: 40, sign: '♒', house: 2 }
  });
  
  const planets = $derived({ ...defaultPlanets, ...planetPositions });
  
  // Convert degrees to radians
  function degToRad(deg: number): number {
    return (deg - 90) * (Math.PI / 180); // -90 to start at top
  }
  
  // Convert degrees to SVG coordinates
  function polarToCartesian(radius: number, degrees: number): { x: number; y: number } {
    const rad = degToRad(degrees);
    return {
      x: centerX + radius * Math.cos(rad),
      y: centerY + radius * Math.sin(rad)
    };
  }
  
  // Create arc path for zodiac sections
  function createArc(startDeg: number, endDeg: number, innerR: number, outerR: number): string {
    const startRad = degToRad(startDeg);
    const endRad = degToRad(endDeg);
    const startInner = polarToCartesian(innerR, startDeg);
    const endInner = polarToCartesian(innerR, endDeg);
    const startOuter = polarToCartesian(outerR, startDeg);
    const endOuter = polarToCartesian(outerR, endDeg);
    
    const largeArc = endDeg - startDeg > 180 ? 1 : 0;
    
    return `M ${startInner.x} ${startInner.y} 
            L ${startOuter.x} ${startOuter.y} 
            A ${outerR} ${outerR} 0 ${largeArc} 1 ${endOuter.x} ${endOuter.y}
            L ${endInner.x} ${endInner.y}
            A ${innerR} ${innerR} 0 ${largeArc} 0 ${startInner.x} ${startInner.y} Z`;
  }
  
  
  // Get aspect line color
  function getAspectColor(type: string): string {
    switch (type) {
      case 'square':
      case 'opposition':
        return '#ef4444'; // red - challenging
      case 'trine':
      case 'sextile':
        return '#22c55e'; // green - harmonious
      case 'conjunction':
      default:
        return '#60a5fa'; // light blue - minor
    }
  }
  
  // Format planet degree display (precise angle)
  function formatPreciseAngle(planet: { degrees: number; sign: string }): string {
    const signDeg = planet.degrees % 30;
    const minutes = Math.floor((signDeg % 1) * 60);
    return `${Math.floor(signDeg)}° ${minutes}'`;
  }
  
  // Format another degree id
  function formatDegreeId(planet: { degrees: number; sign: string }): string {
    const signDeg = planet.degrees % 30;
    const minutes = Math.floor((signDeg % 1) * 60);
    return `${Math.floor(signDeg)}° ${planet.sign} ${minutes}°`;
  }
  
  // Helper to check if content is SVG markup
  function isSvgMarkup(content: string): boolean {
    return content.trim().startsWith('<svg') || content.trim().startsWith('<?xml');
  }
  
  // Get glyph content (handles both Unicode and SVG markup)
  function getGlyphContent(id: string): { type: 'unicode' | 'svg'; content: string } {
    const glyph = glyphs[id];
    if (!glyph) return { type: 'unicode', content: '' };
    
    const svg = glyph.svg;
    if (isSvgMarkup(svg)) {
      return { type: 'svg', content: svg };
    }
    return { type: 'unicode', content: svg };
  }
  
  // SVG element reference
  let svgElement: SVGElement | undefined;
</script>

<svg
  bind:this={svgElement}
  width={size}
  height={size}
  viewBox={`0 0 ${size} ${size}`}
  class="radix-chart"
  xmlns="http://www.w3.org/2000/svg"
>
  <!-- Part 1: Outer border (narrow) -->
  <circle
    cx={centerX}
    cy={centerY}
    r={outerBorderRadius}
    fill="none"
    stroke="currentColor"
    stroke-width={partSize * 0.3}
    opacity="0.4"
  />
  
  <!-- Part 2: Zodiac strip with colored background + sign glyphs -->
  {#each zodiacSigns as sign, i}
    {@const startDeg = sign.degrees - 15}
    {@const endDeg = sign.degrees + 15}
    <path
      d={createArc(startDeg, endDeg, zodiacStripInner, zodiacStripOuter)}
      fill={zodiacColors[i]}
      opacity="0.6"
    />
    {@const pos = polarToCartesian((zodiacStripInner + zodiacStripOuter) / 2, sign.degrees)}
    {@const glyphData = getGlyphContent(sign.name)}
    <g transform={`translate(${pos.x}, ${pos.y})`}>
      {#if glyphData.type === 'svg'}
        {@html glyphData.content}
      {:else}
        <text
          x="0"
          y="0"
          text-anchor="middle"
          dominant-baseline="middle"
          font-size={partSize * 0.8}
          fill="currentColor"
          opacity="0.9"
        >
          {glyphData.content || sign.name.charAt(0).toUpperCase()}
        </text>
      {/if}
    </g>
  {/each}
  
  <!-- Parts 3-6: Grey area between outer strip and center strip (ring shape) -->
  <defs>
    <mask id="greyRingMask">
      <rect width={size} height={size} fill="white"/>
      <circle cx={centerX} cy={centerY} r={houseStripOuter} fill="black"/>
    </mask>
  </defs>
  <circle
    cx={centerX}
    cy={centerY}
    r={zodiacStripInner}
    fill="rgba(128, 128, 128, 0.25)"
    stroke="none"
    mask="url(#greyRingMask)"
  />
  
  <!-- House cusp lines (from center to outer) -->
  {#each defaultHouseCusps as cusp}
    {@const outerPos = polarToCartesian(outerBorderRadius, cusp)}
    {@const innerPos = polarToCartesian(centerDarkOuter, cusp)}
    <line
      x1={innerPos.x}
      y1={innerPos.y}
      x2={outerPos.x}
      y2={outerPos.y}
      stroke="rgba(100, 100, 100, 0.3)"
      stroke-width="0.5"
    />
  {/each}
  
  <!-- Planets with all information (positions 3-6) -->
  {#each Object.entries(planets) as [planetName, planetData]}
    <!-- Position 3: Glyph of object (planet) -->
    {@const planetPos = polarToCartesian(planetGlyphRadius, planetData.degrees)}
    {@const planetGlyphData = getGlyphContent(planetName)}
    <g transform={`translate(${planetPos.x}, ${planetPos.y})`}>
      {#if planetGlyphData.type === 'svg'}
        <g class="planet-glyph" data-planet={planetName}>
          {@html planetGlyphData.content}
        </g>
      {:else}
        <text
          x="0"
          y="0"
          text-anchor="middle"
          dominant-baseline="middle"
          font-size={partSize * 1.2}
          fill="currentColor"
          class="planet-glyph"
          data-planet={planetName}
        >
          {planetGlyphData.content || planetName.charAt(0).toUpperCase()}
        </text>
      {/if}
    </g>
    
    <!-- Position 4: Precise angle -->
    {@const anglePos = polarToCartesian(angleTextRadius, planetData.degrees)}
    <text
      x={anglePos.x}
      y={anglePos.y}
      text-anchor="middle"
      dominant-baseline="middle"
      font-size={partSize * 0.5}
      fill="currentColor"
      opacity="0.8"
    >
      {formatPreciseAngle(planetData)}
    </text>
    
    <!-- Position 5: Sign glyph -->
    {@const signPos = polarToCartesian(signGlyphRadius, planetData.degrees)}
    {@const signGlyphData = getGlyphContent(planetData.sign.toLowerCase())}
    <g transform={`translate(${signPos.x}, ${signPos.y})`}>
      {#if signGlyphData.type === 'svg'}
        {@html signGlyphData.content}
      {:else}
        <text
          x="0"
          y="0"
          text-anchor="middle"
          dominant-baseline="middle"
          font-size={partSize * 0.8}
          fill="currentColor"
          opacity="0.9"
        >
          {signGlyphData.content || planetData.sign}
        </text>
      {/if}
    </g>
    
    <!-- Position 6: Another degree id -->
    {@const degreePos = polarToCartesian(degreeTextRadius, planetData.degrees)}
    <text
      x={degreePos.x}
      y={degreePos.y}
      text-anchor="middle"
      dominant-baseline="middle"
      font-size={partSize * 0.5}
      fill="currentColor"
      opacity="0.7"
    >
      {formatDegreeId(planetData)}
    </text>
  {/each}
  
  <!-- Part 7: Strip with background containing house numbers -->
  <circle
    cx={centerX}
    cy={centerY}
    r={houseStripOuter}
    fill="rgba(100, 100, 100, 0.3)"
    stroke="currentColor"
    stroke-width="1"
    opacity="0.4"
  />
  <circle
    cx={centerX}
    cy={centerY}
    r={houseStripInner}
    fill="rgba(0, 0, 0, 0.1)"
    stroke="none"
  />
  
  <!-- House numbers (in strip 7) -->
  {#each defaultHouseCusps as cusp, i}
    {@const houseNum = i + 1}
    {@const nextCusp = defaultHouseCusps[(i + 1) % 12] || cusp + 30}
    {@const midCusp = (cusp + nextCusp) / 2}
    {@const pos = polarToCartesian((houseStripInner + houseStripOuter) / 2, midCusp)}
    <text
      x={pos.x}
      y={pos.y}
      text-anchor="middle"
      dominant-baseline="middle"
      font-size={partSize * 0.7}
      fill="currentColor"
      opacity="0.9"
      font-weight="600"
    >
      {houseNum}
    </text>
  {/each}
  
  <!-- Parts 8-10: Dark center piece with relations (aspects) -->
  <circle
    cx={centerX}
    cy={centerY}
    r={centerDarkOuter}
    fill="rgba(0, 0, 0, 0.4)"
    stroke="currentColor"
    stroke-width="1"
    opacity="0.5"
  />
  
  <!-- Aspect/Transit lines (inside dark center) -->
  {#each aspects as aspect}
    {@const fromPlanet = planets[aspect.from]}
    {@const toPlanet = planets[aspect.to]}
    {#if fromPlanet && toPlanet}
      {@const fromPos = polarToCartesian(aspectRadius, fromPlanet.degrees)}
      {@const toPos = polarToCartesian(aspectRadius, toPlanet.degrees)}
      <line
        x1={fromPos.x}
        y1={fromPos.y}
        x2={toPos.x}
        y2={toPos.y}
        stroke={getAspectColor(aspect.type)}
        stroke-width={partSize * 0.1}
        opacity="0.8"
      />
    {/if}
  {/each}
  
  <!-- Center point -->
  <circle
    cx={centerX}
    cy={centerY}
    r={partSize * 0.2}
    fill="currentColor"
    opacity="0.6"
  />
</svg>

<style>
  .radix-chart {
    max-width: 100%;
    max-height: 100%;
  }
  
  .planet-glyph {
    cursor: pointer;
    transition: opacity 0.2s;
  }
  
  .planet-glyph:hover {
    opacity: 0.7;
  }
</style>
