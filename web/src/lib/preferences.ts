// Store de préférences utilisateur (C10 — ADR-0010).
//
// Source de vérité : `crates/sobria-core/src/preferences.rs` (Persona +
// ModuleId) et `crates/sobria-app/src/dto.rs` (AppPreferencesDto). Toute
// dérive entre ces deux fichiers TS et Rust doit casser un test e2e ou
// un test Rust serde (cf. `each_persona_bundle_modules_are_in_all_modules`
// dans sobria-core).
//
// Optimistic update + rollback : `savePreferences` met à jour le store
// localement avant l'IPC. Si l'IPC échoue, on restaure l'état précédent
// et on rethrow — l'UI affiche alors la bannière d'erreur.

import { writable, get } from 'svelte/store';
import {
  getAppPreferences,
  setAppPreferences,
  type AppPreferencesDto,
  type ModuleId,
  type Persona
} from './api';

export type { ModuleId, Persona } from './api';
export type AppPreferences = AppPreferencesDto;

// État du store : DTO + flag `loaded` pour distinguer "pas encore chargé"
// (rail unfiltered) de "chargé avec 0 module" (rail vide légitime).
export interface PreferencesState extends AppPreferencesDto {
  loaded: boolean;
}

const INITIAL: PreferencesState = {
  persona: null,
  enabled_modules: [],
  onboarded: false,
  lang: 'fr',
  default_method: 'afnor_sobria',
  also_show_methods: [],
  default_datacenter_id: undefined,
  loaded: false
};

export const preferences = writable<PreferencesState>(INITIAL);

/**
 * Charge les préférences depuis l'IPC et met à jour le store.
 * Doit être appelé une fois au boot (cf. `+layout.svelte`).
 * Throw si IPC indisponible — l'appelant gère.
 */
export async function loadPreferences(): Promise<void> {
  const p = await getAppPreferences();
  preferences.set({ ...p, loaded: true });
}

/**
 * Persiste les préférences via IPC. Optimistic update + rollback.
 */
export async function savePreferences(p: AppPreferencesDto): Promise<void> {
  const prev = get(preferences);
  preferences.set({ ...p, loaded: true });
  try {
    await setAppPreferences(p);
  } catch (e) {
    preferences.set(prev);
    throw e;
  }
}

/** Catalogue ordonné des 24 personas (mirror `sobria_core::Persona::all`). */
export const ALL_PERSONAS: readonly Persona[] = [
  'student',
  'pro_tech',
  'enterprise',
  'public_sector',
  'researcher'
] as const;

/** Catalogue ordonné des 24 modules (mirror `sobria_core::ModuleId::all`). */
export const ALL_MODULES: readonly ModuleId[] = [
  'm1',
  'm2',
  'm3',
  'm5',
  'm6',
  'm7',
  'm8',
  'm9',
  'm10',
  'm11',
  'm12',
  'm13',
  'm14',
  'm15',
  'm16',
  'm17',
  'm18',
  'm19',
  'm20',
  'm21',
  'm22',
  'm23',
  'm24',
  'm25'
] as const;

/**
 * Bundle pré-coché par persona. Mirror exact de
 * `sobria_core::Persona::default_modules` — tout drift fait tomber les
 * tests Rust + le test e2e Playwright qui valide la cohérence.
 *
 * **C32.1** : alignement strict sur le canon Rust + retrait de M14 (page
 * « À propos » accessible via menu, pas un module fonctionnel à pré-cocher).
 */
export function defaultModulesFor(persona: Persona | null): ModuleId[] {
  switch (persona) {
    case 'student':
      return ['m1', 'm8', 'm13', 'm15', 'm25'];
    case 'pro_tech':
      return ['m1', 'm3', 'm7', 'm8', 'm9', 'm13'];
    case 'enterprise':
      return ['m1', 'm7', 'm12', 'm15', 'm17', 'm20', 'm22', 'm25'];
    case 'public_sector':
      return ['m1', 'm8', 'm12', 'm17', 'm20', 'm22'];
    case 'researcher':
      return ['m1', 'm3', 'm7', 'm8', 'm9', 'm17'];
    case null:
      return [];
  }
}

// ─── Catalogue persona ───────────────────────────────────────────────────

export interface PersonaInfo {
  id: Persona;
  label: string;
  tagline: string;
}

// Catalogue des 5 personas — labels FR + taglines de l'ADR-0010 §"Personas v2".
// Les icônes Lucide sont définies côté Svelte (cf. `onboarding/+page.svelte`
// et `parametres/+page.svelte`) pour ne pas importer `@lucide/svelte` ici.
const PERSONA_CATALOG: Record<Persona, PersonaInfo> = {
  student: {
    id: 'student',
    label: 'Étudiant·e / Curieux·se',
    tagline: 'Comprendre votre impact, apprendre les bons réflexes'
  },
  pro_tech: {
    id: 'pro_tech',
    label: 'Professionnel·le tech',
    tagline: 'Optimiser vos prompts, comparer les modèles, exporter pour votre équipe'
  },
  enterprise: {
    id: 'enterprise',
    label: 'Entreprise',
    tagline: 'Piloter votre scope 3 IA, rapport CSRD, forecast budget carbone'
  },
  public_sector: {
    id: 'public_sector',
    label: 'Collectivité / Service public',
    tagline: 'Suivre votre empreinte territoriale, marchés publics frugaux'
  },
  researcher: {
    id: 'researcher',
    label: 'Chercheur·se / Journaliste',
    tagline: 'Reproductibilité, comparaisons inter-modèles, datasets publiables'
  }
};

export function personaLabel(p: Persona): string {
  return PERSONA_CATALOG[p].label;
}

export function personaTagline(p: Persona): string {
  return PERSONA_CATALOG[p].tagline;
}

// ─── Catalogue modules (CDC §4.1) ────────────────────────────────────────

export type ModuleCategory = 'estimation' | 'visualisation' | 'reporting' | 'pedagogie';

export interface ModuleInfo {
  id: ModuleId;
  label: string;
  description: string;
  category: ModuleCategory;
  /** Route SvelteKit du module ; `null` si non encore implémenté. */
  href: string | null;
}

const MODULES: Record<ModuleId, ModuleInfo> = {
  m1: {
    id: 'm1',
    label: 'Estimer un prompt',
    description: 'Mesurer CO₂, énergie, eau, métaux pour une requête unique.',
    category: 'estimation',
    href: '/'
  },
  m2: {
    id: 'm2',
    label: 'Workbench multi-prompts',
    description:
      'Estimer plusieurs prompts en série dans un même atelier. (Différé v1.1+, cf. ADR-0011.)',
    category: 'estimation',
    href: null
  },
  m3: {
    id: 'm3',
    label: 'Comparer modèles',
    description: 'Benchmarker 2 à 8 modèles côte-à-côte sur un même prompt.',
    category: 'estimation',
    href: '/comparer'
  },
  m5: {
    id: 'm5',
    label: 'Exporter rapport',
    description:
      'PDF, CSV ou JSON sourcé — pour intégrer un dossier. (Différé v1.1+, voir M22 Rapport CSRD pour le PDF officiel.)',
    category: 'reporting',
    href: null
  },
  m6: {
    id: 'm6',
    label: 'Géoloc datacenter',
    description: "Préciser l'origine géographique de l'inférence.",
    category: 'visualisation',
    href: null
  },
  m7: {
    id: 'm7',
    label: "Journal d'audit",
    description: 'Ledger chaîné SHA-256 de toutes vos estimations.',
    category: 'reporting',
    href: '/journal'
  },
  m8: {
    id: 'm8',
    label: 'Méthodologie',
    description: 'Comprendre le moteur Monte-Carlo et ses sources.',
    category: 'pedagogie',
    href: '/methodo'
  },
  m9: {
    id: 'm9',
    label: 'Bibliothèque de modèles',
    description: 'Catalogue des 25+ modèles couverts et leurs sources.',
    category: 'pedagogie',
    href: '/modeles'
  },
  m10: {
    id: 'm10',
    label: 'Importer batch',
    description:
      'Charger un CSV/JSON de prompts pour estimation groupée. (Différé v1.1+, backend M18 disponible mais UI dédiée non livrée.)',
    category: 'estimation',
    href: null
  },
  m11: {
    id: 'm11',
    label: 'Extension navigateur',
    description: 'Capter vos vrais usages ChatGPT/Claude depuis votre browser.',
    category: 'estimation',
    href: null
  },
  m12: {
    id: 'm12',
    label: 'Datacenters Europe',
    description: 'Carte interactive des sites avec drill-down énergie.',
    category: 'visualisation',
    href: '/datacenters'
  },
  m13: {
    id: 'm13',
    label: 'Simulateur « Et si...? »',
    description: '7 leviers temps réel pour explorer des scénarios.',
    category: 'estimation',
    href: '/simuler'
  },
  m14: {
    id: 'm14',
    label: 'À propos / Crédits',
    description: 'Sources, licences, contributeurs, méthodologie courte.',
    category: 'pedagogie',
    href: null
  },
  m15: {
    id: 'm15',
    label: 'Tableau de bord personnel',
    description: 'Vos usages agrégés (jour, semaine, mois).',
    category: 'visualisation',
    href: '/suivi'
  },
  m16: {
    id: 'm16',
    label: 'Forecast 12 mois',
    description: "Projection annuelle avec bande d'incertitude et leviers.",
    category: 'reporting',
    href: null
  },
  m17: {
    id: 'm17',
    label: 'Datasheet scientifique',
    description: 'Bilan complet pour reproductibilité scientifique.',
    category: 'reporting',
    href: '/datasheets'
  },
  m18: {
    id: 'm18',
    label: 'Batch CSV → rapport',
    description: 'Import + agrégation pour un rapport sourcé prêt à partager.',
    category: 'reporting',
    href: null
  },
  m19: {
    id: 'm19',
    label: 'Équipe multi-utilisateurs',
    description: 'RBAC léger pour partager une installation entre collègues.',
    category: 'reporting',
    href: null
  },
  m20: {
    id: 'm20',
    label: 'Territoire France',
    description: 'Cartographie RTE IRIS + Sankey énergétique territorial.',
    category: 'visualisation',
    href: '/territoire'
  },
  m21: {
    id: 'm21',
    label: 'Alertes & seuils',
    description: 'Notifications locales quand un seuil personnel est franchi.',
    category: 'reporting',
    href: null
  },
  m22: {
    id: 'm22',
    label: 'Rapport réglementaire (CSRD/AGEC)',
    description: 'PDF signé + bundle PROV-O conforme aux normes.',
    category: 'reporting',
    href: '/rapport-csrd'
  },
  m23: {
    id: 'm23',
    label: 'Marchés publics IA frugale',
    description: "Modèles de cahiers des charges pour appels d'offre.",
    category: 'reporting',
    href: null
  },
  m24: {
    id: 'm24',
    label: 'Apprendre',
    description: 'Mini-cours, bonnes pratiques, parcours guidés.',
    category: 'pedagogie',
    href: null
  },
  m25: {
    id: 'm25',
    label: 'Objectifs & habitudes',
    description: 'Eco-budget personnel hebdomadaire avec encouragements.',
    category: 'pedagogie',
    href: null
  }
};

export function moduleLabel(id: ModuleId): string {
  return MODULES[id].label;
}

export function moduleDescription(id: ModuleId): string {
  return MODULES[id].description;
}

export function moduleHref(id: ModuleId): string | null {
  return MODULES[id].href;
}

export function moduleCategory(id: ModuleId): ModuleCategory {
  return MODULES[id].category;
}

export function moduleInfo(id: ModuleId): ModuleInfo {
  return MODULES[id];
}

// ─── C32.2 — Pourquoi ce module est dans le bundle de ce persona ? ──────
// Tooltip pédagogique affiché au survol des modules pré-cochés dans
// l'onboarding (étape 4 Bundle) et dans /parametres. Le texte est
// persona-spécifique : un même module M1 sera justifié différemment pour
// un étudiant (« mesurer ton premier prompt en grammes ») et pour une
// entreprise (« référence auditable pour ton scope 3 IA »).

const MODULE_PERSONA_REASON: Record<Persona, Partial<Record<ModuleId, string>>> = {
  student: {
    m1: "Mesurer chaque prompt pour comprendre l'ordre de grandeur (CO₂, eau).",
    m8: 'Voir la méthodologie de mesure expliquée simplement.',
    m13: "Explorer 'et si je raccourcis mes prompts ?' ou 'et si je change de modèle ?'.",
    m15: 'Suivre ton usage semaine par semaine, équivalences humaines incluses.',
    m25: 'Fixer un objectif mensuel + alerte quand tu le dépasses.'
  },
  pro_tech: {
    m1: 'Mesurer chaque appel API individuellement (ton terrain de jeu).',
    m3: 'Comparer 3 modèles côte-à-côte pour choisir le plus frugal.',
    m7: 'Ledger SHA-256 chaîné pour le reporting interne / audit.',
    m8: 'Voir les formules par méthodologie (AFNOR + EcoLogits).',
    m9: 'Catalogue 25+ modèles avec P5/P50/P95 + vendor disclosure (Mistral × ADEME, Google, Meta).',
    m13: "Explorer les leviers d'optimisation avant de modifier ton intégration."
  },
  enterprise: {
    m1: "Référence d'estimation auditable pour ton scope 3 IA.",
    m7: 'Ledger chaîné = preuve non-altération pour audits CSRD.',
    m12: 'Comprendre où tournent vos LLM en Europe (PUE + mix élec).',
    m15: 'Vue agrégée de votre usage (admin / RSE).',
    m17: 'Datasheet Gebru pour reproductibilité scientifique.',
    m20: 'Empreinte territoriale FR par IRIS (RTE × ComparIA).',
    m22: 'Rapport CSRD/AGEC trimestriel signé + PROV-O.',
    m25: 'Définir un plafond carbone mensuel avec alerte webhook ou email.'
  },
  public_sector: {
    m1: 'Mesurer chaque prompt avec rigueur méthodologique sourcée Etalab 2.0.',
    m8: 'Méthodologie AFNOR SPEC 2314 + sources officielles FR (ADEME, RTE).',
    m12: 'Cartographier les datacenters européens (carte Leaflet + drill-down).',
    m17: 'Datasheet scientifique réutilisable comme template marchés publics.',
    m20: 'Empreinte par IRIS RTE : différenciateur FR unique de Sobr.ia.',
    m22: 'Rapport réglementaire AGEC PDF + JSON-LD PROV-O.'
  },
  researcher: {
    m1: 'Atelier reproductible (seed SOBRIA_SEED=42).',
    m3: 'Benchmark inter-modèles multi-méthodologie (AFNOR + EcoLogits).',
    m7: 'Ledger chaîné SHA-256 = preuve non-altération pour reviewers.',
    m8: 'Méthodologie complète + sources DOI (EcoLogits, AFNOR SPEC 2314).',
    m9: 'Catalogue P5/P50/P95 + vendor disclosure (transparence multi-méthodo).',
    m17: 'Datasheet Gebru 2018 auto-générée + JSON-LD PROV-O.'
  }
};

/**
 * Renvoie l'explication « pourquoi ce module est dans ton bundle persona »
 * ou `undefined` si pas de raison spécifique (module hors bundle par
 * défaut ou persona null).
 *
 * Utilisé comme attribut `title` natif sur les module-rows.
 */
export function moduleReason(persona: Persona | null, id: ModuleId): string | undefined {
  if (persona === null) return undefined;
  return MODULE_PERSONA_REASON[persona][id];
}

export const CATEGORY_LABELS: Record<ModuleCategory, string> = {
  estimation: 'Estimation',
  visualisation: 'Visualisation',
  reporting: 'Reporting',
  pedagogie: 'Pédagogie'
};

/** Helper de garde de route : true si le module est dans le bundle actif. */
export function isModuleEnabled(id: ModuleId): boolean {
  return get(preferences).enabled_modules.includes(id);
}
