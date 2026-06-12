# C41 — Finitions utilisabilité : défaut moderne, comparateur pré-rempli, lexique, Paramètres réparés

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.
> **Origine** : restes C40 §5 + bug C37 §1 (Paramètres).

## 1. Modèle par défaut du Composer

`gpt-4o-mini` (deprecated, déjà sobre → boucle Réduire muette au premier
essai) → **`claude-haiku-4-5`** (courant, sobre, et la boucle Réduire
montre de vraies alternatives dès la première estimation). `?model=` URL
inchangé, fallback premier du catalogue.

## 2. Comparateur pré-rempli depuis la boucle « Réduire »

Le lien « Comparer en détail » devient
`/comparer?models=<courant,alternatives>&tin=…&tout=…` ; le comparateur
valide les ids contre le catalogue et pré-sélectionne (≥ 2 requis),
reprend les tokens. Complète les params `prompt/tokensOut/model`
préexistants sans conflit.

## 3. Lexique inline (`lib/lexique.ts` + `Term.svelte`)

10 définitions courtes (P50, P5–P95, token, tokens sortie, prefill,
decode, PUE, WUE, IF élec, Monte-Carlo), tooltip accessible
(hover + focus clavier, `role="tooltip"`, ancrage gauche anti-clipping),
lien « Glossaire complet → » vers `/methodo#glossaire`. Posé sur :
ResultBlock (médiane, pill P5–P95) et Composer (tokens entrée / sortie).
Extension aux autres écrans = mécanique (le composant est générique).

## 4. Paramètres : bootstrap scindé (fix C37 §1)

Le `Promise.all` groupé sacrifiait Runtime/Référentiel/Méthodologies dès
que le pairing (desktop-only) rejetait. Désormais : sections démo
chargées d'abord, pairing/équipe en best-effort séparé (`pairingError`
confiné à sa section). Dernier message `cargo run` résiduel remplacé.

## 5. Corrections d'outillage découvertes en route

- **`__APP_VERSION__` manquait des globals ESLint** (no-undef) — passé
  inaperçu depuis C37/C39 : les timeouts sandbox coupaient l'étape
  eslint après prettier. Déclaré `readonly` dans eslint.config.js.
  ⚠ Vérifier sous Windows : `npm run lint` complet doit passer.
- Spec `parametres` réécrit sur le comportement scindé (plus de bannière
  globale ; message confiné ; runtime démo visible).

## 6. Vérifications

svelte-check 0 erreur · eslint ✓ (complet) · prettier ✓ · build ✓ ·
specs : estimate ✓, comparer ✓, parametres ×2 ✓ (+ m15/onboarding C40) ·
captures tooltip + comparateur pré-rempli (8 éléments sélectionnés) ·
sync checksums : 0 divergence.

## 7. Restes (inchangés de C40 §5)

Docker build (poste avec Docker), UAT à dérouler, gating boutons morts
hors Tauri (journal/m17/m25), tooltips lexique sur les autres écrans,
i18n EN post-candidature.
