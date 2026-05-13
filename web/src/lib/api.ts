// Wrapper IPC typé pour les commandes Tauri exposées par `crates/sobria-app`.
//
// Tout appel passe par `invoke()` réel (cf. CLAUDE.md §13 : pas de mock, pas
// de fallback, pas de données factices). Si l'app est ouverte hors contexte
// Tauri (ex: `npm run dev` dans un navigateur seul), les fonctions rejettent
// avec un `SobriaIpcError{ code: 'tauri_unavailable' }` que l'UI doit
// présenter explicitement à l'utilisateur.
//
// Les types ci-dessous **mirrorent à l'identique** les DTO de
// `crates/sobria-app/src/dto.rs` : noms de champs en snake_case (Serde n'a
// pas de `rename_all`), `Option<T>` Rust → `field?: T` TS (compatible avec
// `exactOptionalPropertyTypes`).
//
// Voir `briefs/chantiers/C09-tauri-integration.md` §3 pour les contrats IPC.

import { invoke } from '@tauri-apps/api/core';

// ─── DTO mirrors ─────────────────────────────────────────────────────────

export interface MetaInfo {
  app_version: string;
  estimator_seed: number;
  estimator_n: number;
  audit_path: string;
  data_root: string;
}

export type Openness = 'open' | 'open_weights' | 'closed';
export type Calibration = 'validated' | 'indicative' | 'extrapolated';
export type IndicatorName = 'co2eq' | 'energy' | 'water' | 'critical_metals' | 'cost';

export interface ModelPresetDto {
  id: string;
  display_name: string;
  provider: string;
  family: string;
  approx_params_billions: number;
  openness: Openness;
  calibration: Calibration;
  sources: string[];
}

// ─── Référentiel modèles (C18 — M9) ──────────────────────────────────────
//
// Triplet P5/P50/P95 — mirroir de `crates/sobria-app/src/dto.rs::TripletDto`.
export interface TripletDto {
  p5: number;
  p50: number;
  p95: number;
}

// Fiche détaillée d'un modèle (paramètres distributionnels + baseline
// contextuel calculé sans journalisation). Cf. brief
// `briefs/chantiers/C18-referentiel-modeles.md`.
export interface ModelDetailDto {
  id: string;
  display_name: string;
  provider: string;
  family: string;
  approx_params_billions: number;
  openness: Openness;
  calibration: Calibration;
  sources: string[];
  epsilon_prefill_mj_per_token: TripletDto;
  epsilon_decode_mj_per_token: TripletDto;
  embodied_g_per_request: TripletDto;
  baseline_co2eq_p5_g: number;
  baseline_co2eq_p50_g: number;
  baseline_co2eq_p95_g: number;
  baseline_energy_wh_p50: number;
  baseline_water_l_p50: number;
}

export interface EstimationRequestDto {
  model_id: string;
  tokens_in: number;
  tokens_out_estimated: number;
  datacenter_id?: string;
}

// Histogramme équi-width de la distribution Monte-Carlo d'un indicateur.
// Mirroir de `sobria_core::DistributionBins` (cf. crates/sobria-core/src/
// indicators.rs). `counts.length` bins entre `min` et `max` ; chaque count
// est un u32 (≤ N tirages, 10⁴ par défaut).
export interface DistributionBins {
  min: number;
  max: number;
  counts: number[];
}

export interface IndicatorDto {
  indicator: IndicatorName;
  p5: number;
  p50: number;
  p95: number;
  unit: string;
  /**
   * Histogramme Monte-Carlo (équi-width). Absent pour les entrées d'audit
   * antérieures à v0.2 ou si le moteur a tourné avec N trop faible. Quand
   * absent, le frontend retombe sur une approximation gaussienne depuis
   * P5/P50/P95 (visuel uniquement, non scientifique).
   */
  bins?: DistributionBins;
}

export interface EquivalentDto {
  label: string;
  value: number;
  source: string;
}

export interface HypothesisDto {
  key: string;
  value: unknown;
  source: string;
}

export interface EstimationRequestEchoDto {
  model_id: string;
  tokens_in: number;
  tokens_out_estimated: number;
  datacenter_id?: string;
  timestamp: string;
}

export interface EstimationResultDto {
  request: EstimationRequestEchoDto;
  indicators: IndicatorDto[];
  equivalents: EquivalentDto[];
  hypotheses: HypothesisDto[];
  computed_at: string;
  seed: number;
  audit_id: number;
}

export interface IntegrityReportDto {
  total_entries: number;
  valid: boolean;
  first_invalid_id?: number;
  message: string;
}

export interface AuditEntrySummaryDto {
  id: number;
  timestamp: string;
  model_id: string;
  co2eq_p50: number;
  sig_short: string;
  purged: boolean;
}

// ─── Simulation (C11 — M13 « Et si...? ») ────────────────────────────────
//
// Mirror 1-pour-1 de `crates/sobria-app/src/dto.rs` (bloc "simulation").

export interface ParamOverridesDto {
  model_id?: string;
  tokens_out?: number;
  pue?: number;
  if_electrical_g_per_kwh?: number;
  embodied_g_per_request?: number;
  wue_l_per_kwh?: number;
}

export interface ScenarioDto {
  label: string;
  overrides: ParamOverridesDto;
}

export interface ForecastConfigDto {
  months: number;
  monthly_growth_pct: number;
  base_volume_per_day: number;
}

export interface SimulationRequestDto {
  baseline: EstimationRequestDto;
  scenarios: ScenarioDto[];
  forecast?: ForecastConfigDto;
}

export interface ScenarioOutcomeDto {
  label: string;
  result: EstimationResultDto;
  delta_co2eq_g: number;
  delta_pct: number;
}

export interface ForecastResultDto {
  months: number;
  base_volume_per_day: number;
  monthly_growth_pct: number;
  baseline_monthly_co2eq_g: number[];
  baseline_annual_co2eq_g: number;
  scenarios_annual_co2eq_g: number[];
}

export interface SimulationResultDto {
  baseline: EstimationResultDto;
  scenarios: ScenarioOutcomeDto[];
  forecast?: ForecastResultDto;
}

// ─── Territoire FR (C13 — M20) ───────────────────────────────────────────
//
// Mirror 1-pour-1 de `crates/sobria-app/src/dto.rs` (bloc "territoire_fr").

export interface IndustrialSiteSummaryDto {
  code_iris: string;
  commune: string;
  department_code: string;
  region_iso: string;
  lat: number;
  lon: number;
  consumption_mwh_elec: number;
  consumption_mwh_gas: number;
  consumption_total_mwh: number;
  pdl_total: number;
  year: number;
}

export interface TopSiteDto {
  code_iris: string;
  commune: string;
  consumption_total_mwh: number;
}

export interface RegionFrAggregateDto {
  region_iso: string;
  region_name: string;
  insee_code: string;
  site_count: number;
  total_consumption_mwh_elec: number;
  total_consumption_mwh_gas: number;
  total_consumption_mwh: number;
  centroid_lat: number;
  centroid_lon: number;
  nuclear_share_pct: number;
  top_sites: TopSiteDto[];
}

// ─── Sankey FR (C13) ─────────────────────────────────────────────────────

export interface SankeyNodeDto {
  id: string;
  label: string;
  layer: number;
  value_twh: number;
}

export interface SankeyLinkDto {
  source: string;
  target: string;
  value_twh: number;
}

// ─── Datacenters Europe (C12 — M12) ──────────────────────────────────────

export interface DatacenterSummaryDto {
  id: string;
  name: string;
  operator: string;
  country_iso: string;
  city: string;
  lat: number;
  lon: number;
  pue: number;
  if_electrical_g_per_kwh: number;
}

export interface DatacenterDetailDto {
  id: string;
  name: string;
  operator: string;
  country_iso: string;
  city: string;
  lat: number;
  lon: number;
  pue: number;
  if_electrical_g_per_kwh: number;
  wue_l_per_kwh?: number;
  capacity_mw?: number;
  sources: string[];
  hourly_profile_24h: number[];
  baseline_co2eq_p50_g: number;
  baseline_energy_wh_p50: number;
  baseline_water_l_p50: number;
}

export interface CountryAggregateDto {
  country_iso: string;
  datacenter_count: number;
  avg_pue: number;
  if_electrical_g_per_kwh: number;
  total_capacity_mw?: number;
  centroid_lat: number;
  centroid_lon: number;
}

// ─── Dashboard personnel (C19 — M15) ─────────────────────────────────────
//
// Mirror 1-pour-1 de `crates/sobria-app/src/dto.rs` (bloc "dashboard +
// eco-budget"). Les agrégats sont calculés à la volée depuis le ledger
// d'audit (cf. `crates/sobria-app/src/dashboard.rs`).

/** Période supportée par `get_dashboard_summary`. */
export type DashboardPeriod = 'today' | 'last_7_days' | 'this_month' | 'last_month' | 'this_year';

export interface DashboardComparisonDto {
  previous_total_co2eq_g_p50: number;
  /** +12.0 ou -23.0 (en pourcent). */
  delta_co2eq_pct: number;
  previous_total_requests: number;
  delta_requests_pct: number;
}

export interface TopModelDto {
  model_id: string;
  request_count: number;
  total_co2eq_g_p50: number;
}

export interface DailySeriesPointDto {
  /** Format `YYYY-MM-DD`. */
  date: string;
  request_count: number;
  co2eq_g_p50: number;
  energy_wh_p50: number;
  water_l_p50: number;
}

export interface DashboardSummaryDto {
  period_label: string;
  /** RFC 3339. */
  period_start: string;
  /** RFC 3339. */
  period_end: string;
  total_requests: number;
  total_co2eq_g_p50: number;
  total_energy_wh_p50: number;
  total_water_l_p50: number;
  /** `undefined` si la période précédente est vide. */
  vs_previous?: DashboardComparisonDto;
  top_models: TopModelDto[];
  daily_series: DailySeriesPointDto[];
}

// ─── Eco-budget personnel (C19 — M25) ────────────────────────────────────
//
// Mirror 1-pour-1 de `crates/sobria-app/src/dto.rs` (bloc "dashboard +
// eco-budget"). Le tuple (indicator, period) est la PK côté SQLite —
// un seul objectif par combinaison. UPSERT en backend (cf. `goals_store`).

export type GoalIndicator = 'co2eq' | 'energy' | 'water';
export type GoalPeriod = 'daily' | 'weekly' | 'monthly';
export type GoalUnit = 'gCO2eq' | 'Wh' | 'L';
export type BudgetStatusLevel = 'ok' | 'warning' | 'exceeded';

export interface PersonalGoalDto {
  indicator: GoalIndicator;
  period: GoalPeriod;
  value_max: number;
  unit: GoalUnit;
}

export interface BudgetStatusDto {
  goal: PersonalGoalDto;
  current_value: number;
  period_start: string;
  period_end: string;
  /** 0..100+ (peut dépasser). */
  consumed_pct: number;
  /** "ok" (<70%), "warning" (70-100%), "exceeded" (>100%). */
  status: BudgetStatusLevel;
  /** value_max - current_value (peut être < 0). */
  remaining: number;
}

// ─── Rapport CSRD / AGEC (C14 — M22) ─────────────────────────────────────

export interface CsrdReportRequestDto {
  /** ISO 8601 (`2026-01-01T00:00:00Z`). */
  period_start: string;
  period_end: string;
  organization_name: string;
  /** Locale UI — v1.0 : `"fr"`. */
  locale: string;
}

export interface CsrdReportResultDto {
  pdf_path: string;
  provo_path: string;
  pdf_sha256: string;
  audit_entries_count: number;
  total_requests: number;
  total_co2eq_g_p50: number;
  total_energy_wh_p50: number;
  total_water_l_p50: number;
}

export interface SankeyDataDto {
  nodes: SankeyNodeDto[];
  links: SankeyLinkDto[];
  total_production_twh: number;
  year: number;
  source_url: string;
  source_sha256: string;
}

// ─── Erreurs typées ──────────────────────────────────────────────────────

// Codes alignés sur `crates/sobria-app/src/error.rs::AppError -> IpcError`.
// `tauri_unavailable` est ajouté côté front pour distinguer l'erreur
// "ouvert dans un navigateur sans runtime Tauri" des erreurs IPC réelles.
export type IpcErrorCode =
  | 'unknown_model'
  | 'invalid_request'
  | 'estimator_error'
  | 'audit_error'
  | 'core_error'
  | 'io_error'
  | 'json_error'
  | 'internal'
  | 'tauri_unavailable'
  | 'data_not_ingested'
  | 'not_found'
  | 'empty_period'
  | 'export_error';

export class SobriaIpcError extends Error {
  readonly code: IpcErrorCode;
  readonly details: unknown;

  constructor(code: IpcErrorCode, message: string, details?: unknown) {
    super(message);
    this.name = 'SobriaIpcError';
    this.code = code;
    this.details = details;
  }
}

// ─── Détection du contexte Tauri ─────────────────────────────────────────

const TAURI_GLOBAL = '__TAURI_INTERNALS__';

export function isTauriContext(): boolean {
  return typeof window !== 'undefined' && TAURI_GLOBAL in window;
}

function assertTauriContext(): void {
  if (!isTauriContext()) {
    throw new SobriaIpcError(
      'tauri_unavailable',
      "L'application doit être lancée via `cargo run -p sobria-app` (ou `cargo tauri dev`). Le contexte Tauri n'est pas disponible dans un navigateur seul."
    );
  }
}

// ─── Cœur de l'appel IPC ─────────────────────────────────────────────────

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  assertTauriContext();
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    throw normalizeError(err);
  }
}

// Tauri rejette les promesses avec la forme sérialisée d'`IpcError` :
// `{ code: string, message: string, details?: unknown }`. On en refait une
// instance de `SobriaIpcError` pour que `catch (e) { if (e instanceof
// SobriaIpcError) ... }` fonctionne côté UI.
function normalizeError(err: unknown): SobriaIpcError {
  if (err instanceof SobriaIpcError) return err;

  if (typeof err === 'object' && err !== null && 'code' in err && 'message' in err) {
    const e = err as { code: unknown; message: unknown; details?: unknown };
    if (typeof e.code === 'string' && typeof e.message === 'string') {
      return new SobriaIpcError(e.code as IpcErrorCode, e.message, e.details);
    }
  }

  if (err instanceof Error) {
    return new SobriaIpcError('internal', err.message);
  }

  return new SobriaIpcError('internal', typeof err === 'string' ? err : 'Erreur IPC inconnue');
}

// ─── Commandes ───────────────────────────────────────────────────────────
//
// Une fonction TypeScript = une commande Tauri (cf. `crates/sobria-app/src/
// main.rs` — bloc `*_cmd`). Les noms d'arguments doivent rester en
// snake_case car ils traversent Tauri sans renommage (les paramètres Rust
// `req`, `limit`, `offset`, `path` n'ont pas d'underscore donc aucun
// conflit avec la convention camelCase v2).

export function metaInfo(): Promise<MetaInfo> {
  return call<MetaInfo>('meta_info');
}

export function listModels(): Promise<ModelPresetDto[]> {
  return call<ModelPresetDto[]>('list_models');
}

export function getModelDetail(id: string): Promise<ModelDetailDto> {
  return call<ModelDetailDto>('get_model_detail', { id });
}

export function estimatePrompt(req: EstimationRequestDto): Promise<EstimationResultDto> {
  return call<EstimationResultDto>('estimate_prompt', { req });
}

export function verifyAudit(): Promise<IntegrityReportDto> {
  return call<IntegrityReportDto>('verify_audit');
}

export function listAuditEntries(limit: number, offset: number): Promise<AuditEntrySummaryDto[]> {
  return call<AuditEntrySummaryDto[]>('list_audit_entries', { limit, offset });
}

export function exportAuditNdjson(path: string): Promise<number> {
  return call<number>('export_audit_ndjson', { path });
}

export function simulateScenarios(req: SimulationRequestDto): Promise<SimulationResultDto> {
  return call<SimulationResultDto>('simulate_scenarios', { req });
}

// ─── Territoire FR + Sankey (C13 — M20) ──────────────────────────────────

export function listIndustrialSitesFr(
  limit: number,
  offset: number
): Promise<IndustrialSiteSummaryDto[]> {
  return call<IndustrialSiteSummaryDto[]>('list_industrial_sites_fr', { limit, offset });
}

export function getIndustrialSiteFr(codeIris: string): Promise<IndustrialSiteSummaryDto> {
  // Tauri reçoit l'argument tel quel — snake_case côté Rust = code_iris.
  return call<IndustrialSiteSummaryDto>('get_industrial_site_fr', { codeIris });
}

export function aggregateIndustrialSitesByRegion(): Promise<RegionFrAggregateDto[]> {
  return call<RegionFrAggregateDto[]>('aggregate_industrial_sites_by_region');
}

export function sankeyFrData(): Promise<SankeyDataDto> {
  return call<SankeyDataDto>('sankey_fr_data');
}

// ─── Datacenters Europe (C12 — M12) ──────────────────────────────────────

export function listDatacenters(): Promise<DatacenterSummaryDto[]> {
  return call<DatacenterSummaryDto[]>('list_datacenters');
}

export function getDatacenterDetail(id: string): Promise<DatacenterDetailDto> {
  return call<DatacenterDetailDto>('get_datacenter_detail', { id });
}

export function aggregateDatacentersByCountry(): Promise<CountryAggregateDto[]> {
  return call<CountryAggregateDto[]>('aggregate_datacenters_by_country');
}

// ─── Rapport CSRD/AGEC (C14 — M22) ───────────────────────────────────────

export function exportCsrdReport(
  req: CsrdReportRequestDto,
  outputDir: string
): Promise<CsrdReportResultDto> {
  // Argument Rust = `output_dir` (snake_case) — Tauri 2 convertit
  // automatiquement depuis camelCase JS.
  return call<CsrdReportResultDto>('export_csrd_report', { req, outputDir });
}

// ─── Dashboard personnel (C19 — M15) ─────────────────────────────────────
//
// Lecture seule sur le ledger d'audit ; renvoie un résumé pour la période
// demandée + (optionnel) la comparaison à la période précédente. Le backend
// rejette toute valeur `period` non listée dans `DashboardPeriod` via
// `invalid_request` (cf. `crates/sobria-app/src/logic.rs::get_dashboard_summary`).

export function getDashboardSummary(period: DashboardPeriod): Promise<DashboardSummaryDto> {
  return call<DashboardSummaryDto>('get_dashboard_summary', { period });
}

// ─── Eco-budget personnel (C19 — M25) ────────────────────────────────────
//
// `list_personal_goals` et `get_budget_status` sont sans argument.
// `set_personal_goal` reçoit l'objectif complet (UPSERT côté Rust).
// `delete_personal_goal` est idempotent — pas d'erreur si la clé n'existe pas.

export function listPersonalGoals(): Promise<PersonalGoalDto[]> {
  return call<PersonalGoalDto[]>('list_personal_goals');
}

export async function setPersonalGoal(goal: PersonalGoalDto): Promise<void> {
  await call<null>('set_personal_goal', { goal });
}

export async function deletePersonalGoal(
  indicator: GoalIndicator,
  period: GoalPeriod
): Promise<void> {
  await call<null>('delete_personal_goal', { indicator, period });
}

export function getBudgetStatus(): Promise<BudgetStatusDto[]> {
  return call<BudgetStatusDto[]>('get_budget_status');
}

// ─── Empreinte projet / datasheet Gebru (C20 — M17) ──────────────────────
//
// Mirror 1-pour-1 de `crates/sobria-app/src/dto.rs` (bloc "projects +
// datasheet"). Les dates sont en RFC 3339 UTC côté Rust ; côté UI on
// utilise `<input type="date">` (YYYY-MM-DD) et on normalise en
// `YYYY-MM-DDT00:00:00Z` / `T23:59:59Z` à l'envoi (idem M22). Les dates
// ne sont PAS modifiables après création (cf. brief §1.1 — préserve la
// reproductibilité du datasheet).

export interface ProjectDto {
  id: number;
  name: string;
  description: string;
  /** RFC 3339. */
  period_start: string;
  /** RFC 3339. */
  period_end: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface CreateProjectDto {
  name: string;
  description: string;
  period_start: string;
  period_end: string;
  tags: string[];
}

/** Update partiel — au moins un champ requis côté backend. Dates immutables. */
export interface UpdateProjectDto {
  name?: string;
  description?: string;
  tags?: string[];
}

export interface CompositionDto {
  total_requests: number;
  unique_models: string[];
  total_co2eq_g_p50: number;
  total_energy_wh_p50: number;
  total_water_l_p50: number;
  /** Absent si la période ne contient aucune entrée du ledger. */
  date_first_entry?: string;
  date_last_entry?: string;
}

export interface DatasheetDto {
  project: ProjectDto;
  /** Bloc JSON-LD complet (@context + @graph). À copier / sauvegarder tel quel. */
  jsonld: Record<string, unknown>;
  composition: CompositionDto;
  /** SHA-256 du JSON-LD pretty-printed (64 chars hex). */
  sha256: string;
}

export function listProjects(): Promise<ProjectDto[]> {
  return call<ProjectDto[]>('list_projects');
}

export function getProject(id: number): Promise<ProjectDto> {
  return call<ProjectDto>('get_project', { id });
}

export function createProject(req: CreateProjectDto): Promise<ProjectDto> {
  return call<ProjectDto>('create_project', { req });
}

export function updateProject(id: number, req: UpdateProjectDto): Promise<ProjectDto> {
  return call<ProjectDto>('update_project', { id, req });
}

export async function deleteProject(id: number): Promise<void> {
  await call<null>('delete_project', { id });
}

export function generateProjectDatasheet(id: number): Promise<DatasheetDto> {
  return call<DatasheetDto>('generate_project_datasheet', { id });
}

// ─── Préférences utilisateur (C10 — ADR-0010) ────────────────────────────

export type Persona = 'student' | 'pro_tech' | 'enterprise' | 'public_sector' | 'researcher';

// Liste fermée 24 IDs (M4 réservé en v1.3, cf. sobria_core::ModuleId).
export type ModuleId =
  | 'm1'
  | 'm2'
  | 'm3'
  | 'm5'
  | 'm6'
  | 'm7'
  | 'm8'
  | 'm9'
  | 'm10'
  | 'm11'
  | 'm12'
  | 'm13'
  | 'm14'
  | 'm15'
  | 'm16'
  | 'm17'
  | 'm18'
  | 'm19'
  | 'm20'
  | 'm21'
  | 'm22'
  | 'm23'
  | 'm24'
  | 'm25';

export interface AppPreferencesDto {
  persona: Persona | null;
  enabled_modules: ModuleId[];
  onboarded: boolean;
  lang: 'fr' | 'en';
}

export function getAppPreferences(): Promise<AppPreferencesDto> {
  return call<AppPreferencesDto>('get_app_preferences');
}

export async function setAppPreferences(prefs: AppPreferencesDto): Promise<void> {
  await call<null>('set_app_preferences', { prefs });
}
