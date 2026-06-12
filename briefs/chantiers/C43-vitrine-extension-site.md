# C43 — Vitrine : extension simplifiée + refonte du site de présentation

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.
> **Origine** : « simplifier l'extension, vendre la solution entière sur le
> site comme une présentation, plus beau, épuré, pratique » (Thibault).

## 1. Extension (agent, vérifié — 84/84 tests verts)

- **Popup refondu en 3 niveaux** : LE chiffre du jour (gCO₂eq, serif 52px,
  équivalent voiture ADEME), état d'association en une ligne avec action
  unique, une rangée d'actions. Détails (dernier prompt, eau/énergie,
  méthodo) repliés dans « Détails du jour ».
- États vides soignés, vouvoiement, jargon supprimé (« SPEC 2314 · FR » →
  « Référentiel français »), contrastes AA (#72706a → #b8b4ac), ≥ 12px.
- Indicateurs in-page : retouche minimale (tailles 9-11→12px, anneau
  pulsant et « · LIVE » supprimés, « P5 x ─ P95 y · IC 90 % » →
  « fourchette x – y g (confiance 90 %) ») — logique d'injection intacte.
- +19 tests popup ; build 5 cibles vite vert ; tsc vert ; eslint 0 erreur.
- Captures : ext_popup_avant/apres/apres_vide.png (outputs).

## 2. Site (site/, Astro — page d'accueil scrollytelling)

Le site C33 était déjà fort (6 chapitres, WebGL, calcul live) mais vendait
un produit périmé. Corrigé :

- **Incohérence majeure** : le chapitre Équipe affichait un **classement
  nominatif** (Claire B. 468 g…) — l'inverse de l'ADR-0015 livré en C38.
  Remplacé par le vrai produit : Alice V. « partage activé » + ligne
  agrégée « 6 participants anonymes » + note de seuil k. Le copy vend
  « k-anonyme par construction » + « Docker en 30 minutes, kit CSE inclus ».
- **Nouveau chapitre 06/07 « Mesurer, puis réduire »** : la boucle Réduire
  (C40) avec les vrais deltas du moteur (−99 % / −93 % / −25 %, seed 42)
  — « Un thermomètre ne suffit pas. Sobr.ia est un levier. »
- **Vouvoiement intégral** (hero, 7 chapitres, title/og/twitter, i18n map)
  — aligné sur l'app (C37+).
- **Chiffres défendables** : « 186 datacenters cartographiés » (somme
  marketing) → « 28 datacenters de référence documentés » + régions cloud
  indexées ; v0.8.0 → v0.9.0 (CTA + footer).
- **CTA final** : « Voir les plans cloud » (offre inexistante en tête) →
  3 boutons : Installer l'extension · Télécharger l'app · Mode Équipe
  self-hosted ; footer « Cloud / SaaS » → « Mode Équipe ».
- Rail de progression : 8 points relabellisés (06 · Réduire, 07 · Commencer).

## 3. Vérifications

- Extension : suite vitest 84/84, build, tsc, eslint (rapport agent).
- Site : `astro build` ✓ (45 pages, 14 s) ; prettier ✓ sur index.astro ;
  rendu vérifié par captures pleine page (rail 8 points, zéro erreur JS) ;
  `grep` tutoiement : 0 ; sync checksums : 0 divergence.
- **Préexistant, non traité** : erreurs `astro check` ts(7016) dans les
  composants legacy `src/components/sections/*` (non utilisés par
  l'accueil, build OK) ; tests site (a11y/lighthouse Playwright) non
  lancés ici — à passer en CI.

## 4. Restes

1. Lancer `npm test` du site (a11y + lighthouse) en CI ou local Windows.
2. /telecharger : ajouter une ancre/section app desktop dédiée (les deux
   CTA pointent vers la même page).
3. Page /cloud : relire le framing (self-hosted réel d'abord, cloud
   ADR-0014 « à venir » ensuite).
4. OG image : régénérer avec le nouveau wording vouvoyé si elle contient
   l'ancien titre.
5. Composants legacy sections/* : typer ou supprimer (dette).
