# Audit produit Sobr.ia — Q3 2026

> **Statut** : v1 livré, 2026-05-16
> **Auteur** : Cowork
> **Périmètre** : audit de **clarté produit + clarté UX par persona** avant la candidature data.gouv.fr v1.0.
> **Périmètre exclus** : performance technique, bugs, tests E2E (couverts par d'autres chantiers).
> **Méthodologie** : walkthrough des 5 personas définies en ADR-0010, observation directe des surfaces produit (README, onboarding, page d'accueil, rail nav, paramètres, extension, mode Équipe), analyse messaging.

---

## Synthèse exécutive

Sobr.ia est **techniquement très solide** (multi-méthodologie, audit SHA-256, mode Équipe self-hosted, extension navigateur, pipeline médaillon, 19 sources cataloguées). C'est un produit fini.

**Mais** côté clarté UX, il y a un gap entre la **richesse technique** et la **compréhension utilisateur**. Plus précisément :

1. **Le produit demande "qui es-tu ?" avant d'expliquer "qu'est-ce que c'est ?"** — l'onboarding va trop vite vers la sélection de persona sans poser le problème que résout Sobr.ia.
2. **Le messaging est dev-centrique** — README, taglines, labels modules truffés de jargon (AFNOR SPEC 2314, Monte-Carlo, PROV-O, Datasheet Gebru, K_DECODE…) que seul un dev/chercheur comprend.
3. **Les module IDs M1, M3, M7… restent visibles dans l'UI** — l'utilisateur n'a pas à voir "M7" dans une fiche. Cuisine interne qui fuit.
4. **Manque de fil narratif entre les modules** — un utilisateur qui ouvre l'app voit 13 modules sur le rail sans hiérarchie ni parcours suggéré.
5. **Le dual-track local / Mode Équipe / Cloud (futur) n'est pas raconté simplement** — l'utilisateur ne sait pas dans quel mode il est.
6. **Les vendors disclosure (Mistral × ADEME, Google, Meta) ne sont pas encore exposés** — le scoop pitch 2026 absent du produit.

**Verdict global** : produit prêt à 80 %, **20 % manquants = clarté narrative + messaging + signage**. Ce sont des changements légers techniquement (textes, ordering, tooltips, refonte onboarding) mais critiques pour la candidature data.gouv.fr et pour l'adoption réelle.

**Estimation effort de correction** : 3-5 jours de delivery (chantier C32.1 → C32.5 décrit en fin de rapport).

---

## Méthodologie d'audit

### 5 personas du CLAUDE.md / ADR-0010

| Persona | Tagline (preferences.ts) | Bundle modules par défaut |
|---|---|---|
| `student` | « Apprendre, comprendre, suivre votre usage IA » | M1 + M8 + M13 + M14 + M15 + M25 (6 modules) |
| `pro_tech` | « Estimer, comparer, journaliser pour vos intégrations » | M1 + M3 + M7 + M8 + M9 + M13 + M14 (7 modules) |
| `enterprise` | « Piloter votre scope 3 IA, rapport CSRD, forecast budget carbone » | M1 + M7 + M12 + M14 + M15 + M17 + M20 + M22 + M25 (9 modules) |
| `public_sector` | « Suivre votre empreinte territoriale, marchés publics frugaux » | M1 + M8 + M12 + M14 + M17 + M20 + M22 (7 modules) |
| `researcher` | « Reproductibilité, comparaisons inter-modèles, datasets publiables » | M1 + M3 + M7 + M8 + M9 + M14 + M17 (7 modules) |

### Grille d'évaluation

Pour chaque persona on évalue :
1. **30 sec pitch** — comprend-il l'utilité en 30 sec ?
2. **Onboarding journey** — où décroche-t-il ?
3. **Top use cases** — y a-t-il un parcours guidé pour ses 2-3 cas d'usage canoniques ?
4. **Points de confusion** — jargon, charge cognitive, modules non pertinents.
5. **Gaps clarté** — manques de messaging, tooltips, exemples.
6. **Score clarté** /10.

---

# 🎓 Persona 1 — Student / Curieux

### Bundle actuel : M1 + M8 + M13 + M14 + M15 + M25

### 30 sec pitch ?

❌ **Non.** Si je suis étudiant et que je tombe sur Sobr.ia, le README m'annonce dès la phrase 2 :

> *« angle territorial français unique (ComparIA × RTE IRIS) »* avec *« rigueur scientifique auditable (AFNOR SPEC 2314, Monte-Carlo, audit ledger SHA-256) »*

Aucun étudiant ne sait ce que c'est ComparIA, RTE IRIS, AFNOR SPEC 2314, ledger SHA-256, Monte-Carlo. Il ferme l'onglet.

### Onboarding

Étape 1 splash : auto-advance 3 s, peu de texte → OK.
Étape 2 persona picker : on lui demande "qui es-tu ?" alors qu'il ne sait pas encore ce que Sobr.ia est. Charrue avant bœufs.
Étape 3 bundle : il voit 6 modules pré-cochés mais ne sait pas pourquoi M14 "À propos" est dans son bundle (ce n'est pas un module fonctionnel, c'est de la doc). M25 Eco-budget : "Eco-budget" parle, mais pas évident à 18 ans qu'on doive *fixer un budget CO₂* avant de commencer.
Étape 4 premier prompt : c'est bien, tooltip lime sur le sélecteur de modèle.

### Use cases canoniques manquants

Ce qu'un étudiant veut probablement faire :
1. *« Combien coûte 1 question à ChatGPT en CO₂ ? »* — réponse en 5 secondes, en grammes, comparable à un km de voiture.
2. *« Quel modèle pollue le moins parmi ceux que j'utilise ? »* — comparaison visuelle, idéalement avec note A-F comme un Nutri-Score.
3. *« Mon usage cette semaine = combien de douches ? »* — chiffres relatables (douches, km voiture, kWh frigo).

Le produit actuel **a tout pour le faire** (M1 + M3 + M15) mais ne **raconte pas l'histoire**. M15 Dashboard affiche probablement des gCO₂eq sans équivalence "douches/voiture".

### Points de confusion

- Module M14 « À propos » dans le bundle : pourquoi ?
- Module M25 « Eco-budget » : fixer un budget avant de commencer à utiliser ? Inverser : M15 d'abord pour voir l'usage, M25 ensuite pour fixer un objectif.
- Aucun parcours "Découvrir" guidé.

### Score clarté : **4/10**

Le contenu est là mais le narratif manque cruellement. Refonte messaging + ajout d'équivalences "humaines" (douches, km, kWh frigo) = saut massif.

---

# 💻 Persona 2 — Pro Tech (dev, ML eng)

### Bundle actuel : M1 + M3 + M7 + M8 + M9 + M13 + M14

### 30 sec pitch ?

✅ **Oui, partiellement.** Pour un dev qui connaît EcoLogits, BoaVizta, AI Energy Score, le README parle. Multi-méthodologie + audit chaîné + référentiel modèles → ça résonne.

❌ **Mais** le différenciateur "vendor disclosure agrégé" (Mistral × ADEME, Google Gemini, Meta Llama) n'apparaît pas encore dans le produit (intégration prévue C31). C'est le scoop pitch 2026 absent.

### Onboarding

Bon. Bundle persona ProTech inclut M1 (estimer) + M3 (comparer) + M7 (journal) + M9 (référentiel) — exactement ses 4 use cases canoniques. ✓

### Use cases canoniques

1. ✅ *« J'intègre l'API OpenAI dans une app — quel modèle minimise l'empreinte ? »* → M3 Comparer.
2. ✅ *« Je veux logger toutes mes estimations pour reporting trimestriel »* → M7 Journal.
3. ⚠️ *« Je veux automatiser la mesure via CLI / API »* → manquant. Sobr.ia est GUI-only en v0.7. Pas d'API REST ni CLI prête.
4. ⚠️ *« Je veux comparer ma méthodologie maison à AFNOR/EcoLogits »* → présent dans M1 panneau "Voir aussi" mais pas mis en avant.

### Points de confusion

- L'extension navigateur est-elle complémentaire à l'app ou alternative ? Pas clair côté messaging.
- Mode Équipe vs perso : sur quel critère choisir ?
- Les module IDs (M1, M3...) sont des artefacts internes inutiles pour le dev.

### Score clarté : **7/10**

Bon socle, manque d'expliciter les **points d'entrée techniques** (CLI ? API REST ? lib Rust embeddable ?). Doc dev manquante.

---

# 🏢 Persona 3 — Entreprise (DSI, RSE)

### Bundle actuel : M1 + M7 + M12 + M14 + M15 + M17 + M20 + M22 + M25

### 30 sec pitch ?

✅ **Oui, sur la valeur métier.** "Scope 3 IA + rapport CSRD" est lisible pour un responsable RSE. M22 Rapport CSRD/AGEC parle.

❌ **Mais** la mise en œuvre concrète n'est pas claire : *« Je suis DSI dans une PME, comment je déploie Sobr.ia pour 50 collaborateurs ? »* — il faut creuser jusqu'à v0.7.0 Mode Équipe self-hosted pour comprendre. Pas dans le README en première page.

### Onboarding

❌ **Trou** : un DSI qui veut tester le Mode Équipe doit deviner qu'il faut :
1. Installer l'app Tauri sur son poste admin.
2. **Aussi** lancer `sobria-team-aggregator init` en CLI (binaire Rust séparé).
3. Configurer le TLS, créer admin, créer codes, distribuer aux employés.

Ce parcours **n'est documenté nulle part dans l'onboarding produit**, juste dans `docs/operations/team-aggregator.md`. Un DSI ne sait pas que ce fichier existe.

### Use cases canoniques

1. ✅ *« Reporting CSRD trimestriel »* → M22 Rapport CSRD/AGEC.
2. ⚠️ *« Suivre l'usage IA de mes 50 collaborateurs »* → Mode Équipe, mais install non guidé.
3. ⚠️ *« Définir un budget IA par équipe avec alertes »* → M25 Eco-budget personnel + alertes seuils v0.7.1, mais pas exposé comme un parcours équipe.
4. ❌ *« Justifier mon choix de Mistral plutôt qu'OpenAI sur des critères empreinte »* → pas de comparaison équipe dans M3.

### Points de confusion

- Trop de modules cochés par défaut (9 modules), charge cognitive élevée.
- Mode Équipe = nouveau crate `sobria-team-aggregator` non installé par défaut. Pas évident.
- Pas de quickstart "5 minutes — déployer Sobr.ia dans mon entreprise".

### Score clarté : **5/10**

Tout est là techniquement (mode Équipe v0.7.0 + alertes v0.7.1 + exports CSRD), mais **le DSI ne sait pas où commencer**. Manque cruel de doc quickstart non-IT-friendly.

---

# 🏛️ Persona 4 — Public Sector (collectivité, service public)

### Bundle actuel : M1 + M8 + M12 + M14 + M17 + M20 + M22

### 30 sec pitch ?

✅ **Oui** sur "angle territorial FR". M20 Territoire FR + M22 Rapport CSRD est exactement le langage des collectivités.

### Onboarding

✓ Bundle correct (centré territoire + reporting).

### Use cases canoniques

1. ✅ *« Mesurer l'empreinte IA dans nos marchés publics »* → M22.
2. ✅ *« Cartographier l'usage IA par IRIS »* → M20 Territoire FR.
3. ⚠️ *« Comparer mes datacenters régionaux »* → M12 mais centré Europe pas exclusivement FR.
4. ❌ *« Cadre marchés publics frugaux avec critères mesurables »* → tagline le promet mais pas de module dédié.

### Points de confusion

- Tagline "marchés publics frugaux" non délivrée par un module concret.
- Pas de template "réponse type appel d'offre" ou "critères mesurables IA frugale".

### Score clarté : **6/10**

Le territoire FR est un vrai différenciateur. Manque l'opérationnalisation marchés publics (livrable réutilisable).

---

# 🔬 Persona 5 — Researcher / Journaliste

### Bundle actuel : M1 + M3 + M7 + M8 + M9 + M14 + M17

### 30 sec pitch ?

✅ **Oui, fortement.** Datasheet Gebru + reproductibilité scientifique + multi-méthodologie + audit chaîné — c'est exactement le vocabulaire académique.

✅ Le notebook Quarto `notebook/validation.qmd` (livré en B3) est un atout pour chercheurs.

### Onboarding

✓ Bundle correct.

### Use cases canoniques

1. ✅ *« Reproduire les chiffres Sobr.ia pour mon papier »* → notebook Quarto + datasheet Gebru.
2. ✅ *« Comparer 5 modèles selon 2 méthodologies »* → M3 + catalogue méthodos.
3. ⚠️ *« Citer Sobr.ia avec un DOI »* → pas de DOI publié à date. À discuter (Zenodo ?).
4. ✅ *« Exporter mon dataset d'estimations pour analyse externe »* → M22 + JSON-LD PROV-O.

### Points de confusion

- DOI manquant.
- Pas clair si les datasets utilisés (ComparIA, IRIS) sont citables comme tels avec leur lineage SHA-256.

### Score clarté : **8/10**

Le mieux servi des 5 personas. Quelques petits manques (DOI, citation guidée).

---

# 📊 Synthèse par persona

| Persona | Score clarté | Plus gros gap |
|---|---|---|
| Student | 4/10 | Aucun narratif "humain" (douches/voiture) + jargon |
| Pro Tech | 7/10 | Pas de CLI / API / lib embeddable documentée |
| Enterprise | 5/10 | Mode Équipe install non guidé, DSI perdu |
| Public Sector | 6/10 | Tagline marchés publics non délivrée par un module |
| Researcher | 8/10 | DOI manquant |
| **Moyenne** | **6/10** | |

L'ordre du best-served au moins-served est : **Researcher > Pro Tech > Public Sector > Enterprise > Student**.

Ironiquement, **Student est notre persona avec le plus gros potentiel d'adoption volume** (curiosité, viral, éducation) et c'est le moins bien servi. C'est notre plus grosse opportunité d'amélioration.

---

# 📣 Audit messaging

## README

- **Forces** : exhaustif, transparent, sourcé, citations DOIs.
- **Faiblesses** : dev-centrique, jargon dense en première page, pas de section "Pour qui ? Que résout-il ?".
- **Reco** : ajouter en tête un bloc **"Sobr.ia, c'est quoi ?"** en langage simple + section "Pour qui ?" avec 5 cartes persona + lien doc spécifique.

## Onboarding

- **Forces** : 4 étapes propre, splash + persona + bundle + premier prompt.
- **Faiblesses** : étape 1 (splash 3s) ne pose pas le problème. Étape 2 (persona) demande "qui es-tu ?" sans contexte produit.
- **Reco** : insérer entre splash et persona une **étape "Sobr.ia en 30 secondes"** (≤ 4 phrases + 1 schéma équivalence carbone) pour poser le problème AVANT de demander le persona.

## Labels modules (rail)

- "M1 Estimer un prompt" — OK
- "M3 Comparer modèles" — OK
- "M7 Journal d'audit" — OK pour pro/researcher, opaque pour student
- "M9 Référentiel modèles" — opaque pour tous sauf experts
- "M12 Datacenters Europe" — OK
- "M13 Simulateur Et si...?" — OK
- "M14 À propos" — ne devrait PAS être dans un bundle (pas un module fonctionnel)
- "M15 Dashboard" — OK
- "M17 Empreinte projet" — opaque, on dirait un projet d'entreprise alors que c'est la datasheet Gebru
- "M20 Territoire FR" — OK
- "M22 Rapport CSRD/AGEC" — OK pour entreprise/public, opaque pour student
- "M25 Eco-budget" — OK

**Reco** : 
1. Renommer **M9 "Référentiel modèles" → "Bibliothèque modèles"** ou **"Modèles IA"**.
2. Renommer **M17 "Empreinte projet" → "Datasheet scientifique"** ou **"Fiche reproductibilité"**.
3. **Retirer M14 "À propos" des bundles** — ce module doit être accessible mais pas un point d'entrée par défaut.
4. **Retirer "M1", "M3"... des labels visibles** — c'est de la cuisine interne. Le label utilisateur c'est "Estimer", "Comparer". Les IDs restent en URL et dans le code.

## Extension navigateur

- **Forces** : popup compacte avec total journalier + badge en page.
- **Faiblesses** : la première fois qu'un user installe l'extension, il ne sait pas qu'il faut **aussi installer l'app Tauri + faire un pairing** pour avoir un suivi consolidé. Le bandeau "App non détectée" pourrait être plus pédagogique.
- **Reco** : améliorer le bandeau et lier vers une page "Comment ça marche ? Extension seule vs avec app desktop".

## Mode Équipe

- **Forces** : architecture saine (binaire self-hosted, JWT, Argon2id).
- **Faiblesses** : aucun parcours guidé d'install côté UI. Un DSI doit lire `docs/operations/team-aggregator.md` (existe) pour comprendre.
- **Reco** : ajouter un panneau "Activer Mode Équipe" dans l'app Tauri qui propose **un téléchargement du binaire serveur** + **script d'init en 1 clic** + **scripts d'enrollment** clés en main.

---

# 🎯 Audit positionnement

## Vs concurrents directs

| Outil | Méthodologie | Open source | Self-hostable | Mode équipe | Cloud | Pitch unique |
|---|---|---|---|---|---|---|
| **EcoLogits** (Genmo) | EcoLogits seul | ✅ | API only | ❌ | ❌ | Référence acad. |
| **AI Energy Score** (HF) | AI Energy Score seul | ✅ | API only | ❌ | ❌ | Rating 1-5 ⭐ |
| **Boavizta** | ACV multi-critères | ✅ CC BY-SA | API only | ❌ | ❌ | Référence FR ACV |
| **Green Algorithms** | Green Algorithms | ✅ | Calculateur web | ❌ | ❌ | Académique HPC |
| **Climatiq.io** | Carbon factors DB | Mixte | ❌ | Limité | ✅ payant | API commerciale |
| **Sobr.ia** | **Multi-méthodos AFNOR + EcoLogits + (v1.1) HF + ML.ENERGY** | ✅ MIT | ✅ binaire | ✅ self-hosted | 📋 v1.3 opt-in | **Tiers de confiance souverain qui agrège vendors disclosure (Mistral × ADEME, Google, Meta)** |

**Pitch unique défensable** : *« Le tiers de confiance souverain qui agrège, normalise et présente les disclosures vendor (Mistral × ADEME, Google Gemini, Meta Llama) — et qui marche en 100 % local avec audit chaîné scientifique. »*

C'est cette phrase qu'il faut placarder partout : README, landing page, pitch candidature.

## Value proposition en 1 phrase (à valider)

> **Sobr.ia mesure l'empreinte de vos prompts IA en local, agrège les chiffres officiels des fabricants (Mistral × ADEME, Google, Meta) et vous donne un journal scientifique reproductible — pour particulier, équipe ou administration, sans cloud Sobr.ia.**

29 mots. Tout y est : mesure, local, agrégation vendors, scientifique, multi-persona, anti-cloud.

---

# 🧶 Audit narratif "le produit raconte son histoire"

Si je ré-installe l'app fresh aujourd'hui sans contexte :

1. **Splash 3s** → "Sobr.ia" — joli mais ne dit rien.
2. **Persona picker** → 5 cartes. Je clique sur "Pro Tech" parce que je suis dev.
3. **Bundle** → 7 modules pré-cochés sans expliquer pourquoi ces 7. Je clique "Terminer".
4. **Premier prompt** → l'atelier M1 s'ouvre. Tooltip lime sur le sélecteur de modèle.
5. **J'estime un prompt sur GPT-4o, 100 tokens in, 400 tokens out** → résultat affiché : "1,2 gCO₂eq P50 (P5-P95 : 0,9-1,5)" + "Voir aussi" EcoLogits 1,1 g.
6. **OK et après ?** → je ne sais pas quoi faire. Pas de suggestion *« et maintenant explorez le Dashboard pour voir votre usage cumulé »* ou *« comparez avec d'autres modèles »*.

**Le produit n'a pas de fil narratif post-premier-prompt.** Chaque module est une île. Il manque un **chemin guidé suggéré** dans l'app (style "Suivez le tour Sobr.ia en 5 min").

**Reco** : ajouter une **bannière "Et après ?"** sous le résultat M1 avec 2-3 suggestions contextuelles :
- "Comparer ce modèle à d'autres → M3"
- "Voir votre usage cumulé → M15"
- "Fixer un budget mensuel → M25"

---

# 🔥 Top 10 findings consolidés (du plus critique au moins critique)

| # | Finding | Persona impacté | Effort fix |
|---|---|---|---|
| 1 | **README et taglines dev-centriques, jargon en première page** | Tous, surtout Student | 1 j refonte messaging |
| 2 | **Onboarding demande "qui es-tu ?" avant de dire "qu'est-ce que c'est ?"** | Tous, surtout Student | 0.5 j ajouter étape "Sobr.ia en 30s" |
| 3 | **Vendors disclosure (Mistral × ADEME, Google, Meta) pas dans le produit** | Pro Tech, Researcher, Entreprise | 1.5 j (C31.1 ramené pré-v1.0) |
| 4 | **Module IDs (M1, M3...) fuient dans l'UI** | Tous | 0.5 j cleanup labels |
| 5 | **M14 "À propos" dans les bundles personas** | Tous | 0.1 j retirer M14 des bundles |
| 6 | **Mode Équipe install non guidé côté UI Tauri** | Enterprise | 1 j panneau "Activer Mode Équipe" |
| 7 | **Pas d'équivalences "humaines" (douches, km voiture, kWh frigo)** | Student surtout | 0.5 j affichage équivalences M1 + M15 |
| 8 | **Pas de fil narratif post-premier-prompt** | Student, Pro Tech | 0.5 j bannière "Et après ?" |
| 9 | **DOI manquant pour citation académique** | Researcher | 0.5 j dépôt Zenodo + DOI dans README |
| 10 | **Labels modules opaques (M9 "Référentiel", M17 "Empreinte projet")** | Student, Public Sector | 0.2 j renaming |

**Effort total** : ~7-8 jours si on fait tout. **Mais on peut faire 80 % de la valeur avec les findings 1, 2, 3, 4, 7, 10 = ~4 jours.**

---

# 🚀 Skeleton chantier C32 — exécution clarté produit

Recommandation : **3 sprints courts** plutôt qu'un gros, pour pouvoir valider chaque étape.

## C32.1 — Messaging + labels + nettoyage (1 jour)

- Réécriture README "Sobr.ia, c'est quoi ?" + sections "Pour qui ?" (5 cartes persona) + value proposition 1 phrase.
- Renaming labels modules : "M9 Référentiel" → "Bibliothèque modèles", "M17 Empreinte projet" → "Datasheet scientifique".
- Cleanup : retirer "M1", "M3"... des labels UI (garder en URL et code).
- Retirer M14 "À propos" des bundles personas (mais accessible via menu).
- 1 commit : `refactor(ui): C32.1 messaging + labels clarifiés`.

## C32.2 — Onboarding + fil narratif (1 jour)

- Insérer entre splash et persona picker : étape "Sobr.ia en 30 secondes" (≤ 4 phrases + 1 schéma équivalence carbone d'1 prompt).
- Ajouter bannière "Et après ?" sous le résultat M1 (3 suggestions contextuelles).
- Ajouter tooltip "Pourquoi ces modules ?" sur la liste bundle dans onboarding et /parametres.
- 1 commit : `feat(ui): C32.2 onboarding pédagogique + fil narratif post-prompt`.

## C32.3 — Équivalences humaines + Mode Équipe guidé (1.5 jour)

- Module M1 + M15 : affichage équivalence "= X douches / Y km voiture / Z kWh frigo".
- Module M25 Eco-budget : équivalences également.
- Panneau "Activer Mode Équipe" dans /parametres avec script 1 clic d'init.
- Quickstart "5 minutes — déployer Sobr.ia dans mon entreprise" dans `docs/operations/team-aggregator.md` enrichi + lien dans /parametres.
- 1 commit : `feat(ui,team): C32.3 équivalences humaines + onboarding Mode Équipe guidé`.

## C32.4 — Vendors disclosure (C31.1 ramené pré-v1.0) (1.5 jour)

- Ramener C31.1 du brief C31 : Mistral × ADEME + Google Gemini + Meta Llama presets enrichis avec chiffres officiels.
- Encadré "Données vendor disclosure (vérifiées ADEME pour Mistral)" dans M9 fiche modèle.
- Table comparaison vendor disclosure dans M9.
- 1 commit : `feat(estimator,ui): C32.4 vendors disclosure (Mistral × ADEME, Google Gemini, Meta Llama)`.

## C32.5 — Polish + DOI + ship v0.8.0 (0.5 jour)

- DOI Zenodo + ajout dans README.
- Smoke test E2E des 5 onboardings persona.
- CHANGELOG `[0.8.0] — Clarté produit (C32)`.
- Bump versions 0.7.1 → 0.8.0 partout.
- Tag `v0.8.0`.

**Total C32 : ~4-5 jours** pour livrer une **v0.8.0 Clarté produit** avant d'attaquer v1.0 candidature.

---

# 🎁 Bonus : la nouvelle promesse Sobr.ia

Si tu veux une formule à tester avec des testeurs externes après C32 :

> *« Sobr.ia est l'app qui te montre ce que coûte chaque prompt IA — en grammes de CO₂, en gouttes d'eau, en watts-heures, en kilomètres de voiture si tu préfères.*
>
> *Pour les particuliers curieux : 100 % local, sans compte, sans inscription. Tu vois ton usage, tu fixes des objectifs, tu apprends.*
>
> *Pour les pros et les entreprises : audit scientifique reproductible, rapport CSRD réglementaire, mode équipe self-hosted. Vos données restent chez vous.*
>
> *Sobr.ia est aussi le seul outil qui agrège les chiffres officiels publiés par les fabricants (Mistral × ADEME, Google Gemini, Meta Llama) — pour comparer ce qu'ils disent à ce que d'autres méthodologies estiment. Le tiers de confiance souverain de l'empreinte IA. »*

À condenser en 30 sec côté splash, à utiliser tel quel côté README, à décliner par persona dans l'onboarding.
