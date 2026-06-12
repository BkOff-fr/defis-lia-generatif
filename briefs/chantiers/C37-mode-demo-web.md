# C37 — Mode démo web + relevé a11y/voix (exécuté) & plan « niveau pro »

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.
> **Origine** : audit critique du 2026-06-12 — « la démo web est une coquille
> vide, la moitié visible ne reflète pas la moitié invisible ».
> **Décision structurante** : le « contrat no-mock » hors Tauri devient un
> « contrat démo » → **mérite un ADR court** (à rédiger, voir §6).

## 1. Problème

Le CDC impose une démo web (plateforme 2ᵉ classe, bloquante v1.0), mais
CLAUDE.md §13 interdit les données factices : hors Tauri, chaque page
affichait « Application non lancée via Tauri » + une commande `cargo run`
exposée à l'utilisateur final, et **aucun contenu**. Le jury data.gouv.fr
et tout visiteur voyaient un site mort.

## 2. Résolution méthodologique

Les fixtures sont **générées par le moteur réel** (`sobria-estimator`,
seed 42, N = 10 000, horodatage figé) via `tools/fixturegen/` :

- `models.json` — sérialisation directe de `MODEL_REGISTRY` (34 presets) ;
- `methodologies.json` — `AVAILABLE_METHODS` (AFNOR SPEC 2314 + EcoLogits) ;
- `estimates.json` — 204 résultats Monte-Carlo complets (bins, P5/P50/P95,
  équivalents ADEME, hypothèses) : 34 modèles × 2 méthodes × 3 tailles
  (300/150, 1200/800, 4000/2000) ;
- `datacenters.json` — copie du dataset embarqué `sobria-geoloc`
  (28 DC, sources publiées).

Aucune valeur inventée côté TypeScript : `web/src/lib/demo/index.ts` sert,
adapte (snake_case core → DTO) et agrège arithmétiquement. Les volumes du
dashboard M15 sont un scénario d'usage déterministe **étiqueté « (démo) »**,
les P50 unitaires sortent du moteur. Chaque résultat embarque une hypothèse
`mode_demo`. Régénération :

```bash
cd tools/fixturegen
OUT_DIR_FIXTURES=../../web/src/lib/demo/fixtures cargo run --release
python3 - <<'EOF'   # minification (les fixtures sont .prettierignore)
import json
for n in ['estimates','models','methodologies','datacenters']:
    p = f'../../web/src/lib/demo/fixtures/{n}.json'
    json.dump(json.load(open(p)), open(p,'w'), ensure_ascii=False, separators=(',',':'))
EOF
```

## 3. Garanties

- **Jamais dans l'app de bureau** : activation uniquement si
  `!isTauriContext()` ; module + fixtures chargés en `import()` paresseux
  (le bundle Tauri ne les contient pas).
- **Transparence** : bannière `DemoBanner` ambre sur toutes les pages,
  rail « v0.9.0 · DÉMO », hypothèse `mode_demo` dans chaque résultat.
- Commandes couvertes : meta_info, list_models, get_model_detail,
  list_vendor_comparison, list_methodologies, estimate_prompt (point de
  grille le plus proche), estimate_for_comparison, list_datacenters,
  aggregate_datacenters_by_country, get_dashboard_summary,
  get/set_app_preferences (mémoire), get_referentiel_status (indisponible,
  expliqué). Le reste rejette `tauri_unavailable` avec un message
  utilisateur final (« Application de bureau requise », plus de `cargo run`).

## 4. Aussi dans ce chantier (Lot A — surface)

- **Contrastes WCAG** : `--ivory-3` #72706a (3,94:1, échec AA) → #83817a
  (5,0:1) ; `--ivory-4` #46443f (2,01:1) → #62605a (3,1:1, décoratif).
- **Typographie** : tokens px → rem (respect du réglage navigateur/OS),
  corps 14 → 15px, plancher 12px généralisé (308 occurrences 8-11px
  relevées dans 41 fichiers ; annotations SVG ≥ 10px).
- **Version** : « v0.3.0 · LOCAL » hardcodé → `__APP_VERSION__` injectée
  par Vite `define` depuis `web/package.json` (0.9.0) + suffixe DÉMO/LOCAL.
- **Voix** : vouvoiement unifié (m15, m17, m25, methodo, methodologies,
  rapport-csrd, ResultBlock, home).
- **Eyebrow home** : « Module M2 » → « Module M1 » (M2 retiré du périmètre,
  CDC : M2 ⊂ M3+M18).
- **leaflet** : utilisé par M12/M20 mais absent de package.json (installé
  localement sans --save) → ajouté (`leaflet ^1.9.4`, `@types/leaflet`),
  lock régénéré. Sans ça, `npm ci` en CI cassait.
- **Suite e2e** réécrite sur le contrat démo : 30 passed, 2 skipped
  documentés (drill-down DC Tauri-only ; persistance prefs démo).
  `vite.config.ts` : version via `define` (l'import runtime de
  package.json cassait l'hydratation en dev, 403 `server.fs.allow`).

## 5. Vérifications exécutées

`npm run lint` ✓ · `npm run check` 0 erreur ✓ · `npm run build` ✓ ·
Playwright 18 specs ✓ (Linux, Chromium headless) · captures avant/après ·
serveur statique + parcours estimation complet sans erreur console.
Non vérifié ici (à faire sous Windows/Tauri) : `cargo clippy`, lancement
desktop réel, suite e2e « cargo tauri dev » (C09.5).

## 6. Restes à faire (suivi, par priorité)

**P0 — avant candidature**
1. ADR court « contrat démo web » (remplace le contrat no-mock hors Tauri).
2. Exécuter **C36-uat-externe** (5 personas) — 0/5 à ce jour.
3. Publier le Gold + remote DVC (reproductibilité défi).
4. `/parametres` : scinder le `Promise.all` (couvert vs pairing/team) pour
   que Runtime/Référentiel/Méthodologies s'affichent en démo.
5. `/rapport-csrd` : bannière « app de bureau requise » au chargement
   (aujourd'hui le formulaire est actif et échoue tard, code `internal`).

**P1 — niveau pro**
6. Gating des actions mortes hors Tauri (journal : « Vérifier la chaîne »/
   « Exporter NDJSON » ; m17 « Nouveau projet » ; m25 formulaire ; équipe).
7. Migration rem complète des composants (px conservés hors tokens).
8. Thème clair (`prefers-color-scheme`) + audit axe outillé en CI.
9. Refactor `sobria-app/src/logic.rs` (4 088 lignes) par domaines +
   couverture tests app ≥ 8 %.
10. Signature binaires (SmartScreen/notarisation) avant diffusion large.
11. Slugs parlants (`/modeles`, `/tableau-de-bord`…) avec redirections.
12. Routes `/importer`,`/exporter` : retirées de l'app mais testées avant
    C37 — specs purgés, nettoyer les références restantes éventuelles.
