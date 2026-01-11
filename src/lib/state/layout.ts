// Wrapper module without runes. Re-export from the Svelte-compiled source.
// This file has a .svelte.ts extension so the Svelte plugin transforms runes.
export { 
  layout, 
  tabs, 
  type Tab, 
  type Mode, 
  type ChartData,
  setMode, 
  showOpenExportOverlay, 
  addContext,
  loadChartsFromWorkspace,
  getSelectedChart,
  updateChartComputation
} from './layout.svelte';
