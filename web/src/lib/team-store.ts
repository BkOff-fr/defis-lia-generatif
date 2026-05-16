// Store Mode Équipe self-hosted (C29.1 — brief §C29.1).
//
// Source de vérité : `crates/sobria-app/src/team_settings.rs::TeamStatus`.
// Toute dérive entre ce store TS et la struct Rust doit casser le test
// d'intégration Playwright `parametres-mode-equipe.spec.ts` ou les unit
// tests Rust `sobria-app::team_settings::tests`.
//
// Optimistic update + rollback : `saveTeamField` met à jour le store
// localement avant l'IPC. Si l'IPC échoue, on restaure l'état précédent
// et on rethrow — l'UI affiche alors la bannière d'erreur.

import { writable, get } from 'svelte/store';
import {
  getTeamStatus,
  setTeamUrl,
  setTeamMode,
  setTeamAcceptInvalidCerts,
  type TeamMode,
  type TeamStatusDto
} from './api';

export type { TeamMode, TeamStatusDto } from './api';

/** État du store : DTO + flag `loaded` pour distinguer "pas encore chargé"
 *  (UI en skeleton) de "chargé avec mode local" (état initial légitime). */
export interface TeamState extends TeamStatusDto {
  loaded: boolean;
}

const INITIAL: TeamState = {
  enrolled: false,
  url: null,
  user_id: null,
  mode: 'local',
  fingerprint: null,
  enrolled_at: null,
  accept_invalid_certs: false,
  last_seen_at: null,
  estimations_sent: 0,
  loaded: false
};

export const teamStore = writable<TeamState>(INITIAL);

/** Recharge le snapshot complet depuis l'IPC `team_status`. */
export async function loadTeam(): Promise<void> {
  const s = await getTeamStatus();
  teamStore.set({ ...s, loaded: true });
}

/**
 * Champs modifiables localement via les IPC `team_set_*`. `mode` et
 * `accept_invalid_certs` n'exigent pas un serveur joignable ; `url`
 * est validé côté Rust (doit commencer par `https://`).
 */
export type WritableTeamField = 'url' | 'mode' | 'accept_invalid_certs';

/**
 * Met à jour un champ du store + persiste via IPC.
 * Optimistic : on applique localement, puis on appelle l'IPC ; en cas
 * d'erreur on rollback et on rethrow.
 */
export async function saveTeamField(field: 'url', value: string): Promise<void>;
export async function saveTeamField(field: 'mode', value: TeamMode): Promise<void>;
export async function saveTeamField(field: 'accept_invalid_certs', value: boolean): Promise<void>;
export async function saveTeamField(
  field: WritableTeamField,
  value: string | TeamMode | boolean
): Promise<void> {
  const prev = get(teamStore);
  // Optimistic patch local
  if (field === 'url' && typeof value === 'string') {
    teamStore.set({ ...prev, url: value });
  } else if (field === 'mode' && (value === 'local' || value === 'team' || value === 'both')) {
    teamStore.set({ ...prev, mode: value });
  } else if (field === 'accept_invalid_certs' && typeof value === 'boolean') {
    teamStore.set({ ...prev, accept_invalid_certs: value });
  }
  try {
    if (field === 'url') {
      await setTeamUrl(value as string);
    } else if (field === 'mode') {
      await setTeamMode(value as TeamMode);
    } else {
      await setTeamAcceptInvalidCerts(value as boolean);
    }
  } catch (e) {
    teamStore.set(prev);
    throw e;
  }
}
