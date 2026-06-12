# ADR-0017 — Contrat démo web (fixtures du moteur réel hors Tauri)

- **Statut** : Accepted (2026-06-12) — implémenté en C37, documenté a
  posteriori (dette relevée au brief C37 §6.1)
- **Décideurs** : Thibault, Cowork
- **Contexte** : avant C37, le frontend appliquait un « contrat no-mock »
  strict (CLAUDE.md §13 : pas de données factices) — hors contexte Tauri,
  chaque page rejetait `tauri_unavailable` et n'affichait RIEN.

## Contexte et problème

Le CDC impose une démo web (plateforme 2ᵉ classe, **bloquante v1.0**) :
c'est ce que le jury data.gouv.fr et tout visiteur verront en premier.
Or le contrat no-mock produisait un site déployé fait de coquilles
vides avec des messages développeur (`cargo run -p sobria-app`). Deux
exigences légitimes se contredisaient : *ne jamais montrer de données
inventées* et *montrer quelque chose*.

## Décision

Le contrat no-mock devient un **contrat démo**, aux règles suivantes :

1. **Les fixtures sortent du moteur réel.** Aucune valeur n'est inventée
   côté frontend : `tools/fixturegen/` exécute `sobria-estimator`
   (seed 42, N = 10 000, horodatage figé) et sérialise les résultats
   complets (intervalles, bins, équivalents sourcés, hypothèses). Le
   catalogue (34 modèles), les méthodologies et les 28 datacenters sont
   les données embarquées réelles. Régénération documentée
   (`briefs/chantiers/C37-mode-demo-web.md` §2).
2. **Jamais dans l'application de bureau.** Activation uniquement quand
   `isTauriContext()` est faux ; le module démo et ses fixtures sont
   chargés par `import()` paresseux — le bundle Tauri ne les contient
   pas.
3. **Transparence permanente.** Bannière « Mode démo » sur toutes les
   pages, suffixe `· DÉMO` dans le rail, hypothèse `mode_demo` injectée
   dans chaque résultat (l'utilisateur qui inspecte les hypothèses voit
   l'origine), requête écho = point de grille réellement calculé (on
   n'affiche jamais les tokens de l'utilisateur avec un résultat qui ne
   leur correspond pas).
4. **Couverture partielle assumée.** Les commandes non couvertes
   rejettent `tauri_unavailable` avec un message utilisateur final
   (« Application de bureau requise ») ; les actions desktop-only sont
   désactivées avec explication (C42). La démo n'imite jamais ce qu'elle
   ne sait pas faire (ledger, écritures disque, pairing).
5. **La suite e2e teste CE contrat** (`web/playwright.config.ts`) :
   pages couvertes → contenu + bannière démo ; non couvertes → bandeau
   explicite sans jargon développeur.

## Conséquences

- (+) La démo web montre le vrai comportement du produit (vrais
  intervalles, vraies distributions) sans serveur ni clé API.
- (+) CLAUDE.md §13 est respecté dans son intention : zéro donnée
  inventée, zéro mock silencieux — c'est un mode explicite, étiqueté,
  reproductible (seed).
- (−) Les fixtures doivent être régénérées quand le registry des
  modèles change (sinon dérive démo/produit) — coût accepté, commande
  unique.
- (−) ~500 Ko de JSON minifié chargés paresseusement par la démo web
  uniquement.

## Options rejetées

- **Coquille vide (statu quo)** : site déployé sans fonctionnalité —
  inacceptable pour un livrable bloquant.
- **Calcul client-side simplifié** : aurait créé une méthodologie
  parallèle (interdite par CLAUDE.md §3) et des chiffres divergents.
- **Backend de démo hébergé** : contredit « pas de cloud Sobr.ia »
  (ADR-0013) et ajoute de l'infra pour un besoin statique.
