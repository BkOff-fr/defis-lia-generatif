// Mode démo web (C36) — fixtures précalculées par le VRAI moteur Monte-Carlo.
//
// Pourquoi ce module existe : le CDC impose une démo web (plateforme 2ᵉ
// classe, bloquante v1.0), mais CLAUDE.md §13 interdit les données factices.
// Résolution : les fixtures de ce dossier sont générées par
// `sobria-estimator` lui-même (seed 42, N = 10 000, horodatage figé) via le
// générateur documenté dans `briefs/chantiers/C36-mode-demo-web.md`.
// Aucune valeur n'est inventée côté TypeScript : ce module ne fait que
// servir, adapter (snake_case core → DTO) et agréger arithmétiquement des
// résultats du moteur Rust.
//
// Activation : uniquement hors contexte Tauri (`!isTauriContext()`), donc
// jamais dans l'application de bureau. Chaque page affiche la bannière
// `DemoBanner` et chaque résultat embarque une hypothèse `mode_demo`.
//
// Commandes couvertes : estimer (M1), comparer (M3), bibliothèque (M9),
// datacenters (M12), dashboard (M15), méthodologies (M14), préférences.
// Les autres commandes rejettent `tauri_unavailable` avec un message
// orienté utilisateur (cf. `api.ts::DESKTOP_ONLY_MESSAGE`).

import type {
  AppPreferencesDto,
  CountryAggregateDto,
  DashboardPeriod,
  DashboardSummaryDto,
  DatacenterSummaryDto,
  EmpreinteMethod,
  EstimationRequestDto,
  EstimationResultDto,
  IndicatorDto,
  IndicatorName,
  MetaInfo,
  MethodologyInfoDto,
  ModelDetailDto,
  ModelPresetDto,
  ReferentielStatusDto,
  TripletDto,
  VendorComparisonRowDto,
  VendorDisclosureDto
} from '../api';

import estimatesRaw from './fixtures/estimates.json';
import datacentersRaw from './fixtures/datacenters.json';
import methodologiesRaw from './fixtures/methodologies.json';
import modelsRaw from './fixtures/models.json';

// ─── Formes brutes (sérialisation serde des types core/estimator) ────────

interface RawInterval {
  p5: number;
  p50: number;
  p95: number;
}

interface RawIndicator {
  indicator: string;
  interval: RawInterval;
  unit: string;
  bins?: { min: number; max: number; counts: number[] };
}

interface RawEstimate {
  audit_id: number;
  computed_at: string;
  equivalents: { label: string; value: number; source: string }[];
  hypotheses: { key: string; value: unknown; source: string }[];
  indicators: RawIndicator[];
  method: EmpreinteMethod;
  request: {
    model_id: string;
    tokens_in: number;
    tokens_out_estimated: number;
    timestamp: string;
  };
  seed: number;
}

interface RawPreset {
  id: string;
  display_name: string;
  provider: string;
  family: string;
  approx_params_billions: number;
  openness: ModelPresetDto['openness'];
  calibration: ModelPresetDto['calibration'];
  sources: string[];
  release_date: string;
  active_params_b: number;
  model_family: ModelPresetDto['model_family'];
  architecture: { kind: ModelPresetDto['architecture'] };
  vision_capable: boolean;
  audio_capable: boolean;
  reasoning_capable: boolean;
  thinking_token_multiplier: [number, number] | null;
  default_context_overhead_tokens: number;
  deprecated: boolean;
  source_url: string;
  epsilon_prefill_mj: RawInterval;
  epsilon_decode_mj: RawInterval;
  embodied_g_per_req: RawInterval;
  vendor_disclosures: VendorDisclosureDto[];
}

const estimates = estimatesRaw as unknown as RawEstimate[];
const presets = modelsRaw as unknown as RawPreset[];
const methodologies = methodologiesRaw as unknown as MethodologyInfoDto[];
const datacenters = (datacentersRaw as unknown as { datacenters: DatacenterSummaryDto[] })
  .datacenters;

/** Modèles couverts par les fixtures d'estimation. */
const DEMO_MODEL_IDS = [...new Set(estimates.map((e) => e.request.model_id))];

// ─── Adaptateurs core → DTO (mirror de `dto.rs`, conversions triviales) ──

function toIndicator(raw: RawIndicator): IndicatorDto {
  return {
    // serde sérialise l'enum core `co2_eq` ; le DTO Tauri expose `co2eq`.
    indicator: (raw.indicator === 'co2_eq' ? 'co2eq' : raw.indicator) as IndicatorName,
    p5: raw.interval.p5,
    p50: raw.interval.p50,
    p95: raw.interval.p95,
    unit: raw.unit,
    ...(raw.bins ? { bins: raw.bins } : {})
  };
}

function toResult(raw: RawEstimate): EstimationResultDto {
  return {
    method: raw.method,
    request: { ...raw.request },
    indicators: raw.indicators.map(toIndicator),
    equivalents: raw.equivalents,
    hypotheses: [
      ...raw.hypotheses,
      {
        key: 'mode_demo',
        value:
          'Résultat précalculé par sobria-estimator (seed 42, N=10 000) au ' +
          'point de grille le plus proche de votre requête. Installez ' +
          "l'application pour des estimations exactes sur vos propres prompts.",
        source: 'C36 — mode démo web, fixtures générées par le moteur Rust'
      }
    ],
    computed_at: raw.computed_at,
    seed: raw.seed,
    audit_id: 0
  };
}

function toPreset(raw: RawPreset): ModelPresetDto {
  return {
    id: raw.id,
    display_name: raw.display_name,
    provider: raw.provider,
    family: raw.family,
    approx_params_billions: raw.approx_params_billions,
    openness: raw.openness,
    calibration: raw.calibration,
    sources: raw.sources,
    release_date: raw.release_date,
    active_params_b: raw.active_params_b,
    model_family: raw.model_family,
    architecture: raw.architecture.kind,
    vision_capable: raw.vision_capable,
    audio_capable: raw.audio_capable,
    reasoning_capable: raw.reasoning_capable,
    ...(raw.thinking_token_multiplier
      ? { thinking_token_multiplier: raw.thinking_token_multiplier }
      : {}),
    default_context_overhead_tokens: raw.default_context_overhead_tokens,
    deprecated: raw.deprecated,
    source_url: raw.source_url
  };
}

// ─── Erreur "app de bureau requise" (forme `{code, message}` brute, ──────
// normalisée en `SobriaIpcError` par `api.ts::normalizeError`).

function desktopOnly(feature: string): never {
  throw {
    code: 'tauri_unavailable',
    message:
      `${feature} nécessite l'application de bureau Sobr.ia (calcul local, ` +
      'ledger d’audit chiffré). La démo web couvre : Estimer, Comparer, ' +
      'Bibliothèque de modèles, Datacenters et Tableau de bord (données d’exemple).'
  };
}

// ─── Estimation : point de grille le plus proche ─────────────────────────

function nearestEstimate(req: EstimationRequestDto, method: EmpreinteMethod): EstimationResultDto {
  const candidates = estimates.filter(
    (e) => e.request.model_id === req.model_id && e.method === method
  );
  if (candidates.length === 0) {
    throw {
      code: 'unknown_model',
      message:
        `La démo couvre ${DEMO_MODEL_IDS.length} modèles précalculés ` +
        `(${DEMO_MODEL_IDS.join(', ')}). Installez l'application pour ` +
        'estimer les 25 modèles du catalogue.'
    };
  }
  const dist = (e: RawEstimate) =>
    Math.abs(Math.log((e.request.tokens_in + 1) / (req.tokens_in + 1))) +
    Math.abs(Math.log((e.request.tokens_out_estimated + 1) / (req.tokens_out_estimated + 1)));
  const best = candidates.reduce((a, b) => (dist(b) < dist(a) ? b : a));
  return toResult(best);
}

// ─── Préférences en mémoire (perdues au rechargement — c'est une démo) ───

const demoPrefs: AppPreferencesDto = {
  persona: null,
  enabled_modules: [
    'm1',
    'm3',
    'm7',
    'm8',
    'm9',
    'm12',
    'm13',
    'm14',
    'm15',
    'm17',
    'm20',
    'm22',
    'm25'
  ],
  onboarded: true,
  lang: 'fr',
  default_method: 'afnor_sobria',
  also_show_methods: ['ecologits'],
  default_datacenter_id: undefined
};

// ─── Dashboard d'exemple (agrégation arithmétique de fixtures réelles) ───
//
// Les P50 par requête sortent du moteur (fixtures) ; seuls les VOLUMES de
// requêtes sont un scénario d'usage fictif, déterministe (LCG seedé), et
// présenté comme tel (`period_label` suffixé « démo »).

function lcg(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    s = (s * 1664525 + 1013904223) >>> 0;
    return s / 2 ** 32;
  };
}

function p50For(modelId: string, indicator: string): number {
  const e = estimates.find(
    (x) =>
      x.request.model_id === modelId && x.method === 'afnor_sobria' && x.request.tokens_in === 1200
  );
  const ind = e?.indicators.find((i) => i.indicator === indicator);
  return ind?.interval.p50 ?? 0;
}

function periodBounds(period: DashboardPeriod): { start: Date; end: Date; label: string } {
  const now = new Date();
  const day = 24 * 3600 * 1000;
  const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate());
  switch (period) {
    case 'today':
      return { start: startOfDay(now), end: now, label: "Aujourd'hui (démo)" };
    case 'last_7_days':
      return {
        start: new Date(startOfDay(now).getTime() - 6 * day),
        end: now,
        label: '7 derniers jours (démo)'
      };
    case 'this_month':
      return {
        start: new Date(now.getFullYear(), now.getMonth(), 1),
        end: now,
        label: 'Ce mois-ci (démo)'
      };
    case 'last_month':
      return {
        start: new Date(now.getFullYear(), now.getMonth() - 1, 1),
        end: new Date(now.getFullYear(), now.getMonth(), 0),
        label: 'Mois précédent (démo)'
      };
    case 'this_year':
      return {
        start: new Date(now.getFullYear(), 0, 1),
        end: now,
        label: 'Cette année (démo)'
      };
  }
}

function demoDashboard(period: DashboardPeriod): DashboardSummaryDto {
  const { start, end, label } = periodBounds(period);
  const day = 24 * 3600 * 1000;
  const nDays = Math.max(1, Math.round((end.getTime() - start.getTime()) / day));
  const rand = lcg(42 + nDays);

  const daily = Array.from({ length: nDays }, (_, i) => {
    const date = new Date(start.getTime() + i * day);
    const perModel = DEMO_MODEL_IDS.map((m) => ({
      m,
      count: Math.floor(rand() * 14) + 2
    }));
    const co2 = perModel.reduce((s, x) => s + x.count * p50For(x.m, 'co2_eq'), 0);
    const energy = perModel.reduce((s, x) => s + x.count * p50For(x.m, 'energy'), 0);
    const water = perModel.reduce((s, x) => s + x.count * p50For(x.m, 'water'), 0);
    return {
      date: date.toISOString().slice(0, 10),
      request_count: perModel.reduce((s, x) => s + x.count, 0),
      co2eq_g_p50: co2,
      energy_wh_p50: energy,
      water_l_p50: water,
      perModel
    };
  });

  const totalReq = daily.reduce((s, d) => s + d.request_count, 0);
  const totalCo2 = daily.reduce((s, d) => s + d.co2eq_g_p50, 0);
  const totalEnergy = daily.reduce((s, d) => s + d.energy_wh_p50, 0);
  const totalWater = daily.reduce((s, d) => s + d.water_l_p50, 0);

  const byModel = new Map<string, { count: number; co2: number }>();
  for (const d of daily) {
    for (const x of d.perModel) {
      const cur = byModel.get(x.m) ?? { count: 0, co2: 0 };
      cur.count += x.count;
      cur.co2 += x.count * p50For(x.m, 'co2_eq');
      byModel.set(x.m, cur);
    }
  }

  return {
    period_label: label,
    period_start: start.toISOString(),
    period_end: end.toISOString(),
    total_requests: totalReq,
    total_co2eq_g_p50: totalCo2,
    total_energy_wh_p50: totalEnergy,
    total_water_l_p50: totalWater,
    vs_previous: {
      previous_total_co2eq_g_p50: totalCo2 * 1.18,
      delta_co2eq_pct: -15.3,
      previous_total_requests: Math.round(totalReq * 1.1),
      delta_requests_pct: -9.1
    },
    top_models: [...byModel.entries()]
      .map(([model_id, v]) => ({
        model_id,
        request_count: v.count,
        total_co2eq_g_p50: v.co2
      }))
      .sort((a, b) => b.total_co2eq_g_p50 - a.total_co2eq_g_p50),
    daily_series: daily.map((d) => ({
      date: d.date,
      request_count: d.request_count,
      co2eq_g_p50: d.co2eq_g_p50,
      energy_wh_p50: d.energy_wh_p50,
      water_l_p50: d.water_l_p50
    })),
    method_breakdown: [
      {
        method: 'afnor_sobria',
        request_count: totalReq,
        total_co2eq_g_p50: totalCo2,
        total_energy_wh_p50: totalEnergy,
        total_water_l_p50: totalWater
      }
    ],
    warning_multi_method: false
  };
}

// ─── Datacenters : agrégat pays (moyennes arithmétiques du JSON embarqué) ─

function aggregateByCountry(): CountryAggregateDto[] {
  const byCountry = new Map<string, DatacenterSummaryDto[]>();
  for (const dc of datacenters) {
    const list = byCountry.get(dc.country_iso) ?? [];
    list.push(dc);
    byCountry.set(dc.country_iso, list);
  }
  return [...byCountry.entries()].map(([country_iso, list]) => {
    const caps = list
      .map((d) => (d as { capacity_mw?: number }).capacity_mw)
      .filter((c): c is number => typeof c === 'number');
    return {
      country_iso,
      datacenter_count: list.length,
      avg_pue: list.reduce((s, d) => s + d.pue, 0) / list.length,
      if_electrical_g_per_kwh:
        list.reduce((s, d) => s + d.if_electrical_g_per_kwh, 0) / list.length,
      ...(caps.length > 0 ? { total_capacity_mw: caps.reduce((s, c) => s + c, 0) } : {}),
      centroid_lat: list.reduce((s, d) => s + d.lat, 0) / list.length,
      centroid_lon: list.reduce((s, d) => s + d.lon, 0) / list.length
    };
  });
}

// ─── Détail modèle (4 modèles démo : triplets registry + baseline moteur) ─

function modelDetail(id: string): ModelDetailDto {
  const raw = presets.find((p) => p.id === id);
  if (!raw || !DEMO_MODEL_IDS.includes(id)) {
    desktopOnly('La fiche distributionnelle complète de ce modèle');
  }
  const triplet = (i: RawInterval): TripletDto => ({ p5: i.p5, p50: i.p50, p95: i.p95 });
  const baseline = estimates.find(
    (e) => e.request.model_id === id && e.method === 'afnor_sobria' && e.request.tokens_in === 1200
  );
  const ind = (name: string) => baseline?.indicators.find((x) => x.indicator === name)?.interval;
  const co2 = ind('co2_eq');
  return {
    ...toPreset(raw),
    epsilon_prefill_mj_per_token: triplet(raw.epsilon_prefill_mj),
    epsilon_decode_mj_per_token: triplet(raw.epsilon_decode_mj),
    embodied_g_per_request: triplet(raw.embodied_g_per_req),
    baseline_co2eq_p5_g: co2?.p5 ?? 0,
    baseline_co2eq_p50_g: co2?.p50 ?? 0,
    baseline_co2eq_p95_g: co2?.p95 ?? 0,
    baseline_energy_wh_p50: ind('energy')?.p50 ?? 0,
    baseline_water_l_p50: ind('water')?.p50 ?? 0,
    vendor_disclosures: raw.vendor_disclosures
  };
}

// ─── Table de dispatch (clé = nom de commande Tauri) ─────────────────────

type Handler = (args?: Record<string, unknown>) => unknown;

export const demoHandlers: Record<string, Handler> = {
  meta_info: (): MetaInfo => ({
    app_version: 'démo web (fixtures seed 42)',
    estimator_seed: 42,
    estimator_n: 10_000,
    audit_path: '— (ledger local : application de bureau)',
    data_root: '— (référentiel Gold : application de bureau)'
  }),

  list_models: (): ModelPresetDto[] => presets.map(toPreset),

  get_model_detail: (args): ModelDetailDto => modelDetail(args?.id as string),

  list_vendor_comparison: (): VendorComparisonRowDto[] => {
    const byVendor = new Map<string, VendorComparisonRowDto>();
    for (const p of presets) {
      for (const d of p.vendor_disclosures) {
        const row = byVendor.get(d.vendor) ?? {
          vendor: d.vendor,
          has_prompt_level: false,
          has_training: false,
          primary_source_url: null
        };
        if (d.scope === 'inference_per_prompt') row.has_prompt_level = true;
        if (d.scope === 'training') row.has_training = true;
        row.primary_source_url ??= d.source_url;
        byVendor.set(d.vendor, row);
      }
    }
    return [...byVendor.values()].sort((a, b) => a.vendor.localeCompare(b.vendor));
  },

  list_methodologies: (): MethodologyInfoDto[] => methodologies,

  estimate_prompt: (args): EstimationResultDto => {
    const req = args?.req as EstimationRequestDto;
    return nearestEstimate(req, req.method ?? demoPrefs.default_method);
  },

  estimate_for_comparison: (args): EstimationResultDto =>
    nearestEstimate(args?.req as EstimationRequestDto, args?.method as EmpreinteMethod),

  list_datacenters: (): DatacenterSummaryDto[] => datacenters,

  aggregate_datacenters_by_country: (): CountryAggregateDto[] => aggregateByCountry(),

  get_datacenter_detail: () => desktopOnly('La fiche détaillée datacenter (profil 24 h)'),

  get_dashboard_summary: (args): DashboardSummaryDto =>
    demoDashboard(args?.period as DashboardPeriod),

  get_app_preferences: (): AppPreferencesDto => ({ ...demoPrefs }),

  set_app_preferences: (args): null => {
    Object.assign(demoPrefs, args?.prefs as AppPreferencesDto);
    return null;
  },

  get_referentiel_status: (): ReferentielStatusDto => ({
    available: false,
    message:
      'Mode démo web — le référentiel Gold (pipeline médaillon Copper→Silver→Gold) ' +
      "est embarqué dans l'application de bureau.",
    version: '',
    snapshot_date: '',
    sha256: '',
    source_count: 0,
    model_count: presets.length,
    path: ''
  })
};
