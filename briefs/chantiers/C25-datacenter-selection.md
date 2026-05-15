# Chantier C25 — Sélection datacenter + carte immersive /datacenters

> **Statut** : à exécuter — décidé 2026-05-14.
> **Effort estimé** : 2-3 jours-dev (Rust backend léger + UI Svelte + restyle /datacenters).
> **Dépendances** : aucune (utilise les composants M12 existants et l'IPC `list_datacenters` déjà déployée).
> **Sprint** : à intégrer dans le sprint courant.

---

## 1. Pourquoi ce chantier

### Problème détecté
- L'écran « Estimer » (M1) affiche une carte `Datacenter (auto) · US-East · Virginie` en dur, non interactive et non câblée. L'utilisateur ne peut pas indiquer où son inférence tourne — pourtant cette information détermine PUE, mix électrique et WUE, qui pèsent fortement sur le résultat.
- L'écran `/datacenters` (M12) présente une carte d'Europe contrainte dans une grille 2 colonnes ; l'effet immersif d'une vraie cartographie est perdu, et les composants secondaires (filtres, drill-down) consomment de l'espace qui devrait être donné à la donnée géographique.
- Les flux dérivés (M3 *Comparer*, M13 *Simuler*) acceptent déjà un `datacenter_id` côté DTO mais ne l'exploitent pas pour surcharger les paramètres — autrement dit, le champ existe mais ne fait rien.

### Réponse
Un seul chantier couvrant deux livrables cohérents :
- **A — picker fonctionnel** : composant `DatacenterPicker` Svelte réutilisé dans M1 / M3 / M13, surcharge serveur de PUE / IF / WUE, persistance « last used » dans les préférences.
- **B — `/datacenters` immersif** : carte plein-cadre, filtres et drill-down flottants en verre dépoli au-dessus.

Avantages :
- **Honnêteté méthodo** : l'utilisateur voit l'impact réel du choix d'infra (un DC français à PUE 1.20 ne renvoie pas la même chose qu'un DC us-east-1 à 1.30).
- **Réutilisation** : le picker est un composant unique partagé par 3 routes ; pas de code dupliqué.
- **Pas de coût IPC** : l'API `list_datacenters` existe déjà (28 DCs Europe + grandes régions hyperscalers). `find_datacenter` existe dans `sobria-geoloc`.
- **Différenciation produit** : la carte immersive devient une vitrine, alignée avec le positionnement « cartographie souveraine + transparence des infras » du dossier candidature data.gouv.

---

## 2. Architecture cible

### 2.1 Frontend — nouveau composant `DatacenterPicker.svelte`

Emplacement : `web/src/lib/components/DatacenterPicker.svelte`.

```ts
// Props
interface Props {
  datacenters: DatacenterSummaryDto[];   // liste injectée par la route
  selected: DatacenterSummaryDto | null; // sélection courante (bind:)
}

// Émissions
$emit('select', dc | null);
```

- **État fermé** : carte au look de `.context-card` existante :
  - Si `selected` est défini : drapeau pays + `dc.name · dc.city`, `dc.operator · {if} g/kWh · PUE {pue}`.
  - Si `null` : `Aucun choisi` + sous-titre `L'estimation utilise vos PUE/IF par défaut`.
- **État ouvert** : popover ancré sur la carte, contient :
  - Champ de recherche texte (filtre case-insensitive sur `name`, `city`, `operator`, `country_iso`).
  - Liste groupée par pays (drapeau emoji via `country_iso` → `Intl.DisplayNames`).
  - Chaque item : `{flag} {name} · {city}` (gauche), `{operator} · {if} g/kWh · PUE {pue}` (droite).
  - Item spécial « Aucun choisi » en tête → permet de revenir à `null`.
- **A11y** : navigation clavier ↑/↓ Enter Esc, `role="listbox"` + `role="option"`, `aria-activedescendant`, focus trap natif via `dialog` ou `tabindex` rolling.
- **Click-outside** : ferme le popover.

### 2.2 Modifications des routes consommatrices

- `web/src/routes/+page.svelte` (M1) — bootstrap `listDatacenters()`, passe `datacenters` à `Composer`. Composer remplace son bloc carte hardcodé (lignes ~200-213) par `<DatacenterPicker bind:selected={selectedDatacenter} {datacenters} />`. Au mount, pré-remplit `selectedDatacenter` depuis `$preferences.default_datacenter_id` en cherchant dans la liste.
- `web/src/routes/comparer/+page.svelte` (M3) — un seul picker au-dessus de la liste de modèles ; le DC choisi s'applique à toutes les estimations comparées.
- `web/src/routes/simuler/+page.svelte` (M13) — picker pour le baseline du simulateur.
- Soumission IPC : tous incluent `datacenter_id: selectedDatacenter?.id ?? null` dans la requête.

### 2.3 Backend — helper `apply_datacenter_override`

Emplacement : `crates/sobria-app/src/logic.rs`.

```rust
/// Quand l'utilisateur a sélectionné un datacenter, on remplace les
/// distributions `pue`, `if_electrical_g_per_kwh` et (si dispo) `wue_l_per_kwh`
/// par des `Distribution::Point` dérivées du record.
///
/// Erreurs :
/// - `InvalidRequest` si l'id est inconnu (l'UI doit avoir une liste à jour ;
///   un id orphelin est forcément un bug).
fn apply_datacenter_override(
    params: &mut EstimationParams,
    datacenter_id: Option<&str>,
) -> IpcResult<()> {
    let Some(id) = datacenter_id else { return Ok(()); };
    let dc = sobria_geoloc::find_datacenter(id).ok_or_else(|| {
        IpcError::from(AppError::InvalidRequest(format!(
            "datacenter inconnu : {id}"
        )))
    })?;
    params.pue = Distribution::Point { value: dc.pue };
    params.if_electrical_g_per_kwh = Distribution::Point {
        value: dc.if_electrical_g_per_kwh,
    };
    if let Some(wue) = dc.wue_l_per_kwh {
        params.wue_l_per_kwh = Distribution::Point { value: wue };
    }
    Ok(())
}
```

Sites d'appel (juste après `EstimationParams::for_model(...)`) :
- `estimate_prompt` — flux M1 principal.
- `estimate_for_comparison` — panneau « Voir aussi » C24.
- `benchmark_models` — boucle par modèle, même DC pour tous.
- `simulate` — baseline params (les scénarios continuent d'utiliser leur propre `ParamOverrides`).
- Handler batch CSV — si la ligne CSV a une colonne `datacenter_id`, on l'applique row-wise.

### 2.4 Persistance « last used »

- **Rust core** : `crates/sobria-core/src/preferences.rs::AppPreferences` reçoit un nouveau champ
  ```rust
  #[serde(default)]
  pub default_datacenter_id: Option<String>,
  ```
  `#[serde(default)]` garantit la rétro-compat : les préférences pré-v0.5 désérialisent avec `None` sans migration explicite.
- **DTO IPC** : `crates/sobria-app/src/dto.rs::AppPreferencesDto` miroir du même champ.
- **Store** : `crates/sobria-app/src/preferences_store.rs` gagne une méthode `set_default_datacenter_id(id: Option<&str>)`.
- **Auto-save** : à la fin de `estimate_prompt`, on persiste best-effort la valeur courante de `req.datacenter_id` (qu'elle soit `Some(id)` ou `None`). Sémantique « la prochaine session reflète mon dernier choix, même si j'ai explicitement reverti à *Aucun choisi* » — symétrique et prévisible. Échec verrou ou écriture → `tracing::warn!`, mais n'échoue jamais l'estimation.
- **Frontend** : `web/src/lib/api.ts::AppPreferencesDto` reçoit `default_datacenter_id?: string`. `web/src/lib/preferences.ts::INITIAL` ajoute `default_datacenter_id: undefined`. Les 6 sites d'appel existants à `savePreferences({...})` ajoutent `default_datacenter_id: $preferences.default_datacenter_id` (même pattern que `default_method`).
- **Hydratation** : `loadPreferences()` charge déjà tout le DTO ; les routes lisent `$preferences.default_datacenter_id` au mount pour pré-remplir le picker via `datacenters.find(...)`.

### 2.5 `/datacenters` immersif

Modification structurelle de `web/src/routes/datacenters/+page.svelte` (et de son CSS scoped) :

- Conteneur racine du contenu de route : `position: relative; width: 100%; height: 100%; overflow: hidden;`.
- `DatacenterMap` : passe en `position: absolute; inset: 0;`, fill complet.
- `DatacenterFilters` : `position: absolute; top: 16px; left: 16px; max-width: 280px;` ; style verre :
  ```css
  background: color-mix(in oklab, var(--surface) 70%, transparent);
  backdrop-filter: blur(14px) saturate(1.2);
  -webkit-backdrop-filter: blur(14px) saturate(1.2);
  border: 1px solid color-mix(in oklab, var(--ink-mute) 12%, transparent);
  border-radius: 14px;
  box-shadow: 0 8px 24px color-mix(in oklab, black 12%, transparent);
  ```
- `DatacenterDrillDown` / `CountryDrillDown` : `position: absolute; top: 16px; right: 16px; bottom: 16px; width: 340px;` ; même style verre. **Rendu uniquement** quand un DC ou pays est sélectionné — pas de panneau vide affiché.
- Le shell app (header + rail latéral) reste inchangé : on n'agit que sur le contenu de la route.

Pas de changement IPC ni de composant nouveau pour la partie B.

---

## 3. Flux de données

```
list_datacenters (au mount route)
        │
        ▼
DatacenterSummaryDto[] ─────────► DatacenterPicker (props)
                                      │ select(dc | null)
                                      ▼
                              parent route :
                              selectedDatacenter = dc

submit estimate :
    estimatePrompt({ model_id, tokens_*, datacenter_id: dc?.id, method })
                                      │
                                      ▼
                       estimate_prompt (logic.rs)
                          ├── EstimationParams::for_model
                          ├── apply_datacenter_override(&mut params, dc_id)
                          ├── engine.estimate(req, &params)
                          ├── ledger.append
                          └── store.set_default_datacenter_id(dc_id)  (best-effort)
```

---

## 4. Tests

### 4.1 Rust

Tous dans `crates/sobria-app/src/logic.rs::mod tests` (et un dans `dto.rs` pour le round-trip prefs) :

1. `estimate_prompt_with_datacenter_overrides_pue_if_wue` — comparé à baseline sans DC, P50 des trois indicateurs (CO2eq, énergie, eau) diffère pour un DC dont les valeurs PUE/IF/WUE sont volontairement éloignées des valeurs par défaut.
2. `estimate_prompt_with_unknown_datacenter_returns_invalid_request` — vérifie le code d'erreur typé.
3. `estimate_prompt_persists_default_datacenter_id` — après un appel réussi avec un DC, la prochaine lecture des préférences renvoie l'id.
4. `benchmark_with_datacenter_applies_to_every_model` — chaque outcome du benchmark utilise les params surchargés.
5. `simulate_baseline_with_datacenter_applies_override` — baseline du simulateur respecte le DC ; les scénarios continuent de surcharger par-dessus comme avant.
6. `app_preferences_dto_round_trip_with_default_datacenter_id` — serde sérialise/désérialise correctement, et `#[serde(default)]` permet le chargement d'un JSON antérieur (sans le champ).

### 4.2 Frontend — Playwright

`web/tests/datacenter-picker.spec.ts` (nouveau fichier) :

- /estimate : ouvrir le picker → saisir « us-east » → choisir un item → vérifier la carte fermée affiche le nom + opérateur + PUE.
- Lancer une estimation, vérifier un P50 non NaN et non nul.
- Reload de la page : le picker pré-remplit avec le dernier DC choisi.
- /datacenters : vérifier que `.dc-map` couvre 100 % de la zone de route, que `.dc-filters` est en `position: absolute`, et que le drill-down n'est pas dans le DOM tant qu'aucune pin n'a été cliquée.

---

## 5. Hors-scope (renvoyés en backlog)

- **Datacenter utilisateur custom** : ajout d'un DC perso (PUE, IF) — ergonomie + persistance JSON. À traiter en C26+.
- **Intégration Electricity Maps LIVE** : le picker affiche `if_electrical_g_per_kwh` *statique* depuis le record. La vraie valeur LIVE viendra plus tard.
- **Animation enter/leave de la carte immersive** : opacité CSS suffit en v1, pas d'API JS d'animation.
- **UI batch M18** : pas de picker ajouté au flux CSV ; le backend honore la colonne `datacenter_id` si présente, l'utilisateur édite son CSV.
- **Picker dans /m17 (empreinte projet)** : pas pour cette itération — M17 agrège du ledger existant, pas de nouvelle estimation.

---

## 6. Definition of Done

- [ ] `DatacenterPicker.svelte` créé, a11y validée (audit axe), 0 erreur svelte-check / lint.
- [ ] Composer + /comparer + /simuler câblés au picker, pré-remplissage `default_datacenter_id` fonctionnel.
- [ ] `apply_datacenter_override` câblé aux 5 sites (`estimate_prompt`, `estimate_for_comparison`, `benchmark_models`, `simulate`, batch CSV) + tests Rust verts.
- [ ] `AppPreferences.default_datacenter_id` ajouté en `sobria-core` + DTO + TS, rétro-compat serde testé.
- [ ] `/datacenters` plein-cadre, filtres + drill-down en verre flottant ; pas de régression du clic pin / filtre / drill-down.
- [ ] Playwright `datacenter-picker.spec.ts` passe en local Tauri.
- [ ] `cargo clippy -p sobria-app -p sobria-core -p sobria-geoloc -- -D warnings` clean.
- [ ] `npm run check && npm run lint` clean.
- [ ] CHANGELOG.md mis à jour.
- [ ] Screenshot de /estimate (picker ouvert) + /datacenters (carte immersive) dans la PR.

---

*Décisions prises lors du brainstorming 2026-05-14 :*
- *Picker UX = dropdown sur la carte existante (option A1).*
- *Effet = override PUE + IF + WUE (option A2 + WUE étendue).*
- *Défaut initial = aucun choisi, picker optionnel.*
- *Persistance = implicite (last used), pas de section dédiée dans /parametres.*
- *Implémentation = composant Svelte custom avec recherche.*
- *Périmètre = M1 + M3 + M13 + batch backend (M18 reste CSV-only côté UI).*
- *Map immersif = filtres haut-gauche, drill-down panel droit, shell app conservé.*
