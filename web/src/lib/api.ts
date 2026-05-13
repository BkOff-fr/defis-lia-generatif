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
  | 'tauri_unavailable';

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
