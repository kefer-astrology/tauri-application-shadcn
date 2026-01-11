// Runes global state (no stores, no classes)

export const tabs = ['Radix', 'Aspects', 'Transits', 'Settings', 'About'] as const;
export type Tab = (typeof tabs)[number];

// Chart data structure
export interface ChartData {
  id: string;
  name: string;
  chartType: string;
  dateTime: string;
  location: string;
  tags: string[];
  // Full chart settings
  houseSystem?: string | null;
  zodiacType?: string;
  engine?: string | null;
  model?: string | null;
  overrideEphemeris?: string | null;
  latitude?: number;
  longitude?: number;
  timezone?: string;
  computed?: {
    positions?: Record<string, number>;
    aspects?: any[];
  };
}

export type Mode =
  | 'new_radix'
  | 'open'
  | 'radix_view'
  | 'radix_table'
  | 'radix_transits'
  | 'info'
  | 'dynamic'
  | 'revolution'
  | 'favorite'
  | 'settings'
  | 'export';

export const layout = $state({
  selectedTab: tabs[0] as Tab,
  selectedContext: '' as string,
  contexts: [] as ChartData[],
  workspacePath: null as string | null,
  leftExpanded: true,
  rightExpanded: true,
  mode: 'radix_view' as Mode,
  prevMode: 'radix_view' as Mode,
  overlay: {
    openExport: false,
  },
});

export function setMode(next: Mode) {
  if (layout.mode !== next) {
    layout.prevMode = layout.mode as Mode;
    layout.mode = next;
  }
}

export function showOpenExportOverlay(show: boolean) {
  layout.overlay.openExport = show;
}

export function addContext(name: string) {
  const n = name?.trim();
  if (!n) return;
  // Check if context with this name already exists
  const existing = layout.contexts.find(c => c.name === n);
  if (!existing) {
    const newChart: ChartData = {
      id: n.toLowerCase().replace(/\s+/g, '-'),
      name: n,
      chartType: 'NATAL',
      dateTime: '',
      location: '',
      tags: [],
    };
    layout.contexts = [...layout.contexts, newChart];
  }
  layout.selectedContext = n;
}

export function loadChartsFromWorkspace(charts: ChartData[]) {
  layout.contexts = charts;
  if (charts.length > 0) {
    layout.selectedContext = charts[0].name;
  }
}

export function getSelectedChart(): ChartData | undefined {
  return layout.contexts.find(c => c.name === layout.selectedContext);
}

export function updateChartComputation(chartId: string, computed: ChartData['computed']) {
  const chart = layout.contexts.find(c => c.id === chartId);
  if (chart) {
    chart.computed = computed;
    layout.contexts = [...layout.contexts]; // Trigger reactivity
  }
}
