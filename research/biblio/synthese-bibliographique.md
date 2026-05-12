# Synthèse bibliographique — Sobr.ia

> **Version** : 0.1 (squelette S0)
> **Auteur** : Thibault (+ contributions Claude Code)
> **Statut** : structure prête, contenu à remplir au fil des fiches de lecture
> **Cible** : 10-15 p. en fin de S0

---

## 1. Introduction et périmètre

*[à rédiger en fin de S0]*

- Pourquoi mesurer l'impact environnemental de l'IA générative ?
- Quel périmètre se donne Sobr.ia ?
- Quelles questions de recherche sous-jacentes ?

---

## 2. Phases du cycle de vie d'un LLM

### 2.1 Production hardware (embodied)

*Références à mobiliser* : Gupta et al. 2022, Wu et al. 2022, datasheets constructeurs.

*Points à couvrir* :
- Chaîne d'approvisionnement (terres rares, fabrication wafer, packaging)
- Amortissement sur la durée de vie utile
- Variance constructeur (NVIDIA, AMD, Google TPU)

### 2.2 Entraînement

*Références à mobiliser* : Patterson 2021, Luccioni 2023, Strubell 2019.

*Points à couvrir* :
- Ordres de grandeur (GPT-3, LLaMA, BLOOM, Mistral)
- Sensibilité au mix électrique de la zone
- Amortissement entraînement → inférences

### 2.3 Inférence (priorité projet)

*Références à mobiliser* : EcoLogits 2024, ML.Energy, AI Energy Score, Faiz 2024.

*Points à couvrir* :
- Coût par requête (Wh, gCO₂eq, mL d'eau)
- Variance entre modèles
- Effets de la taille du prompt (entrée vs sortie)
- Quantization, distillation, MoE

### 2.4 Fin de vie

*Références* : éclairage Mytton, ADEME.

*Points à couvrir* :
- Recyclage, réemploi
- Émissions de fin de vie (faibles vs cycle de vie total)

---

## 3. Indicateurs et unités

*Tableau à construire* :

| Indicateur | Unité standard | Source de référence | Justification |
|------------|----------------|---------------------|---------------|
| CO₂eq | gCO₂eq | GHG Protocol | métrique unifiée |
| Énergie | Wh | ISO/IEC 21031 | granularité prompt |
| Eau | L | Mytton 2021 | WUE direct + indirect |
| Métaux critiques | mg eq. TR | Gupta 2022 | proxy embodied |

---

## 4. Sources d'incertitude

*Cartographie à produire* :

| Paramètre | Incertitude typique | Distribution recommandée | Source |
|-----------|---------------------|--------------------------|--------|
| ε_decode (mJ/token) | ±30 % | log-normale | HF AI Energy Score |
| PUE | ±10 % | uniforme | datacenters publiés |
| IF électrique | ±5 % horaire | discrète (mix) | RTE, Electricity Maps |
| Embodied / req | ±50 % | log-normale | Gupta 2022 |
| Tokens sortie | ±20 % | log-normale | empirique |

---

## 5. État de l'art des estimations

*Points à couvrir* :
- Convergences entre Patterson 2021 et Luccioni 2023.
- Divergences entre Faiz 2024 (LLMCarbon) et EcoLogits.
- Position particulière de ComparIA (méthodologie EcoLogits ISO 14044).

---

## 6. Méthodologies normatives

- AFNOR SPEC 2314 — référentiel IA frugale (à synthétiser dans `AFNOR-SPEC-2314-synthese.md`)
- ISO/IEC 21031:2024 — méthodologie environnementale ICT
- ITU-T L.1410 — LCA pour les TIC
- GHG Protocol scope 3 catégorie 1

---

## 7. Effets rebond et limites

*Références* : Bender 2021, Jevons (historique).

*Points* :
- Paradoxe de Jevons appliqué à l'IA.
- Limites des estimations bottom-up.
- Limites d'une approche purement technique (économie politique).

---

## 8. Sélection des 3 études de validation croisée

Décision figée en fin de S0. Choix par défaut :
1. **Luccioni 2023 (BLOOM)** — entraînement, méthodologie complète, données publiques.
2. **Patterson 2021** — diversité de modèles, position de référence Google.
3. **EcoLogits 2024** — outil officiel ComparIA, alignement méthodologique.

Tolérance retenue : **±15 % sur le P50**.

---

## 9. Hypothèses retenues pour Sobr.ia

*Synthèse à figer en fin de S0, sera consommée par `sobria-estimator` (S4).*

```toml
# config/methodology.toml (à générer en S0)
[seed]
default = 42

[distributions.epsilon_decode]
type = "log_normal"
mu = ...
sigma = ...
source = "..."

# ... etc
```

---

## 10. Bibliographie complète

Voir `research/biblio/references.bib` (≥ 30 entrées attendues en fin de S0).

---

*Document de travail — à compléter au fil des fiches de lecture.*
