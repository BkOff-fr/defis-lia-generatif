# Chantier C31 — Intégration des datasets Tier 2 (v1.1)

> **Version cible** : v1.1.0 (post-candidature data.gouv.fr v1.0)
> **Sprint** : ~10 jours
> **Pré-requis** : v1.0 candidature shippée, **audit C30 livré** (`docs/sources/AUDIT-2026-Q3.md`)
> **Liens** : ADR-0009 (pipeline médaillon), ADR-0012 (multi-méthodologie), brief C30

---

## 0. Objectif

Le rapport `docs/sources/AUDIT-2026-Q3.md` a identifié **12 quick wins** prêts à être intégrés en C31 :

1. ⭐⭐⭐ Mistral × ADEME ACV Large 2 (0.5 j)
2. ⭐⭐ Google Gemini Environmental Disclosure (0.5 j)
3. ⭐ Meta Llama 3.x model cards (0.5 j)
4. ADEME Base Empreinte API (1.5 j)
5. ML.ENERGY Benchmark v3.0 (1 j)
6. IEA Energy and AI 2025 (0.5 j)
7. HF AI Energy Score (1 j)
8. EpochAI Models dataset (1 j)
9. ODRE complémentaire — 4 sous-datasets (1.5 j)
10. ARCEP édition 2025 parse PDF (0.5 j)
11. Shift Project projections 2030 (0.5 j)
12. Cloud Carbon Footprint extraction PUE (1 j)

**Total estimé : ~10 jours** pour le sprint v1.1.

---

## 1. Découpage en sous-chantiers

### C31.1 — Vendors disclosure (vendeur officiel) — 1.5 j

Intégrer dans Sobr.ia les chiffres ACV publiés directement par les vendors :

- **Mistral × ADEME Large 2** :
  - Mise à jour preset `mistral-large-2` avec valeurs Mistral/ADEME.
  - Encadré M9 fiche modèle "Données ACV vendor (vérifiées ADEME)".
- **Google Gemini** :
  - Preset Gemini enrichi : 0.24 Wh + 0.03 gCO₂eq + 0.26 mL eau (médian text).
  - Encadré M9 + avertissement méthodologie.
- **Meta Llama 3.x** :
  - Preset Llama 3.1 / 3.3 enrichi (training : 39.3M GPU h, 11 390 tCO₂eq location-based).
  - **Encadré pédagogique** "location-based vs market-based" pour expliquer la nuance.

**Nouvelle table** : `vendor_disclosures(model_id, vendor, scope, value, unit, source_url, published_at, methodology_note)` dans Gold.

**Nouvelle vue M9** : "Données vendor disclosure" avec table comparaison `Mistral / Google / Meta / Anthropic / OpenAI` montrant ce qui est publié vs ce qui ne l'est pas.

### C31.2 — Sources scientifiques officielles FR — 3.5 j

- **ADEME Base Empreinte API** (1.5 j) : nouveau crate `sobria-ingest::sources::ademe_base_empreinte` :
  - Trait `DataLayer` complet (ingest_copper, promote_silver, contribute_gold).
  - Endpoint API : [data.ademe.fr](https://data.ademe.fr/), GET /datasets/...
  - Schéma Silver `schemas/silver/ademe_facteurs_emission-v1.json`.
  - Migration moteur AFNOR/Sobr.ia : `K_DECODE_MJ_PER_TOKEN_PER_B` calibré contre facteurs ADEME officiels.
  - Citation source dans tous les commentaires `// Source: ADEME Base Empreinte V23.6, doi:...`.
- **ARCEP édition 2025 parse PDF** (0.5 j) :
  - Téléchargement PDF.
  - Extraction des tableaux (consommation datacenters FR, terminaux, etc.).
  - Insertion dans Gold `external_indicators(source, year, indicator, value, unit, note)`.
- **ODRE complémentaire** (1.5 j) — 4 sous-datasets :
  - Registre national installations production électricité (M12 datacenters → géolocalisation des centrales).
  - Eco2mix national consolidé (mix élec horaire FR — actuellement juste annuel dans M15 Dashboard).
  - Eco2mix régional consolidé (mix régional NUTS-2).
  - Consommation EPCI annuelle (maille intercommunalité).

### C31.3 — Calibration empirique + benchmarks — 2 j

- **ML.ENERGY Benchmark v3.0** (1 j) :
  - Téléchargement du leaderboard via GitHub.
  - Insertion dans Gold `external_benchmarks(model, task, hardware, energy_wh, source)`.
  - Ajout au catalogue `/methodologies` comme **4ème référence** (après AFNOR/Sobr.ia, EcoLogits, HF AI Energy Score).
  - Calibration des constantes Monte-Carlo en croisant avec mesures empiriques.
- **HF AI Energy Score** (1 j) :
  - Mapping rating 1-5 étoiles ↔ score Sobr.ia A-F.
  - Insertion dans `model_overview` Gold.
  - Encadré M9 "Score externe HuggingFace".

### C31.4 — Référentiel modèles enrichi — 2 j

- **EpochAI Models dataset** (1 j) :
  - Bulk-load des ~1500 modèles tracés EpochAI.
  - Enrichissement `model_overview` Gold : FLOPs training, paramètres, dates, training compute.
  - Citation CC-BY dans datasheet Gebru.
- **Cloud Carbon Footprint extraction PUE** (1 j) :
  - Extraction des constantes PUE et carbon intensity par région cloud (AWS/GCP/Azure × régions).
  - Insertion dans `datacenter_iris_link` Gold (déjà créé en v0.5.0 mais générique).
  - Application : M12 Datacenters Europe peut maintenant distinguer Azure West Europe vs AWS eu-west-1.

### C31.5 — Citations + datasheet enrichi — 1 j

- **IEA Energy and AI 2025** (0.5 j) :
  - Insertion des chiffres clés (945 TWh 2030, +17 % 2025, +50 % AI-focused) dans `external_projections`.
  - Citation dans M16 Forecaster + datasheet.
- **Shift Project projections 2030** (0.5 j) :
  - Idem (1 250-1 500 TWh 2030, ×2 à ×4 empreinte, part IA 15 % → 55 %).
  - Comparer dans M16 Forecaster comme fourchette IEA Base / Shift Project.

---

## 2. Definition of Done v1.1.0

- [ ] 12 sources Tier 2 intégrées (= 12 dans `LayerRegistry::standard()` + presets enrichis + citations).
- [ ] `cargo test --workspace` 100 % vert.
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] M9 fiche modèle affiche **table comparaison vendor disclosure** (5 vendors).
- [ ] M9 fiche modèle affiche **3 sources externes** par modèle quand dispo (vendor + HF AI Energy Score + ML.ENERGY).
- [ ] M16 Forecaster cite IEA + Shift Project (fourchette 945-1500 TWh 2030).
- [ ] Datasheet Gebru cite toutes les sources Tier 2 avec lineage Copper SHA-256.
- [ ] Catalogue `/methodologies` enrichi : AFNOR/Sobr.ia + EcoLogits + **HF AI Energy Score** + **ML.ENERGY**.
- [ ] CHANGELOG `[1.1.0]` complète.
- [ ] Bump versions : Cargo workspace + tauri.conf + web/package + extension + web-team → 1.1.0.
- [ ] Tag `v1.1.0`.

---

## 3. Anti-périmètre

- Sources Tier 3 strategic bets (ENTSO-E, Boavizta, MLPerf Power, LMSYS Arena) → v1.2+.
- Refactor moteur Monte-Carlo profond (rester sur calibration légère).
- Modifications du pipeline médaillon ADR-0009 (rester sur DataLayer existant).
- ElectricityMaps API (watchlist, attendre clarification licence cloud).
- Anthropic / OpenAI (pas de disclosure officielle à date, ne rien intégrer en valeur officielle).

---

## 4. Risques + mitigations

| Risque | Mitigation |
|---|---|
| ADEME Base Empreinte API change schéma V23→V24 | Versionner explicitement, fail fast à l'ingestion si schéma mismatch |
| Meta location/market-based mal compris par utilisateurs | Encadré pédagogique obligatoire + tooltip M9 |
| ML.ENERGY ne couvre que modèles open-source | Documenter limitation dans datasheet + utiliser pour calibration interne uniquement |
| Vendor disclosures évoluent (nouvelle publi Mistral/Google) | Workflow review trimestriel audit catalogue |

---

## 5. Découpage temporel suggéré

| Jour | Sous-chantier | Livrable |
|---|---|---|
| J1 | C31.1 vendors disclosure | 3 presets enrichis + table M9 |
| J2-J3 | C31.2 ADEME Base Empreinte | Source `DataLayer` + recalibration moteur |
| J4 | C31.2 ARCEP + ODRE | Parsing PDF + 4 sous-datasets ODRE |
| J5-J6 | C31.3 ML.ENERGY + HF AI Energy Score | 4ème reference + mapping rating |
| J7 | C31.4 EpochAI | Bulk-load 1500 modèles |
| J8 | C31.4 CCF PUE | Extraction constantes cloud |
| J9 | C31.5 IEA + Shift Project | Citations M16 + datasheet |
| J10 | Ship | CHANGELOG + bump + smoke test + tag v1.1.0 |

Total estimé : **~10 jours**.

---

## 6. Output attendu

À la livraison de C31, Sobr.ia v1.1.0 doit pouvoir afficher en M9 fiche modèle de Mistral Large 2 (exemple) :

```
═══════════════════════════════════════════════════════════
  Mistral Large 2 (mistral-large-2)
═══════════════════════════════════════════════════════════

╭─ Données ACV vendor (vérifiées ADEME) ─────────────────╮
│ Training : 20.4 ktCO₂eq + 281 000 m³ eau (18 mois)     │
│ Inference : 1.14 gCO₂eq par requête (400 tokens)       │
│ Source : Mistral AI × ADEME × Carbone 4, août 2025     │
╰─────────────────────────────────────────────────────────╯

╭─ Estimation Sobr.ia (méthodologie AFNOR) ──────────────╮
│ Inference 400 tokens (modèle dense ~123B):              │
│   P50 : 0.95 gCO₂eq (P5-P95: 0.74-1.21)                │
│   Eau : 4.2 mL (P5-P95: 3.4-5.3)                       │
│   Énergie : 2.6 Wh (P5-P95: 2.1-3.2)                   │
╰─────────────────────────────────────────────────────────╯

╭─ Cross-validation ─────────────────────────────────────╮
│ EcoLogits 2026-01 : 0.92 gCO₂eq                        │
│ HF AI Energy Score : ★★★★☆ (4/5)                      │
│ ML.ENERGY Benchmark : non testé (closed source)        │
╰─────────────────────────────────────────────────────────╯

╭─ Comparaison vendor disclosure ────────────────────────╮
│ Mistral AI (FR)      ✓ Prompt-level + Training         │
│ Google Gemini        ✓ Prompt-level (médian)           │
│ Meta Llama 3.3       ✓ Training (location + market)    │
│ Anthropic Claude     ⚠ Pas de disclosure officielle    │
│ OpenAI GPT-4o        ⚠ Pas de disclosure officielle    │
╰─────────────────────────────────────────────────────────╯
```

C'est ce niveau de richesse + transparence qui matérialise la mission Sobr.ia comme **tiers de confiance** sur l'empreinte IA générative.
