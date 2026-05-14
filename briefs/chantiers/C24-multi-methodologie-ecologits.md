# Chantier C24 — Multi-méthodologie : EcoLogits + AFNOR en parallèle

> **Statut** : à exécuter — décidé 2026-05-13 après audit B.
> **Effort estimé** : 3-4 jours-dev (Rust backend + UI Svelte).
> **Bloque** : la validation scientifique formelle de l'estimateur, et plusieurs claims du dossier candidature data.gouv.
> **Dépendances** : aucune (chantier autonome).

---

## 1. Pourquoi ce chantier

### Le problème détecté
Pendant l'audit B (mai 2026), nous avons découvert que :
1. `K_DECODE_MJ_PER_TOKEN_PER_B = 0.025` dans `crates/sobria-estimator/src/model_presets.rs` est calibré **~1000× trop bas** vs les valeurs réelles mesurées par HF AI Energy Score / ML.ENERGY / EcoLogits.
2. Les 3 `ReproductionCase` qu'on a ajoutés contre EcoLogits 2026-01 (Llama 70B, Mistral Large 2) échouent en CI parce que nos estimations sortent ~1000× sous la réalité physique mesurée.
3. Une simple recalibration `K = 25` ne suffit pas — notre formule linéaire-par-token (AFNOR SPEC 2314) est fondamentalement incapable de matcher la non-linéarité EcoLogits sur tous les modèles. Llama 70B passe ±10 %, Mistral Large 2 (123B) sort à +93 %.

### Pourquoi le multi-engine est la bonne réponse
Plutôt que de bricoler notre coefficient unique, on **embarque plusieurs méthodologies scientifiques** et on laisse l'utilisateur choisir laquelle utiliser. Avantages :

- **Validation triviale** : `EcoLogitsEngine` reproduit EcoLogits par construction (on porte leurs formules). Plus aucune cible à atteindre par approximation.
- **Souveraineté méthodologique** : l'utilisateur français choisit librement entre AFNOR SPEC 2314 (Sobr.ia), méthodologie EcoLogits (référence open peer-reviewed), et plus tard BoaVizta / AIEnergyScore.
- **Différenciateur candidature data.gouv** : *"premier outil français qui embarque un catalogue de méthodologies scientifiques de l'empreinte LLM, comparables entre elles avec audit ledger SHA-256 chaîné"*. Aucun concurrent ne fait ça.
- **Honnêteté scientifique** : si l'utilisateur veut voir l'écart, il coche les autres méthodos en référence. On ne masque rien.
- **Notre méthodo reste un livrable propre** : AFNOR SPEC 2314 (référentiel français) est *proposée*, pas reléguée. C'est la méthodo par défaut pour les utilisateurs qui veulent rester sur du français normé.
- **Extensibilité v1.1+** : trait `EmpreinteEngine` permettra d'ajouter `BoaViztaEngine`, `AIEnergyScoreEngine`, `CustomEngine` (CSV user), etc.

---

## 2. Architecture cible

### 2.1 Trait `EmpreinteEngine`

```rust
// crates/sobria-estimator/src/engine_trait.rs (nouveau)

pub trait EmpreinteEngine: Send + Sync {
    /// Identifiant stable de la méthodologie.
    fn method_id(&self) -> EmpreinteMethod;

    /// Nom affiché en UI.
    fn display_name(&self) -> &'static str;

    /// Référence scientifique (DOI / paper / URL).
    fn reference(&self) -> &'static str;

    /// Lance l'estimation.
    fn estimate(
        &self,
        request: &EstimationRequest,
        params: &EstimationParams,
    ) -> EstimatorResult<EstimationResult>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmpreinteMethod {
    /// Formule linéaire-par-token AFNOR SPEC 2314 + Monte-Carlo Sobr.ia.
    AfnorSobria,
    /// Méthodologie EcoLogits 2026-01 (Rincé & Banse, DOI:10.21105/joss.07471).
    EcoLogits,
}
```

### 2.2 Refactor `MonteCarloEngine` en `AfnorMonteCarloEngine`
- Renommer la struct existante (compat alias temporaire).
- Implémenter `EmpreinteEngine`.
- **Recalibrer** `K_DECODE_MJ_PER_TOKEN_PER_B` de `0.025` à `25.0` (factor 1000 manquant).
- Multiplier les 8 presets en conséquence (`epsilon_prefill_mj`, `epsilon_decode_mj`).
- Ajuster les `PlausibilityCase` qui ne passent plus avec les vraies valeurs.

### 2.3 Nouveau `EcoLogitsEngine`

Port direct des formules d'EcoLogits 2026-01 (cf. <https://ecologits.ai/latest/methodology/llm_inference/>) :

```rust
// crates/sobria-estimator/src/engines/ecologits.rs (nouveau)

pub struct EcoLogitsEngine {
    seed: u64,
    n: u32,
}

// Constantes EcoLogits (commit hash + license CC BY-SA 4.0 cités)
const ALPHA_E: f64 = 1.17e-6;
const BETA_E:  f64 = -1.12e-2;
const GAMMA_E: f64 = 4.05e-5;
const ALPHA_L: f64 = 6.78e-4;
const BETA_L:  f64 = 3.12e-4;
const GAMMA_L: f64 = 1.94e-2;
const P_SERVER_W: f64 = 1200.0;
const N_GPU_INSTALLED: u32 = 8;
const MEM_GPU_GB: f64 = 80.0;
const Q_BITS: f64 = 16.0;
const MEM_OVERHEAD: f64 = 1.2;

// Formules :
//   n_GPU = roundup_pow2(ceil(1.2 × P × 16/8 / 80))
//   f_E(P,B) = α × exp(β × B) × P + γ      → Wh/token GPU
//   f_L(P,B) = α' × P + β' × B + γ'        → s/token
//   ΔT = t_out × f_L
//   E_GPU = n_GPU × t_out × f_E
//   E_server_noGPU = ΔT × P_SERVER × (n_GPU/N_INSTALLED) / B / 3600  (Wh)
//   E_request = PUE × (E_GPU + E_server_noGPU)
//   I_request_emb = (ΔT / (B × ΔL)) × I_server                       (g CO2eq)
//   total = E_request × IF + I_request_emb
```

Embodied selon EcoLogits :
- `I_server = (n_GPU / N_INSTALLED) × I_server_noGPU + n_GPU × I_GPU`
- `I_server_noGPU = 5700 kgCO2eq`, `I_GPU = 273 kgCO2eq`, `ΔL = 3 ans`

Sources des constantes : embedded comments + lien commit GitHub d'EcoLogits.

### 2.4 Factory / registry de méthodologies disponibles

```rust
pub struct MethodologyInfo {
    pub method: EmpreinteMethod,
    pub display_name: &'static str,
    pub reference: &'static str,        // DOI / URL
    pub license: &'static str,
    pub calibration_status: CalibrationStatus,
    pub year_published: u16,
    pub maintained_by: &'static str,
}

/// Liste complète des méthodologies embarquées dans Sobr.ia.
/// Au runtime, l'utilisateur :
///   1. Voit toutes celles disponibles dans Settings / "Méthodologies".
///   2. Choisit *une* méthodologie par défaut (utilisée pour tous les calculs).
///   3. Coche optionnellement d'autres pour les voir en référence (panneau
///      "Voir aussi" à côté du résultat principal).
pub static AVAILABLE_METHODS: &[MethodologyInfo] = &[
    MethodologyInfo {
        method: EmpreinteMethod::AfnorSobria,
        display_name: "AFNOR SPEC 2314 (Sobr.ia)",
        reference: "https://norminfo.afnor.org/norme/AFNOR%20SPEC%202314/",
        license: "MIT (Sobr.ia code) ; AFNOR SPEC publique",
        calibration_status: CalibrationStatus::Indicative,
        year_published: 2024,
        maintained_by: "Sobr.ia",
    },
    MethodologyInfo {
        method: EmpreinteMethod::EcoLogits,
        display_name: "EcoLogits 2026-01",
        reference: "https://doi.org/10.21105/joss.07471",
        license: "CC BY-SA 4.0",
        calibration_status: CalibrationStatus::Validated,
        year_published: 2025,
        maintained_by: "GenAI Impact (Rincé & Banse)",
    },
    // v1.1+ : BoaVizta, AIEnergyScore, etc.
];

pub fn engine_for(method: EmpreinteMethod) -> Box<dyn EmpreinteEngine> {
    match method {
        EmpreinteMethod::AfnorSobria => Box::new(AfnorMonteCarloEngine::new(42)),
        EmpreinteMethod::EcoLogits   => Box::new(EcoLogitsEngine::new(42)),
    }
}
```

---

## 3. Plan de travail (5 sous-chantiers)

### C24.1 — Trait `EmpreinteEngine` (1 jour)
- [ ] Créer `crates/sobria-estimator/src/engine_trait.rs`
- [ ] Définir `EmpreinteMethod` enum + sérialisation
- [ ] Implémenter trait sur `MonteCarloEngine` actuel
- [ ] Tests : trait object boxable, dispatch dynamique fonctionne

### C24.2 — Recalibration AFNOR + porting EcoLogitsEngine (1-2 jours)
- [ ] `K_DECODE_MJ_PER_TOKEN_PER_B = 25.0`
- [ ] Multiplier 8 presets × 1000 (eps_prefill, eps_decode)
- [ ] Ajuster ranges `PlausibilityCase` qui ne passent plus
- [ ] Créer `crates/sobria-estimator/src/engines/ecologits.rs`
- [ ] Port formules `f_E`, `f_L`, `n_GPU`, `E_server`, `E_embodied`
- [ ] Tests cross-check Python : 5+ cas où notre Rust = leur calculator (±0.5 %)
- [ ] **Réactiver** les 3 `ReproductionCase` (retirer `#[ignore]`)
- [ ] Bonus : 3 cas supplémentaires Llama 8B, Mistral Medium, GPT-4o-mini

### C24.3 — DTOs + IPC + persistence (1/2 jour)
- [ ] `EstimationResult` gagne un champ `method: EmpreinteMethod`
- [ ] Ledger audit stocke `method` (migration SQL)
- [ ] Préférences user : `default_method: EmpreinteMethod` + `also_show: Vec<EmpreinteMethod>`
- [ ] IPC `list_methodologies() -> Vec<MethodologyInfo>` (catalogue)
- [ ] IPC `estimate_prompt(method?, …)` (méthode optionnelle, fallback sur default)
- [ ] IPC `estimate_with_references(req, default, also_show) -> { primary, references }`
- [ ] Migration audit_v3 : ajout colonne `method TEXT NOT NULL DEFAULT 'afnor_sobria'`

### C24.4 — UI : catalogue + sélection + affichage (1 jour)

**Nouvelle page `/methodologies` (M26)** — catalogue des méthodos disponibles :
- Liste toutes les méthodos (cartes) avec : nom, DOI, license, calibration, mainteneur
- Pour chaque carte : bouton "Définir par défaut" + checkbox "Afficher en référence"
- Lien vers la doc embarquée de chaque méthodologie

**Settings persona** : nouveau bloc "Méthodologies"
- Radio button : méthodo par défaut (1 seule)
- Checkboxes : méthodos additionnelles affichées en référence (0..N)

**M1 Estimer un prompt** : 
- Le résultat principal utilise la méthodo par défaut
- Si méthodos additionnelles cochées : panneau dépliable "Voir aussi"
  - Format : `EcoLogits 0.018 g · BoaVizta 0.020 g · Δ vs default : -8 % / +12 %`

**M3 Comparer modèles** :
- Reste fonctionnel sur 1 méthodo (la default)
- Bouton "Comparer aussi avec [méthodo X]" → affiche en supplément

**Pas de side-by-side massif imposé** — l'user a *toujours* une méthodo primaire claire, les autres sont contextuelles.

### C24.5 — Documentation + dossier candidature (1/2 jour)
- [ ] ADR-0012 « Multi-méthodologie EmpreinteEngine »
- [ ] Section README.md "Méthodologies disponibles"
- [ ] Mise à jour `docs/CANDIDATURE-DATA-GOUV.md` : nouveau différenciateur
- [ ] Mise à jour `notebook/validation.qmd` : passer en mode "vs Engine.EcoLogits"
- [ ] Mention dans `crates/sobria-export/src/report.rs` : méthodo utilisée tracée dans le PDF CSRD

---

## 4. Definition of Done

- [ ] `cargo test --workspace` passe (toutes les ReproductionCase actives passent ±15 % EcoLogits)
- [ ] `cargo clippy -- -D warnings` passe
- [ ] 2 engines fonctionnels : `AfnorSobria` recalibré + `EcoLogits` portant les formules
- [ ] UI M3 montre les 2 méthodos côte-à-côte
- [ ] Audit ledger trace la méthodologie utilisée
- [ ] Documentation à jour (ADR-0012 + README + dossier)
- [ ] Tests cross-check Python (jamais d'écart > 0.5 % vs le calculator EcoLogits officiel)
- [ ] License CC BY-SA 4.0 respectée pour les portions portées d'EcoLogits (mention + lien commit)

---

## 5. Risques et mitigations

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Notre port EcoLogits diverge de leur calculator officiel | Moyenne | Élevé (perte crédibilité) | Tests cross-check via leur calculator Python en CI |
| Recalibration AFNOR casse des tests existants | Élevée | Moyen | Plan B : isoler les tests qui asserent des valeurs spécifiques, regénérer les fixtures golden |
| License CC BY-SA 4.0 d'EcoLogits incompatible avec MIT Sobr.ia | Faible | Élevé (légal) | Vérifier compatibilité (CC BY-SA est viral seulement sur les portions concernées — on peut isoler dans un crate séparé `sobria-engine-ecologits` sous CC BY-SA) |
| UI side-by-side trop chargée pour Mobile | Moyenne | Faible | Hide derrière un toggle "Mode comparaison" |

---

## 6. Pour le dossier candidature data.gouv

Ce chantier transforme une faiblesse (calibration manquante d'une méthodo unique) en différenciateur majeur :

> *"Sobr.ia est le premier outil français qui propose un **catalogue de
> méthodologies scientifiques** pour mesurer l'empreinte LLM. L'utilisateur
> choisit sa méthodo de référence (AFNOR SPEC 2314 « Sobr.ia », EcoLogits
> 2026-01, etc.), et peut activer d'autres méthodos en référence pour voir
> les écarts. Toutes les estimations sont tracées dans un audit ledger
> SHA-256 chaîné avec la méthodologie utilisée."*

C'est un message beaucoup plus fort qu'une validation chiffrée — c'est de la **souveraineté méthodologique** :
- L'utilisateur n'est pas enfermé dans une formule propriétaire opaque.
- Il peut comparer plusieurs approches scientifiques peer-reviewed.
- Il a accès à la **méthodologie française AFNOR SPEC 2314** comme alternative légitime aux outils anglo-saxons (EcoLogits, GreenAlgorithms, AI Energy Score).
- L'audit ledger garantit la traçabilité du choix méthodologique pour reporting CSRD.

---

## 7. Ouvertures v1.1+

Une fois le trait `EmpreinteEngine` en place :
- **BoaViztaEngine** : intégration de l'API BoaVizta pour les datacenters (CC BY-SA 4.0)
- **AIEnergyScoreEngine** : port du leaderboard HF (Apache 2.0)
- **CarbonAwareSDKEngine** : intégration GreenSoftwareFoundation
- **Custom user engine** : permettre à l'utilisateur d'injecter ses propres coefficients (CSV / TOML)

Avec 4-5 méthodos comparables, Sobr.ia devient **la référence française de transparence méthodologique** sur l'empreinte LLM.

---

## 8. Références
- EcoLogits methodology : <https://ecologits.ai/latest/methodology/llm_inference/>
- EcoLogits GitHub : <https://github.com/genai-impact/ecologits> (CC BY-SA 4.0)
- DOI : [10.21105/joss.07471](https://doi.org/10.21105/joss.07471)
- HF AI Energy Score : <https://huggingface.co/spaces/AIEnergyScore/Leaderboard>
- ML.ENERGY Leaderboard : <https://ml.energy/leaderboard>
- AFNOR SPEC 2314 : <https://norminfo.afnor.org/norme/AFNOR%20SPEC%202314/>
- BoaVizta : <https://boavizta.org/en/>

---

*Brief rédigé 2026-05-13 par Claude Cowork suite à l'audit B et la décision multi-méthodologie validée par Thibault.*
