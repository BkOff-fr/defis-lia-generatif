# Sprint S0 — Revue bibliographique et cadrage scientifique

> **Période** : 12 mai 2026 → 18 mai 2026 (5 jours ouvrés)
> **Mode** : exécution conjointe Cowork (synthèse, planification) + Claude Code (extraction, rédaction) + Thibault (validation finale + entretien mentor)
> **Objectif unique** : produire la base scientifique sur laquelle reposera tout le projet. Pas une ligne de code applicatif durant ce sprint.

---

## Pourquoi ce sprint est crucial

Une candidature au défi data.gouv.fr est jugée d'abord sur sa **rigueur méthodologique**. Un outil beau mais avec des chiffres faux est rejeté. Un outil austère mais scientifiquement défendable peut gagner. S0 nous donne la base pour le second cas.

C'est aussi le seul moment où on a le luxe de **tout remettre en question**. Une fois en S1, l'inertie technique s'installe.

---

## Livrables attendus en fin de sprint

| ID | Livrable | Format | Critère d'acceptation |
|----|----------|--------|------------------------|
| L-S0-1 | Synthèse bibliographique | `research/biblio/synthese-bibliographique.md` | 10-15 p., structure imposée (voir §4) |
| L-S0-2 | Bibliographie BibTeX | `research/biblio/references.bib` | ≥ 30 entrées, toutes citées dans L-S0-1 |
| L-S0-3 | Synthèse AFNOR SPEC 2314 | `docs/methodology/AFNOR-SPEC-2314-synthese.md` | 3-5 p. couvrant les exigences applicables |
| L-S0-4 | Cartographie des risques méthodologiques | `docs/methodology/RISQUES-METHODO.md` | ≥ 10 risques identifiés avec mitigation |
| L-S0-5 | Sélection des 3 études de validation croisée | `docs/methodology/VALIDATION-CROISEE.md` | Études figées, paramètres reproductibles documentés |
| L-S0-6 | Distributions d'incertitude par paramètre | `docs/methodology/DISTRIBUTIONS.md` | 6 paramètres clés, distribution + sources |
| L-S0-7 | Glossaire bilingue FR/EN | `docs/methodology/GLOSSAIRE.md` | ≥ 40 termes |
| L-S0-8 | Notes d'entretien mentor (si réalisé) | `research/notes/entretien-mentor-YYYY-MM-DD.md` | Compte-rendu structuré |

---

## Planning détaillé jour par jour

### Jour 1 (lundi) — Cartographie du terrain

**Matin** : lecture rapide des 5 documents cadres :
1. AFNOR SPEC 2314 — Référentiel général pour l'IA frugale (Ecolab / AFNOR)
2. Étude ADEME — Impact environnemental du numérique en France (2022, mise à jour 2024)
3. Feuille de route Numérique & IA — septembre 2025
4. Module pédagogique IA & Environnement (Observatoire IA Paris 1)
5. Rapport ADEME — Regards croisés sur l'IA générative (janvier 2025)

**Après-midi** : constitution de la liste de papers à lire (≥ 30) avec catégorisation :
- Catégorie A : papers d'estimation directe (Patterson, Luccioni, Wu, Strubell, etc.)
- Catégorie B : benchmarks et leaderboards (ML.Energy, AI Energy Score)
- Catégorie C : méthodologies LCA / ICT (ITU-T L.1410, ISO/IEC 21031)
- Catégorie D : impact eau et hardware (Mytton, Li & Ren sur l'eau)
- Catégorie E : effets rebond et sociologie (Jevons, paradoxe de Jevons appliqué)

**Livrable du jour** : `research/biblio/references.bib` rempli à ≥ 30 entrées avec abstracts.

### Jour 2 (mardi) — Lectures profondes catégorie A et B

**Toute la journée** : lecture intégrale de :
- Luccioni, Viguier, Ligozat (2023) — *Estimating the Carbon Footprint of BLOOM*
- Patterson et al. (2021) — *Carbon Emissions and Large Neural Network Training*
- Patterson et al. (2022) — *The Carbon Footprint of Machine Learning Training Will Plateau*
- Faiz et al. (2024) — *LLMCarbon: Modeling the End-to-End Carbon Footprint*
- Touvron et al. (2023) — *Llama 2: Open Foundation and Fine-Tuned Chat Models* (annexes énergie)
- EcoLogits whitepaper (Data for Good, 2024)

**Pour chacun** : fiche de lecture standard (modèle ci-dessous §6).

**Livrable du jour** : 6 fiches de lecture catégorie A intégrées à la synthèse en cours.

### Jour 3 (mercredi) — Lectures eau, hardware, embodied

**Matin** :
- Mytton (2021) — *Data centre water consumption*
- Li, Ren et al. (2023) — *Making AI Less "Thirsty": Uncovering and Addressing the Secret Water Footprint of AI Models*
- Gupta et al. (2022) — *Chasing Carbon: The Elusive Environmental Footprint of Computing*

**Après-midi** :
- AFNOR SPEC 2314 — lecture intégrale et synthèse (L-S0-3)
- Tableau de correspondance : exigences SPEC 2314 ↔ modules Sobr.ia

**Livrable du jour** : L-S0-3 rédigée.

### Jour 4 (jeudi) — Méthodo, incertitude, validation croisée

**Matin** :
- ISO/IEC GUM (Guide to expression of Uncertainty in Measurement) — extraits applicables
- Choix des 3 études de validation croisée (Luccioni 2023, Patterson 2021, EcoLogits 2024)
- Pour chaque étude : extraction des paramètres reproductibles + résultat cible + tolérance ±15 %

**Après-midi** :
- Définition des distributions d'incertitude par paramètre (L-S0-6)
- Cartographie des risques méthodologiques (L-S0-4)

**Livrable du jour** : L-S0-4, L-S0-5, L-S0-6 rédigées.

### Jour 5 (vendredi) — Synthèse + entretien mentor + figeage

**Matin** : rédaction finale de la synthèse bibliographique (L-S0-1), du glossaire (L-S0-7).

**Après-midi** :
- Entretien mentor Ecolab/ADEME (1 h) si planifié
- Compte-rendu (L-S0-8)
- Figeage de toutes les hypothèses dans le code de l'estimateur (config TOML qui sera consommé par Rust en S4)

**Livrable du jour** : L-S0-1 finale + L-S0-7 + L-S0-8.

---

## Structure imposée de la synthèse bibliographique (L-S0-1)

```
1. Introduction et périmètre (1 p.)
2. Phases du cycle de vie d'un LLM (2 p.)
   2.1 Production hardware (embodied)
   2.2 Entraînement
   2.3 Inférence (priorité projet)
   2.4 Fin de vie
3. Indicateurs et unités (1 p.)
   - CO₂eq, énergie, eau, métaux critiques
4. Sources d'incertitude (2 p.)
   - Hardware, datacenter, modèle, usage
5. État de l'art des estimations (3 p.)
   - Patterson 2021, Luccioni 2023, EcoLogits, ML.Energy
   - Convergences et divergences
6. Méthodologies normatives (2 p.)
   - AFNOR SPEC 2314, ISO/IEC 21031, ITU-T L.1410
7. Effets rebond et limites (1 p.)
8. Sélection des 3 études de validation croisée (1 p.)
9. Hypothèses retenues pour Sobr.ia (2 p.)
10. Bibliographie complète (références)
```

---

## Modèle de fiche de lecture

```markdown
# [Auteurs] (Année) — [Titre court]

## Référence complète
`@article{key, ...}` au format BibTeX, à intégrer dans `references.bib`.

## Question de recherche
[1 phrase]

## Méthodologie
- Données utilisées
- Modèles couverts
- Métriques calculées
- Hypothèses clés

## Résultats principaux (chiffres)
- [Indicateur 1] = X (intervalle Y)
- [Indicateur 2] = Z

## Limites reconnues par les auteurs
- ...

## Pertinence pour Sobr.ia
- À utiliser pour : [validation / paramétrage / contexte]
- Distributions extractibles : ...
- Risque méthodologique soulevé : ...

## Notes personnelles
[Critique, doute, lien avec autres papers]
```

---

## Sources prioritaires à acquérir

| Source | Accès | Coût | Priorité |
|--------|-------|------|----------|
| AFNOR SPEC 2314 | Site AFNOR | Gratuit (publié 2024) | P0 |
| Études ADEME | ADEME Infos + ecologie.gouv.fr | Gratuit | P0 |
| Papers académiques (arXiv, ACL, NeurIPS) | arXiv, Semantic Scholar | Gratuit | P0 |
| ML.Energy Leaderboard | ml.energy | Gratuit | P0 |
| EcoLogits + GenAI Impact | GitHub Data for Good | Gratuit | P0 |
| ISO/IEC 21031 | ISO | Payant (~150 €) — utiliser bibliothèque uni | P1 |
| ITU-T L.1410 | ITU | Gratuit | P1 |

---

## Critères de validation du sprint (DoD)

- [ ] Tous les livrables L-S0-1 à L-S0-7 produits et committés
- [ ] L-S0-1 relue par Thibault et validée
- [ ] BibTeX `references.bib` parsable (`biber --tool`)
- [ ] Aucune affirmation chiffrée sans citation
- [ ] Le tableau de distributions (L-S0-6) est implémentable tel quel en S4
- [ ] Décision de scope finale prise (LLMs uniquement confirmé / pas d'autre modalité)

---

## Anti-patterns à éviter

- ❌ Vouloir lire 100 papers superficiellement → lire 30 papers à fond.
- ❌ Citer un paper qu'on n'a pas vraiment lu.
- ❌ Adopter une formule sans comprendre ses hypothèses.
- ❌ Ignorer les sources critiques (Bender, sceptiques) — au contraire les inclure.
- ❌ Reporter à S1 ce qui doit être figé en S0.

---

## Risques du sprint et mitigations

| Risque | Mitigation |
|--------|-----------|
| Mentor indisponible cette semaine | Reporter l'entretien à S1 ou S2, ne pas bloquer |
| AFNOR SPEC 2314 difficile d'accès | Backup : module pédagogique Observatoire IA + référentiels équivalents |
| Surcharge cognitive (30 papers en 5 jours) | Discipline fiche standardisée, lecture en diagonale pour catégorie C/E |
| Tentation de coder | Interdit cette semaine. Repo prêt mais aucun commit applicatif. |

---

## Sortie du sprint

À la fin de S0, on doit pouvoir répondre à ces 5 questions sans hésiter :

1. Quelle est la formule de calcul officielle de Sobr.ia, paramètre par paramètre ?
2. Quelles 3 études allons-nous reproduire pour valider notre moteur ?
3. Quelle distribution d'incertitude pour chaque paramètre, avec quelle source ?
4. Quelles exigences AFNOR SPEC 2314 s'appliquent à nous, et comment on les satisfait ?
5. Quels sont les 5 risques méthodologiques principaux et leur mitigation ?

Si on ne peut pas, S1 ne démarre pas.
