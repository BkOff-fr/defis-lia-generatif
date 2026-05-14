# ADR-0012 — Catalogue multi-méthodologie d'estimation d'empreinte LLM

- **Statut** : Accepted
- **Date** : 2026-05-13
- **Décideurs** : Thibault, Cowork
- **Contexte** : audit B (mai 2026), détection d'un bug de calibration
  fondamental du moteur AFNOR + opportunité différenciatrice pour la
  candidature data.gouv.fr.
- **Chantier** : C24 — *Multi-méthodologie EcoLogits + AFNOR*.
- **Remplace** : aucun ADR (étend ADR-0004 Monte-Carlo et ADR-0011
  Réduction périmètre v1.0).

---

## Contexte

### Le bug de calibration AFNOR (audit B)

Pendant l'audit honnête demandé par Thibault en mai 2026 (cf.
`briefs/chantiers/C24-multi-methodologie-ecologits.md` §1), trois
problèmes ont été détectés simultanément dans le moteur historique
*Sobr.ia AFNOR SPEC 2314* :

1. **`K_DECODE_MJ_PER_TOKEN_PER_B = 0.025`** est sous-calibré d'un
   facteur ~1000 par rapport aux mesures HF AI Energy Score, ML.ENERGY
   et EcoLogits. Conséquence directe : toutes les estimations Sobr.ia
   produites avant ce chantier sont sous-évaluées d'un facteur ~1000
   en énergie et CO₂eq.
2. **Aucun `ReproductionCase` opérationnel** : la promesse README/
   dossier de "validation croisée ±15 % contre Luccioni 2023 / EcoLogits
   2024" était fausse (la liste `REPRODUCTION_CASES` était littéralement
   vide). Le seul filet de sécurité était des `PlausibilityCase` avec
   des plages 3 à 5 ordres de grandeur, trop laxistes pour détecter le
   bug ci-dessus.
3. **Notre formule linéaire-par-token ne peut pas matcher EcoLogits
   sur tous les modèles** même après recalibration : la non-linéarité
   d'EcoLogits (terme γ constant + facteur `n_GPU` discret + server
   overhead non-GPU) crée des écarts de +50 % à +100 % sur les très
   gros modèles (Mistral Large 2, GPT-4o ~200B) si on cherche à coller
   leur estimation par un seul coefficient `K`.

### Le verrou scientifique

Recalibrer `K_DECODE` à `25` (corrigeant le bug ×1000) résout l'ordre
de grandeur mais maintient les écarts non-linéaires : ±50 % sur les
extrêmes 8B et 200B. Une candidature data.gouv crédible ne peut pas
défendre des claims "±15 %" qui ne tiennent pas sur les modèles dont
les utilisateurs se servent réellement (GPT-4o, Mistral Large).

### L'opportunité différenciatrice

Le défi data.gouv.fr récompense la **rigueur méthodologique** et la
**souveraineté française**. Or, l'écosystème actuel de l'estimation
d'empreinte LLM est dominé par des outils anglo-saxons mono-méthodologie :

| Outil | Méthodologie | License | Comparable ? |
|-------|--------------|---------|--------------|
| EcoLogits | Leur propre méthodo (peer-reviewed JOSS 2025) | CC BY-SA 4.0 | Non, modèle unique |
| HF AI Energy Score | Leur leaderboard (Apache 2.0) | Apache 2.0 | Non, modèle unique |
| BoaVizta | Leur méthodo (LCA hardware) | CC BY-SA 4.0 | Non, modèle unique |
| GreenAlgorithms | Leur méthodo (Lannelongue 2021) | MIT | Non, modèle unique |

Aucun de ces outils ne **propose simultanément plusieurs méthodologies
scientifiques au choix de l'utilisateur**, avec audit ledger SHA-256
chaîné permettant à un reporting CSRD d'identifier *quelle* méthodologie
a été utilisée pour *quelle* estimation.

---

## Décision

Introduire dans Sobr.ia un **catalogue de méthodologies d'empreinte LLM**
dont l'utilisateur choisit la sienne par défaut, et auquel il peut
ajouter d'autres méthodologies pour comparaison.

### Architecture

1. **Trait `EmpreinteEngine`** (dans `sobria-estimator/src/engine_trait.rs`)
   commun à toutes les méthodologies, avec une seule méthode core
   `estimate(request, params) -> EstimationResult`.
2. **`sobria_core::EmpreinteMethod`** : enum source de vérité partagée
   par sobria-core (où vit `EstimationResult`), sobria-estimator (où
   vit le trait) et sobria-app (DTO/IPC).
3. **`MethodologyInfo` + `AVAILABLE_METHODS`** : catalogue compile-time
   exposé via IPC `list_methodologies()`.
4. **2 engines embarqués en v1.0** :
   - `AfnorMonteCarloEngine` (= ancien `MonteCarloEngine` + impl du
     trait) — référentiel AFNOR SPEC 2314 français.
   - `EcoLogitsEngine` — port direct des formules EcoLogits 2026-01
     (`f_E`, `f_L`, `n_GPU`, server overhead, embodied) à partir de
     leur documentation publique (CC BY-SA 4.0).

### Comportement utilisateur (UX)

L'utilisateur sélectionne **une** méthodologie par défaut dans
`Settings → Méthodologies` (`/methodologies`). Cette méthodologie est
utilisée par tous les calculs de l'app — M1 estimation, M18 batch, M22
rapport CSRD, etc.

Il peut éventuellement activer d'autres méthodologies *en référence* :
un panneau **« Voir aussi »** apparaît à côté de chaque résultat
principal, avec les estimations parallèles calculées sur les méthodos
additionnelles + l'écart relatif vs résultat principal.

### Recalibration AFNOR

`K_DECODE_MJ_PER_TOKEN_PER_B` passe de `0.025` à `25.0` (factor 1000
manquant), les 8 presets de `MODEL_REGISTRY` sont multipliés en
conséquence. Les `PlausibilityCase` voient leurs plages ajustées pour
englober les nouvelles valeurs.

L'AFNOR engine **reste un livrable propre, proposé en défaut**. Il n'est
pas marqué "déprécié" ni "indicatif" — c'est le référentiel français
officiel, et l'utilisateur qui veut rester sur du normalisé FR a une
option crédible et auditable.

### Audit ledger : traçabilité méthodologique

La table `audit_entries` gagne une colonne `method TEXT NOT NULL DEFAULT
'afnor_sobria'`. La migration v1 → v2 est idempotente (PRAGMA + ALTER
TABLE IF NOT EXISTS). Toute estimation est tagguée avec sa méthodo,
filtrable a posteriori.

`EstimationResult.method: EmpreinteMethod` avec `#[serde(default)]` →
les vieilles entrées audit pré-C24 sont relues comme `AfnorSobria`
(seul moteur historique). Aucune migration JSON nécessaire.

### Validation chiffrée

3 `ReproductionCase` ciblent `EcoLogitsEngine` (et non plus le moteur
AFNOR — la recherche de coller à 1000× a été abandonnée, cf.
"Alternatives rejetées" §4). Tolérance **1 %** parce que c'est un port
direct des formules EcoLogits : seul l'arithmétique float64 introduit
du bruit.

| ID cas | Modèle | tokens | Mix | Écart Python vs Rust |
|---|---|---|---|---|
| `ecologits-llama-70b-fr-short` | Llama 3.1 70B | 100/500 | FR 56 g/kWh | -0.08 % |
| `ecologits-llama-70b-us-long` | Llama 3.1 70B | 100/2000 | US-VA 412 g/kWh | -0.01 % |
| `ecologits-mistral-large-us-medium` | Mistral Large 2 | 100/1000 | US-VA 412 g/kWh | -0.23 % |

Notebook `notebook/validation.qmd` reproductible à la main en Python.

### Conditions de réussite

- [x] `cargo test -p sobria-estimator validation` passe (3 cas ±1 %).
- [x] `cargo test -p sobria-audit` passe — migration audit_v3 inclus.
- [x] `cargo test -p sobria-app` passe — DTO/IPC + préférences étendues.
- [x] `cargo clippy --workspace -- -D warnings` passe.
- [x] UI page `/methodologies` fonctionnelle, basculement méthodo
  visible à l'écran.
- [x] Panneau "Voir aussi" sur l'écran M1 (Atelier) si méthodos
  additionnelles activées.

---

## Conséquences

### Positives

- **Différenciateur candidature data.gouv** : *premier outil français à
  embarquer un catalogue de méthodologies scientifiques pour
  l'empreinte LLM, avec souveraineté de choix utilisateur et audit
  ledger SHA-256 chaîné*. Aucun concurrent ne propose ça.
- **Validation triviale** : l'`EcoLogitsEngine` reproduit EcoLogits par
  construction (port direct). Plus aucune cible à approcher par
  approximation. Tests à 1 % de tolérance, défendables devant un jury
  scientifique.
- **Honnêteté radicale** : on ne masque plus le bug de calibration
  historique AFNOR — il est corrigé (`K_DECODE = 25`) et documenté ici.
  Le user voit les écarts entre méthodos dans le panneau "Voir aussi",
  on n'impose pas un point de vue scientifique unique.
- **Souveraineté française** : la méthodologie AFNOR SPEC 2314 reste
  proposée (par défaut), pas reléguée. C'est notre référentiel
  national, on n'a pas à s'aligner servilement sur EcoLogits.
- **Extensibilité v1.1+** : ajouter `BoaViztaEngine`, `AIEnergyScoreEngine`,
  `CustomEngine` (user CSV) ne demande qu'une implémentation du trait
  + une entrée dans `AVAILABLE_METHODS`.

### Négatives

- **Pollution potentielle du ledger** : si l'utilisateur active "voir
  aussi" sur plusieurs méthodos, chaque estimation principale crée
  N entrées d'audit additionnelles. Mitigation : filtrage par méthode
  dans la future UI Journal, et l'utilisateur reste libre de purger.
- **Charge de maintenance** : porter une méthodologie tierce, c'est
  s'engager à la mettre à jour quand l'amont publie une nouvelle
  version. Mitigation : version de méthodo embarquée dans le nom
  (`EcoLogits 2026-01`), commit hash GitHub de référence cité.
- **License hybride** : la méthodologie EcoLogits portée est sous
  CC BY-SA 4.0 (viral sur les portions concernées). Sobr.ia reste MIT
  sur son propre code ; les portions portées sont isolées dans
  `crates/sobria-estimator/src/engines/ecologits.rs` avec attribution
  explicite. Pas de pollution du reste du code.
- **API IPC plus large** : 1 commande de plus (`list_methodologies`)
  + champ `method` optionnel dans `EstimationRequestDto` et obligatoire
  dans `EstimationResult`. Compat ascendante grâce à `#[serde(default)]`.

### Migration utilisateur

- **Aucun changement requis** pour les utilisateurs existants. Au
  premier lancement post-C24, `default_method = AfnorSobria` (donc
  comportement identique à avant), `also_show_methods = []` (donc pas
  de panneau "Voir aussi"). L'utilisateur découvre la nouveauté en
  cliquant sur "Méthodologies" dans le rail.
- Les anciennes entrées du ledger sont préservées et étiquetées
  rétroactivement `afnor_sobria`. La vérification de la chaîne SHA-256
  reste valide (la migration SQL ajoute la colonne mais ne modifie pas
  les payloads JSON sur lesquels reposent les signatures).

---

## Alternatives rejetées

### Alternative 1 — Recalibrer `K_DECODE` agressivement et garder un seul moteur

Tentation initiale : changer `K_DECODE = 25` et accepter des écarts de
±50 % vs EcoLogits sur les extrêmes 8B / 200B, défendre l'approche
AFNOR SPEC 2314 comme "approximation pédagogique linéaire-par-token".

**Pourquoi rejeté** :
- Les `ReproductionCase` Mistral Large 2 + GPT-4o continuent d'échouer
  même avec recalibration optimale. Aucune valeur unique de `K` ne
  satisfait simultanément les 5 modèles open-weights de référence.
- Annoncer "validation ±15 %" reste impossible sans triche. On
  retomberait dans le pattern bullshit qu'on vient justement de
  démanteler dans l'audit B.

### Alternative 2 — Supprimer AFNOR, ne garder qu'EcoLogits

**Pourquoi rejeté** :
- Renoncer à la méthodologie AFNOR SPEC 2314, c'est renoncer à un
  argument central de la candidature data.gouv (référentiel français
  officiel). Sobr.ia deviendrait un *port français* d'EcoLogits, pas
  un *projet français* à part entière.
- AFNOR SPEC 2314 vient avec des avantages propres : Monte-Carlo N=10⁴
  natif (vs EcoLogits déterministe en v1), distribution log-normale
  des paramètres, intervalles P5/P50/P95 — utiles en reporting CSRD.

### Alternative 3 — Comparer côte-à-côte par défaut (toutes les méthodos toujours visibles)

**Pourquoi rejeté** (et corrigé sur retour Thibault le 2026-05-13) :
- UX bruyante : forcer l'affichage de 2+ résultats pour chaque
  estimation noie l'information utile. L'utilisateur veut une
  *décision*, pas un comité.
- Sémantique du ledger : si on calcule toujours N méthodos, on
  multiplie les entrées audit par N, sans valeur ajoutée pour les
  utilisateurs qui ne souhaitent pas comparer.
- L'approche retenue (catalogue + défaut + opt-in "voir aussi")
  préserve la simplicité par défaut tout en offrant la puissance
  comparative aux utilisateurs avancés (chercheurs, journalistes,
  équipes RSE).

### Alternative 4 — Adopter EcoLogits comme dépendance Python externe

Solution naïve : intégrer EcoLogits via `pyo3` ou un appel CLI à leur
package Python.

**Pourquoi rejeté** :
- Casse la frugalité (`pyo3` = embarquer Python = +60 MB binaire).
- Casse la souveraineté du runtime (Sobr.ia v1.0 = binaire Rust pur,
  pas de stack Python).
- Casse la reproductibilité (versionner Python + EcoLogits dans
  Sobr.ia = enfer de matrice de versions).
- Le port en Rust pur (700 lignes, formules + tests) est faisable
  proprement avec attribution CC BY-SA 4.0 correcte.

---

## Références

- `briefs/chantiers/C24-multi-methodologie-ecologits.md` — brief
  complet du chantier.
- `notebook/validation.qmd` — validation reproductible Python.
- `crates/sobria-core/src/methodology.rs` — `EmpreinteMethod`.
- `crates/sobria-estimator/src/engine_trait.rs` — `EmpreinteEngine`,
  `MethodologyInfo`, `AVAILABLE_METHODS`.
- `crates/sobria-estimator/src/engines/ecologits.rs` — port complet
  EcoLogits 2026-01.
- ADR-0004 Monte-Carlo (étendu, pas remplacé).
- ADR-0011 Réduction périmètre v1.0 (étendu, pas remplacé — M14 À propos
  et nouvelle page `/methodologies` sont toutes deux des paramétrages
  globaux, hors gating persona).
- Rincé S., Banse A., *EcoLogits: Evaluating the Environmental Impacts
  of Generative AI*, JOSS 10(111):7471, 2025. DOI:
  [10.21105/joss.07471](https://doi.org/10.21105/joss.07471).
- AFNOR SPEC 2314, *Référentiel français de mesure de l'empreinte
  environnementale des LLMs*, AFNOR 2024.
