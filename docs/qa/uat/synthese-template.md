# Synthèse UAT v1.0 — campagne 5 personas (modèle à remplir)

> **Après les 5 sessions** : copier ce fichier en `docs/qa/uat-synthese-v1.0.md`
> (livrable C36 §4) et le remplir à partir des grilles `sessions/uat-session-*.md`.
> Cette synthèse décide de ce qui est **corrigé avant la candidature** (tag v1.0)
> et de ce qui part au **backlog v1.x** — règle C36 §8 : en cas de doute, on patche.

---

## 0. Carte d'identité de la campagne

| Champ | Valeur |
|---|---|
| Dates des sessions | du ____ au ____ |
| Sessions réalisées | __ / 5 (personas couverts : ____) |
| Formats | __ live / __ async |
| Version app testée | v0.9.__ |
| Grilles sources | sessions/uat-session-*.md (lister) |
| Rédacteur synthèse | |

## 1. Métriques

### 1.1 SUS — par persona et global

| Persona | Testeur | Score SUS /100 |
|---|---|---|
| Étudiant·e | T_ | |
| Pro tech | T_ | |
| Entreprise | T_ | |
| Collectivité | T_ | |
| Chercheur·se | T_ | |
| **Moyenne** | | **____** |

**Cible C36 : ≥ 70.** Atteinte : oui / non.
Si < 70 : reporter la candidature de 1-2 semaines et intégrer les findings
(mitigation prévue C36 §7).

### 1.2 Objectif « use case clé < 5 min sans aide » (objectif central C36)

| Persona | Tâche clé | Temps | Sans aide ? | Atteint ? |
|---|---|---|---|---|
| Étudiant·e | S2 estimer une question | | | |
| Pro tech | P2 comparer 3 modèles | | | |
| Entreprise | E1 comprendre l'offre équipe | | | |
| Collectivité | C4 rapport réglementaire | | | |
| Chercheur·se | R3 reproductibilité | | | |

### 1.3 Taux de réussite par tâche

Réussite stricte = « oui » sans aide. Reporter chaque tâche jouée
(IDs : `taches-personas.md`).

| Tâche | Réussite | Avec aide | Échec | Non jouée | Temps médian (seuil) | Gravité max observée |
|---|---|---|---|---|---|---|
| S1 | | | | | | |
| S2 | | | | | | |
| S3 | | | | | | |
| S4 | | | | | | |
| S5 | | | | | | |
| S6 | | | | | | |
| P1-P6 | | | | | | |
| E1-E5 | | | | | | |
| C1-C5 | | | | | | |
| R1-R5 | | | | | | |

(Éclater les lignes P/E/C/R par tâche au remplissage.)

### 1.4 Découvrabilité du rail « Plus » (demande C39 §6)

| Testeur | « Plus » découvert sans aide ? | Quand / déclencheur |
|---|---|---|
| T1 | | |
| T2 | | |
| T3 | | |
| T4 | | |
| T5 | | |

Conclusion (le chevron suffit-il ? label ? autre ?) :

## 2. Top 5 frictions — gravité × fréquence

**Score de priorité** = gravité (bloquant 4, majeur 3, mineur 2, cosmétique 1)
× fréquence (nombre de testeurs touchés, 1-5). On classe par score décroissant ;
à score égal, la friction qui touche une tâche « clé < 5 min » passe devant.

| Rang | Friction (où, quoi) | Tâches touchées | Gravité | Fréquence /5 | Score | Décision |
|---|---|---|---|---|---|---|
| 1 | | | | | | corriger avant candidature / backlog v1.x / ne rien faire |
| 2 | | | | | | |
| 3 | | | | | | |
| 4 | | | | | | |
| 5 | | | | | | |

Frictions hors top 5 mais notables (liste rapide + décision) :

-

## 3. Verbatims marquants

### 3.1 Cinq verbatims positifs (DoD C36 — pour le dossier de candidature)

> Choisir des citations courtes, anonymisées (T1-T5 + persona), qui parlent
> du produit (clarté, confiance, utilité), réutilisables telles quelles.

1. « ____ » — T_, persona ____
2. « ____ » — T_, persona ____
3. « ____ » — T_, persona ____
4. « ____ » — T_, persona ____
5. « ____ » — T_, persona ____

### 3.2 Verbatims révélateurs (négatifs ou ambivalents)

> Ceux qui justifient les décisions du §4 — un par friction majeure si possible.

1. « ____ » — T_, contexte : tâche __
2. « ____ » — T_, contexte : tâche __
3. « ____ » — T_, contexte : tâche __

### 3.3 Questions ouvertes — lecture transversale

| Question | Tendance sur 5 testeurs |
|---|---|
| Un mot pour Sobr.ia | |
| Surprises récurrentes | |
| Recommanderait pour… | |
| LA chose à ajouter (votes) | |
| LA chose à retirer (votes) | |
| Phase 1 : « c'est quoi Sobr.ia » correct du premier coup ? | __ / 5 |

## 4. Décisions

### 4.1 Corriger avant candidature (tag v1.0) — patches express

Règle C36 : top 3 frictions patchées avant le tag ; s'il faut choisir entre
tagger et patcher, on patche.

| # | Correctif | Friction d'origine (rang §2) | Où (C35 / patch v0.9.x) | Effort estimé | Responsable | Statut |
|---|---|---|---|---|---|---|
| 1 | | | | | | |
| 2 | | | | | | |
| 3 | | | | | | |

### 4.2 Backlog v1.x (assumé, non bloquant)

| Sujet | Justification du report | Cible (v1.1 / v1.2 / à rediscuter) |
|---|---|---|
| | | |

### 4.3 Non-problèmes constatés (à ne pas « corriger »)

> Ce que l'équipe croyait cassé et qui passe bien en test — pour éviter le
> polish inutile.

-

## 5. Definition of Done C36 — état final

- [ ] 5 sessions UAT réalisées (sinon : __ / 5, personas manquants : ____)
- [ ] 5 grilles `sessions/uat-session-*.md` remplies
- [ ] Cette synthèse complétée et relue par Thibault
- [ ] Score SUS global ≥ 70 (réel : ____)
- [ ] Top 3 frictions patchées avant tag v1.0 (réf. commits/PR : ____)
- [ ] 5 verbatims positifs transmis au dossier candidature data.gouv.fr

## 6. Annexes

- Grilles de session : `docs/qa/uat/sessions/`
- Enregistrements : emplacement ____ (suppression prévue le ____ — engagement
  consentement : ≤ 6 mois)
- Issues GitHub ouvertes suite à l'UAT : ____
