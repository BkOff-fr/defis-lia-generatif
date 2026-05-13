# Prompts Claude Code — Écrans M12, M13, M20, M22

> **Mode d'emploi** : envoie un seul prompt à la fois à Claude Code, dans
> l'ordre M13 → M20 → M12 → M22 (du plus simple au plus structurant).
> Chaque prompt est autonome et démarre par `--- PROMPT ---`.
>
> **Pré-requis** : Cowork a livré tout le backend Rust (IPC commandes
> testées). Claude Code a déjà livré C09 (Estimer) + C10 (onboarding
> + design system tokens + rail).

---

## Prompt 1 — M13 Simulateur « Et si...? »

```
Tu es Claude Code, en charge du frontend pour le chantier C11 / module M13.

OBJECTIF
========
Implémenter l'écran "Simulateur Et si...?" qui consomme l'IPC
`simulate_scenarios` exposée par sobria-app. L'écran présente 7 leviers
réglables, recalcule en temps réel (debounced) le verdict CO2eq, affiche
un waterfall de contribution, un before/after, et une projection 12 mois.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C11-simulateur-et-si.md — spec complète, 7 leviers,
   §4bis "insight méthodologique" (CRUCIAL : afficher le levier dominant).
2. docs/CAHIER-DES-CHARGES-v1.0.md §4 M13.
3. crates/sobria-app/src/dto.rs — types SimulationRequestDto,
   SimulationResultDto, ScenarioOutcomeDto, ForecastResultDto.
4. CLAUDE.md §13 — zéro mock, données réelles uniquement.

CONTRAT IPC
===========
- `simulate_scenarios({req: SimulationRequestDto}) -> SimulationResultDto`
- Erreurs typées : `unknown_model`, `invalid_request`, `estimator_error`.
- Bornes : 20 scénarios max, forecast 1-60 mois, growth ±50%.

LES 7 LEVIERS À EXPOSER (cf. brief §1)
======================================
1. Modèle (Select depuis listModels(), 8+ options)
2. Région datacenter (radio FR/DE/IE/NL/GB/SE/...) → impacte IF mix
3. PUE (slider 1.05–1.6)
4. Mix élec (slider 10–800 g/kWh, presets FR=56, US=400, charbon=633)
5. Tokens de sortie (input number, 1–10000)
6. Embodied / req (slider 0.0001–1.0 g)
7. WUE (slider 0–5 L/kWh)

UX FLOW
=======
1. À l'arrivée : formulaire baseline (modèle, tokens) côté gauche, panneau
   "leviers" côté droit (7 contrôles).
2. À chaque changement de levier (debounced 300ms), reconstruction d'un
   SimulationRequestDto avec un Scenario nommé "Configuration actuelle" :
     - overrides remplis depuis les leviers modifiés
     - baseline = config sans aucun override
3. Affichage :
   - Verdict CO2eq P50 du scénario en gros (animé)
   - Pill comparative : "+12%" ou "-23%" vs baseline (couleur lime si <0,
     coral si >0)
   - Histogramme distributionnel des bins (déjà calculés côté Rust)
   - Carte "Levier dominant" : composant qui prend les 7 scénarios "isoler
     un seul levier" et affiche celui dont |delta| est max + part %
   - Waterfall horizontal : barres successives baseline → +lever1 → +lever2 → …
   - Forecast 12 mois : courbe de mois 0 à 11 (volume × P50 × (1+growth)^n).
     Inputs : volume/jour, growth%/mois.

CONTRAINTES
===========
- Design system existant (tokens app.css) : ink/lime/ivory, Instrument Serif
  pour les chiffres P50, Geist pour le corps, JetBrains Mono pour les valeurs.
- A11y : tous les sliders ont un <label> + aria-valuenow, contrastes WCAG AA.
- TS strict, pas de any, pas de mock.
- Composants découpés (Composer/LeverPanel/Verdict/Waterfall/Forecast).
- 1 test Playwright qui :
  - charge /m13 en SvelteKit pur (sans Tauri)
  - vérifie que les sliders sont rendus
  - attend l'erreur `tauri_unavailable` (contrat no-mock)

INSIGHT MÉTHODOLOGIQUE OBLIGATOIRE (cf. C11 §4bis)
==================================================
Pour gpt-4o-mini sur 100/500 tokens, l'embodied carbon (~99% du total)
écrase la composante électrique. Conséquence : les leviers PUE / mix élec
ont un impact minuscule sur ce profil. Le simulateur DOIT afficher en
clair : "Sur ce profil, votre principal levier est X (Y% du total). Les
autres ont un impact marginal." (cf. carte "Levier dominant").

DEFINITION OF DONE
==================
- [ ] Route `/m13` (ou autre selon convention rail).
- [ ] 7 leviers réglables avec debounce 300ms.
- [ ] Verdict P50 + delta% animé.
- [ ] Histogramme distributionnel rendu depuis bins.
- [ ] Carte "Levier dominant" calculée par 7 sous-simulations.
- [ ] Waterfall + Forecast 12 mois.
- [ ] Erreurs IPC typées affichées proprement.
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright (au minimum no-mock contract).
- [ ] Screenshot dans le commit message.

À NE PAS FAIRE
==============
- Pas d'attribution Shapley (séquentielle uniquement).
- Pas de plus de 20 scénarios par appel IPC.
- Pas de calcul du total absolu sans le baseline_volume_per_day fourni.
- Pas d'overlay LLM par DC (manque de données réelles).
```

---

## Prompt 2 — M20 Territoire FR + Sankey

```
Tu es Claude Code, en charge du frontend pour le chantier C13 / module M20.

OBJECTIF
========
Implémenter l'écran "Territoire FR" qui :
1. Cartographie 200 sites industriels FR (Leaflet + tuiles CARTO sombres)
   avec drill-down par site et agrégation par région.
2. Affiche un Sankey énergétique national alimenté par le mix RTE eco2mix.
3. Permet à l'utilisateur de "télécharger les données officielles" si elles
   ne sont pas encore présentes (bouton consentement explicite).

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C13-territoire-fr-sankey.md
2. crates/sobria-app/src/dto.rs — IndustrialSiteSummaryDto,
   RegionFrAggregateDto, SankeyDataDto, SankeyNodeDto, SankeyLinkDto.
3. docs/CAHIER-DES-CHARGES-v1.0.md §4 M20.
4. CLAUDE.md §13 — zéro mock, données réelles uniquement.

CONTRATS IPC
============
- `list_industrial_sites_fr({limit, offset}) -> IndustrialSiteSummaryDto[]`
- `get_industrial_site_fr({code_iris}) -> IndustrialSiteSummaryDto`
- `aggregate_industrial_sites_by_region() -> RegionFrAggregateDto[]`
- `sankey_fr_data() -> SankeyDataDto`

Erreurs typées :
- `data_not_ingested` : message explicite "Lance cargo run -p sobria-ingest
  -- fetch territoire-fr". L'UI doit afficher un bouton "Télécharger les
  données officielles" qui appelle... TODO v1.1 (pour l'instant texte
  d'instruction CLI). À implémenter en C18 quand on aura un IPC fetch.

LIVRABLES
=========
A) Route `/m20` avec layout 3 colonnes :
   - Gauche (1/4) : filtres (région, secteur si dispo, plage conso).
   - Centre (2/4) : carte Leaflet en plein écran adapté.
   - Droite (1/4) : drill-down du site/région cliqué.

B) Carte Leaflet :
   - Provider de tuiles : CARTO dark
     (`https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png`)
     déclaré dans tauri.conf.json CSP connect-src.
   - Zoom < 6 → markers agrégés par région (RegionFrAggregateDto, 13 max).
   - Zoom ≥ 6 → markers individuels sites (IndustrialSiteSummaryDto).
   - Click marker site → ouvre le panneau drill-down.
   - Click marker région → recentre + zoom à 7.

C) Drill-down site :
   - Code IRIS, commune, département, région
   - Consommation élec / gaz / total (MWh annuel)
   - Nb points de livraison
   - Année source
   - Sources cliquable vers ODRÉ

D) Drill-down région :
   - Nom, INSEE
   - Total conso (élec / gaz)
   - Nb sites
   - Top 5 sites (depuis aggregate_industrial_sites_by_region)
   - Part nucléaire mix régional (badge)

E) Section Sankey énergétique national :
   - SVG Sankey custom (pas de lib lourde), composant réutilisable
     (déjà identifié comme composant dataviz transverse dans CDC §4.3).
   - 2 layers : sources de production → consommation/export.
   - Liens proportionnels en épaisseur.
   - Couleurs cohérentes (vert : renouvelables, jaune : nucléaire,
     gris : fossiles, bleu : import/export).
   - Tooltip au hover : valeur TWh + part %.
   - Footer : "Source : RTE eco2mix [year] — fetched [date]".

CONTRAINTES
===========
- Design system existant (tokens app.css).
- Leaflet via CDN ou npm (`leaflet`, `@types/leaflet`).
- Pas de lib Sankey externe — SVG manuel (frugalité, voir CDC §8).
- A11y : carte navigable au clavier (Leaflet keyboard nav), Sankey avec
  fallback table ARIA.
- Si IPC retourne `data_not_ingested` : afficher un état empty propre
  avec instructions CLI + tooltip "future v1.1 : bouton in-app".

DEFINITION OF DONE
==================
- [ ] Route /m20 avec carte fonctionnelle.
- [ ] 200 markers individuels au zoom élevé.
- [ ] Agrégat 13 régions au zoom faible.
- [ ] Drill-down site + région.
- [ ] Sankey SVG avec liens proportionnels + tooltips.
- [ ] État `data_not_ingested` géré (message + instruction CLI).
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright (no-mock contract).
- [ ] Screenshot dans commit.

À NE PAS FAIRE
==============
- Pas d'attribution LLM dans le Sankey (pas de données fiables v1.0).
- Pas de pull RTE eco2mix horaire — l'agrégat annuel suffit.
- Pas de fetch live in-app (réservé v1.1, IPC manquant).
```

---

## Prompt 3 — M12 Datacenters Europe

```
Tu es Claude Code, en charge du frontend pour le chantier C12 / module M12.

OBJECTIF
========
Carte des 28 datacenters européens servant l'inférence LLM, avec
agrégation pays/site selon le zoom, drill-down complet (donut + barres
+ 24h) sur click marker.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C12-datacenters-europe.md
2. crates/sobria-app/src/dto.rs — DatacenterSummaryDto, DatacenterDetailDto,
   CountryAggregateDto.
3. docs/sources/CATALOGUE-DATACENTERS.md
4. CLAUDE.md §13.

CONTRATS IPC
============
- `list_datacenters() -> DatacenterSummaryDto[]` (28 DC)
- `get_datacenter_detail({id}) -> DatacenterDetailDto`
- `aggregate_datacenters_by_country() -> CountryAggregateDto[]` (13 pays)

Erreurs : `not_found` si id de DC inconnu.

LIVRABLES
=========
A) Route `/m12` layout 3 colonnes (gauche filtres / centre carte /
   droite drill-down).

B) Carte Leaflet :
   - Provider CARTO dark (idem M20, CSP à vérifier).
   - Zoom < 5 → markers agrégés par pays (CountryAggregateDto, 13 max),
     taille proportionnelle au nb de DC.
   - Zoom ≥ 5 → markers individuels DC, color-codé par opérateur (palette
     issue du design system).
   - Click DC → panneau drill-down détaillé.

C) Drill-down DC (composant principal) :
   - Header : nom, opérateur, pays (drapeau), ville
   - Stats baseline (issues de get_datacenter_detail) :
     - CO2eq P50 (g) sur prompt référence gpt-4o-mini 100/500
     - Énergie P50 (Wh)
     - Eau P50 (L)
   - **Donut** : composition mix élec local (issu de country IF
     correspondant) — 5 slices : nucléaire/renouv/gaz/charbon/autres.
     (Données : on n'a pas la décomposition réelle du country mix par DC
     en v1.0 — afficher juste le IF global du pays + un tooltip "détail
     par source en v1.1".)
   - **Barres** : 3 indicateurs (CO2eq / énergie / eau) avec valeurs
     P50 baseline, échelle log si besoin.
   - **Profil 24h** : courbe normalisée hourly_profile_24h (0-1) avec
     pic vers 18-20h. Tooltip horaire.
   - PUE, WUE (si dispo), capacité MW (si dispo).
   - Sources : liste cliquable vers rapports sustainability publics.

D) Mini-comparateur : sélectionner 2 DC → afficher en regard les 3 stats
   baseline. Bonus si possible mais pas bloquant.

CONTRAINTES
===========
- Design system existant.
- Donut + barres : SVG inline ou Chart.js (déjà importé en C09 pour les
  bins). Pas de nouvelle lib.
- Profil 24h : SVG path stroke fin, area fill discrète, axe X labels
  "00h", "06h", "12h", "18h".
- A11y : carte navigable clavier, donut avec fallback table %, profil
  24h avec aria-label horaire.

DEFINITION OF DONE
==================
- [ ] Route /m12.
- [ ] 28 markers DC au zoom élevé.
- [ ] Agrégation pays au zoom faible.
- [ ] Drill-down complet avec donut + barres + 24h.
- [ ] Sources cliquables.
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright (no-mock contract).
- [ ] Screenshot dans commit.

À NE PAS FAIRE
==============
- Pas de calcul réel d'estimation sur le DC sélectionné (déjà fait côté
  Rust dans get_datacenter_detail).
- Pas de comparaison auto avec les 27 autres DC (sauf mini-comparateur).
- Pas d'ajout de DC manuel (le dataset est figé v1.0).
```

---

## Prompt 4 — M22 Rapport CSRD / AGEC

```
Tu es Claude Code, en charge du frontend pour le chantier C14 / module M22.

OBJECTIF
========
Écran "Rapport CSRD/AGEC" qui permet à l'utilisateur de générer un
rapport PDF officiel conforme AFNOR SPEC 2314 pour une période donnée,
accompagné du JSON-LD PROV-O pour la reproductibilité audit.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C14-rapport-csrd-agec.md — structure rapport,
   méthodologie, PROV-O.
2. crates/sobria-app/src/dto.rs — CsrdReportRequestDto,
   CsrdReportResultDto.
3. docs/CAHIER-DES-CHARGES-v1.0.md §4 M22.
4. CLAUDE.md §13.

CONTRAT IPC
===========
- `export_csrd_report({req, output_dir}) -> CsrdReportResultDto`
- Erreurs : `invalid_request` (dates), `empty_period` (aucune entrée),
  `export_error` (génération PDF).

LIVRABLES
=========
A) Route `/m22` avec formulaire à 4 champs :
   - Nom de l'organisation (text, requis)
   - Date début (date picker, défaut : début du trimestre précédent)
   - Date fin (date picker, défaut : aujourd'hui)
   - Langue (select fr/en, défaut fr — en désactivé en v1.0)

B) Bouton "Générer le rapport" :
   - Appelle save dialog Tauri pour choisir le dossier de sortie
     (plugin-dialog déjà installé en C09).
   - Appelle `export_csrd_report({req, output_dir: path})`.
   - Loading state pendant la génération.

C) Après succès :
   - Card "Rapport généré" avec :
     - Bouton "Ouvrir le PDF" (utilise `tauri-plugin-shell` ou
       équivalent — si non dispo, juste le chemin copiable)
     - Bouton "Ouvrir le JSON-LD PROV-O" (idem)
     - SHA-256 affiché en monospace
     - Stats agrégées : total_requests, total_co2eq_g_p50,
       total_energy_wh_p50, total_water_l_p50
     - audit_entries_count
   - Lien cliquable vers la méthodologie locale.

D) Section "Aperçu" (optionnel mais recommandé) :
   - Avant génération, montre un preview des stats calculées sur la
     période sélectionnée (appel à un IPC `summary_for_period` si
     disponible — sinon génère un mini-aperçu côté front à partir de
     `list_audit_entries`).
   - Sert d'estimation pour que l'utilisateur valide la période avant
     d'engendrer le PDF.

CONTRAINTES
===========
- Date picker natif HTML5 (pas de lib externe).
- Format date envoyé à l'IPC : ISO 8601 RFC 3339 (`2026-01-01T00:00:00Z`).
- A11y : labels associés, aria-live pour le statut de génération,
  feedback visuel pendant le loading.
- Erreur `empty_period` → message explicite "Aucune estimation enregistrée
  dans cette période. Essaie une plage plus large."

DEFINITION OF DONE
==================
- [ ] Route /m22.
- [ ] Formulaire 4 champs avec validation.
- [ ] Save dialog Tauri fonctionnel.
- [ ] Génération PDF + PROV-O.
- [ ] Card de succès avec SHA-256 + stats.
- [ ] Erreurs typées affichées.
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright (no-mock contract).
- [ ] Screenshot dans commit.

À NE PAS FAIRE
==============
- Pas de templating multi-organisations (v1.1).
- Pas de signature GPG (v1.1).
- Pas de génération asynchrone avec progress bar — la génération est
  rapide (< 1s pour 1000 entrées).
- Pas de preview HTML du PDF (le PDF s'ouvre dans le viewer système).
```

---

## Ordre suggéré d'exécution

1. **M13** d'abord : il introduit Chart.js distribution + le concept de
   leviers / waterfall qui sera réutilisé.
2. **M20** ensuite : il introduit Leaflet + Sankey SVG (réutilisables M12).
3. **M12** : reutilise Leaflet de M20 + donut/barres déjà familiers.
4. **M22** : le plus simple (formulaire + save dialog), bouclage CSRD.

Chaque écran ≈ 4-8h de Claude Code. Total ≈ 2-3 jours frontend.

---

## Tests Playwright communs

Tous les écrans doivent avoir au minimum un test e2e validant le
**contrat no-mock** :

```ts
import { test, expect } from '@playwright/test';

test('m{XX} respects no-mock contract', async ({ page }) => {
  await page.goto('http://localhost:5173/m{XX}');
  // En mode SvelteKit dev sans Tauri, on doit voir l'erreur
  // tauri_unavailable ou similaire, pas du contenu fake.
  await expect(page.getByText(/tauri_unavailable|cargo tauri dev/i))
    .toBeVisible({ timeout: 5000 });
});
```

C'est le filet anti-mock.
