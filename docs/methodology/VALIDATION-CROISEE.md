# Validation croisée — méthodologie

> **Crate** : `sobria-estimator`, module `validation`.
> **Statut** : v1 — plausibilité uniquement. Cas de reproduction stricte
> à ajouter après lecture biblio S0.

---

## 1. Pourquoi deux niveaux

Le moteur Monte-Carlo est validable de deux manières indépendantes :

| Niveau | Question posée | Comment |
|--------|----------------|---------|
| **Plausibilité** | Le moteur produit-il des chiffres dans le bon ordre de grandeur ? | Vérifier que le P50 tombe dans une plage large (facteur 10²-10³) |
| **Reproduction** | Le moteur reproduit-il un chiffre publié à ±15 % ? | Comparer ponctuellement à une mesure publiée |

Les deux niveaux sont complémentaires :
- La **plausibilité** protège contre les bugs grossiers (unités, ordre
  d'opérations, oubli d'un facteur 1000) — toujours active en CI.
- La **reproduction** prouve la précision méthodologique — activée
  modèle par modèle au fil de la calibration biblio.

## 2. Niveau 1 — Plausibilité

### 2.1 Définition

Un cas de plausibilité fournit :
- Un modèle (preset registry)
- Des tokens d'entrée et de sortie
- Un mix électrique (gCO₂eq/kWh)
- Une **plage attendue** `[min, max]` pour le P50

Le test passe si le P50 calculé tombe dans cette plage.

### 2.2 Plages typiques

Les plages couvrent **2-3 ordres de grandeur** :

```
[1e-5, 1e-2] pour un petit modèle (8B) sur mix FR (56 g/kWh) et prompt court.
[1e-4, 1e-1] pour un modèle moyen (70B) sur mix FR.
[1e-3, 1.0]  pour un modèle moyen (70B) sur mix carboné (US-VA 412 g/kWh).
```

Ces plages sont volontairement larges. Elles ne valident pas la précision,
elles valident la **non-absurdité**.

### 2.3 Cas v1 (6 cas actifs)

Voir `crates/sobria-estimator/src/validation/cases.rs` :

| ID | Modèle | Mix | Tokens | Plage (g CO₂eq) |
|----|--------|-----|--------|-----------------|
| `gpt-4o-mini-fr-short` | gpt-4o-mini | FR | 100/500 | `[1e-5, 1e-2]` |
| `llama-70b-fr-medium` | llama-3-1-70b | FR | 200/1000 | `[1e-4, 1e-1]` |
| `llama-70b-us-va-medium` | llama-3-1-70b | US-VA | 200/1000 | `[1e-3, 1.0]` |
| `gpt-4o-fr-long` | gpt-4o | FR | 500/2000 | `[1e-3, 0.1]` |
| `mistral-large-fr-short` | mistral-large-2 | FR | 100/500 | `[1e-4, 1e-2]` |
| `us-mix-is-more-carbon-intensive-than-fr` | llama-3-1-8b | US-VA | 100/500 | sanity qualitatif |

### 2.4 Limites de la plausibilité

- Ne valide pas la **précision** d'une estimation individuelle.
- Une plage large peut masquer des bugs subtils (ex: facteur 2 d'erreur).
- N'engage rien sur la conformité AFNOR SPEC 2314.

C'est pourquoi le niveau 2 (reproduction) est nécessaire pour la
candidature finale.

## 3. Niveau 2 — Reproduction stricte

### 3.1 Définition

Un cas de reproduction fournit :
- Une **source publiée** (DOI, URL)
- Un modèle, des tokens, un mix, un PUE
- Une **valeur cible** publiée pour le P50
- Une **tolérance relative** (typiquement `0.15` pour ±15 %)

Le test passe si :

```
|P50_computed − P50_target| / P50_target ≤ tolerance
```

### 3.2 Cas à calibrer en S0 biblio

Trois cas prioritaires identifiés dans le brief S0 :

#### 3.2.1 `luccioni-2023-bloom-inference`

**Source** : Luccioni, Viguier, Ligozat (2023). *Estimating the Carbon
Footprint of BLOOM*. Journal of Machine Learning Research 24(253).
[https://jmlr.org/papers/v24/23-0069.html](https://jmlr.org/papers/v24/23-0069.html)

**Setup à extraire du paper** :
- Hardware (Jean Zay A100 80 GB)
- Mix électrique nucléaire France (~56 g/kWh à l'époque)
- Workload type
- Valeur cible CO₂eq par tâche / par heure

**Statut** : 🟡 À calibrer en S0.

#### 3.2.2 `patterson-2021-meena`

**Source** : Patterson et al. (2021). *Carbon Emissions and Large Neural
Network Training*. arXiv:2104.10350.

**Setup à extraire** : Meena training, GPU TPU, mix électrique zone Google.

**Statut** : 🟡 À calibrer en S0.

#### 3.2.3 `ecologits-2024-gpt-4-standard`

**Source** : EcoLogits (Data for Good 2024).
[https://github.com/genai-impact/ecologits](https://github.com/genai-impact/ecologits)

**Setup** : EcoLogits expose des chiffres par modèle (GPT-4o, Claude, etc.)
pour un prompt type. On peut piocher directement dans leur API Python.

**Statut** : 🟡 À calibrer en S0.

### 3.3 Processus d'ajout d'un cas

1. Lire la source, extraire les paramètres exacts (hardware, mix, prompt).
2. Renseigner `ReproductionCase` dans `cases.rs`.
3. Ajuster les presets du modèle dans `model_presets.rs` si nécessaire.
4. Lancer `cargo test -p sobria-estimator`. Si le test ne passe pas :
   - Soit la calibration du preset est à raffiner.
   - Soit notre formule diverge de celle du paper → écrire un commentaire
     explicatif et ouvrir une discussion.
5. Mettre à jour ce document avec la valeur cible exacte et la date.
6. Mettre à jour `model_presets.rs` : passer le `CalibrationStatus` du
   modèle de `Indicative` à `Validated`.

### 3.4 Garde-fou actuel

Un test (`no_reproduction_cases_yet_in_v1`) alerte tant que la liste
`REPRODUCTION_CASES` est vide. Quand Thibault aura calibré le premier
cas, il faudra augmenter le seuil dans ce test pour qu'il continue à
faire sens (ou le retirer).

## 4. Statut de calibration des modèles (v1)

| Modèle | C06 (preset) | Plausibilité | Reproduction |
|--------|:------------:|:------------:|:------------:|
| gpt-4o | Extrapolated | ✅ | 🟡 attendu |
| gpt-4o-mini | Extrapolated | ✅ | 🟡 attendu |
| claude-3-5-sonnet | Extrapolated | — | 🟡 attendu |
| mistral-large-2 | Indicative | ✅ | 🟡 attendu |
| mistral-medium-3 | Indicative | — | 🟡 attendu |
| llama-3-1-70b | Indicative | ✅ (2 cas) | 🟡 attendu |
| llama-3-1-8b | Indicative | ✅ (sanity) | 🟡 attendu |
| gemini-2-0-flash | Extrapolated | — | 🟡 attendu |

Légende : ✅ = test actif et vert / 🟡 = à venir / — = pas encore couvert.

## 5. Politique de promotion vers `Validated`

Un preset passe de `Indicative` / `Extrapolated` à `Validated` quand :

1. **Au moins un `ReproductionCase`** existe pour le modèle.
2. Le test passe à ±15 % en CI.
3. La référence est citée explicitement ici (§3.2.x).
4. Une PR documente le passage avec les chiffres bruts du paper.

Avant cela, un preset reste `Indicative` ou `Extrapolated` et l'app
affichera cette précision dans le `SourcePopover` pour transparence.

## 6. Reproductibilité

- Tous les tests utilisent `SOBRIA_SEED = 42`.
- Même seed + même `EstimationParams` → P50 identique à 10⁻¹⁰ près.
- Le test `reproducibility_same_seed_same_result` (engine) le garantit.

## 7. Évolutions prévues

- **Chantier C08** : calibration Luccioni + EcoLogits effective (post S0).
- **Chantier C09** : tests visuels de distributions (KS test contre Luccioni).
- **Chantier C10** : intégration validation dans CI nocturne avec rapport.
