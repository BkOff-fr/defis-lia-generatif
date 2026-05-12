# Brief — Entretien mentor scientifique (S0)

> **Objectif** : sécuriser une heure avec le mentor Ecolab/ADEME en S0 pour caler la méthodologie tant qu'on a la flexibilité.
> **Durée cible** : 60 min.
> **Format** : visio (Tixeo / BBB préférés) ou présentiel.
> **Livrable** : compte-rendu structuré à committer en `research/notes/entretien-mentor-YYYY-MM-DD.md`.

---

## Email d'invitation (template)

```
Objet : Sobr.ia — relecture méthodologique pour le défi data.gouv.fr (1 h)

Bonjour [Prénom],

Je porte une candidature au défi data.gouv.fr « L'impact environnemental
de l'IA générative » sous le nom Sobr.ia, et je serais ravi de bénéficier
de votre regard méthodologique avant de figer mes choix.

Le projet en bref :
• Une application native multi-plateforme (Rust + Tauri + SvelteKit) qui
  exploite les jeux ComparIA et RTE/NaTran/Teréga IRIS publiés sur
  data.gouv.fr, dans une stack frugale (binaire < 20 Mo, RAM < 100 Mo).
• Méthodologie alignée sur AFNOR SPEC 2314 et EcoLogits (ISO 14044),
  propagation d'incertitude Monte-Carlo N=10⁴, validation croisée sur
  Luccioni 2023, Patterson 2021, et EcoLogits.
• Architecture médaillon stricte (Copper → Silver → Gold) pour la
  traçabilité de bout en bout.

Je souhaiterais une heure avec vous, idéalement la semaine du [date], pour
vérifier que la méthodologie tient la route et anticiper les angles morts.

Disponibilités proposées :
  - [créneau 1]
  - [créneau 2]
  - [créneau 3]

Pièces jointes :
  • Cahier des charges v1.2 (PDF)
  • Liste des 3 études de validation croisée envisagées
  • Catalogue des 8 sources retenues

Merci pour votre temps.
[Signature]
```

---

## Préparation matérielle (à apporter)

- Cahier des charges v1.2 (`docs/CAHIER-DES-CHARGES-v1.0.md`).
- Catalogue sources (`docs/sources/CATALOGUE-SOURCES.md`).
- Roadmap (`docs/ROADMAP.md`).
- 10 fiches de lecture déjà rédigées (papers prioritaires).
- Première version du tableau « Distributions par paramètre » (`docs/methodology/DISTRIBUTIONS.md`).
- Stylo + carnet (prise de notes manuscrite recommandée pour rester actif).
- Magnétophone (avec accord explicite du mentor).

---

## Trame d'entretien — 60 min chrono

### 1. Introduction (5 min)
- Présenter le projet en 2 min : 11 + 1 modules, stack Rust+Tauri, datasets officiels du défi.
- Rappeler le cadre de la candidature et l'objectif de l'entretien : **valider, challenger, anticiper**.
- Demander l'accord pour enregistrer (pour le compte-rendu).

### 2. Validation méthodologique (15 min)

#### 2.1 Formule de référence
- Présenter la formule (CO₂eq prompt) telle qu'elle figure au CDC §9.1.
- Questions ouvertes :
  - Est-ce que la décomposition compute / cooling / embodied est défendable ?
  - Y a-t-il des termes manquants à vos yeux (par exemple : transmission réseau, terminal utilisateur) ?
  - Sur quels termes faut-il être le plus prudent dans la communication grand public ?

#### 2.2 Propagation d'incertitude
- Justifier le choix Monte-Carlo N=10⁴, seed 42.
- Demander :
  - Validation des distributions par défaut (log-normale, uniforme, normale).
  - Est-ce que P5-P95 est la bonne granularité de restitution ?
  - Faut-il publier les paramètres des distributions (transparence) ou seulement les résultats ?

### 3. Validation des sources (10 min)

- Présenter le Tier 1 (ComparIA + RTE IRIS) et Tier 2 (ADEME, EcoLogits…).
- Questions :
  - Manque-t-il une source incontournable ?
  - Quelles précautions sur ComparIA (biais d'utilisation, secret statistique, hétérogénéité modèles) ?
  - Comment traiter les modèles **fermés** (GPT, Claude) dont on ne connaît pas les paramètres exacts ?
  - Conseil sur le traitement du **secret statistique** pour les petits IRIS ?

### 4. Validation croisée et tolérance (10 min)

- Présenter les 3 études cibles (Luccioni 2023, Patterson 2021, EcoLogits 2024).
- Questions :
  - Tolérance ±15 % défendable ou trop laxiste ?
  - Faut-il ajouter un quatrième cas de test (suggestion ?) ?
  - Quel format de publication des résultats de validation (table dans le notebook, annexe rapport, dépôt séparé) ?

### 5. Angles morts et risques (10 min)

- Demander : « Quels sont les pièges classiques que vous avez vus chez des projets similaires ? »
- Demander : « Sur quoi le jury sera-t-il le plus sceptique ? »
- Demander : « Que feriez-vous différemment si vous étiez à ma place ? »

### 6. Communication et restitution (5 min)

- Question : « Comment vulgariser sans trahir scientifiquement ? »
- Question : « Sur quels équivalents grand public (km voiture, douches…) êtes-vous d'accord, lesquels vous gênent ? »
- Question : « Recommanderiez-vous d'inclure un avertissement standard sur l'usage des chiffres ? »

### 7. Engagement et suite (5 min)

- Demander : peut-on revenir vers vous pour une relecture du notebook Quarto en S9 ?
- Demander : accepteriez-vous d'apparaître nommément dans les remerciements ?
- Demander : connaissez-vous un autre expert utile à contacter (eau, hardware, territoires) ?

---

## Compte-rendu standard

À publier dans `research/notes/entretien-mentor-YYYY-MM-DD.md`.

```markdown
# Compte-rendu — Entretien mentor [nom prénom]
Date : YYYY-MM-DD
Durée : XX min
Format : visio / présentiel
Mentor : [nom, organisation, fonction]

## Validations
- [ ] Formule de référence : OK / réserve
- [ ] Propagation Monte-Carlo : OK / réserve
- [ ] Sources Tier 1+2 : OK / réserve
- [ ] Validation croisée à ±15 % : OK / réserve

## Modifications demandées
1. ...
2. ...

## Risques signalés
- ...

## Suggestions d'experts complémentaires
- ...

## Engagements
- [ ] Relecture notebook en S9 : oui / non
- [ ] Mention dans les remerciements : oui / non
- [ ] Diffusion de la candidature dans son réseau : oui / non

## Citations marquantes
> « ... »

## Actions immédiates (≤ 48 h)
- [ ] Mettre à jour le CDC sur le point X
- [ ] Ouvrir une issue GitHub sur le risque Y
- [ ] Envoyer email de remerciement
```

---

## Anti-patterns à éviter pendant l'entretien

- ❌ Tomber dans le pitch commercial. C'est un échange technique.
- ❌ Défendre nos choix sans écouter les objections. C'est le moment de douter.
- ❌ Promettre plus que ce qu'on peut tenir.
- ❌ Oublier de demander la permission d'enregistrer.
- ❌ Quitter sans prochaine échéance fixée.

---

*Une heure bien préparée, c'est une candidature qui gagne en solidité.*
