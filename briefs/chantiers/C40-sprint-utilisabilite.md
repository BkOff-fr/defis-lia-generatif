# C40 — Sprint utilisabilité : première heure, boucle Réduire, kits UAT & déploiement

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.
> **Origine** : « comment rendre la solution mieux utilisable ? » — diagnostic :
> le moment magique (mesure automatique) était caché, les chiffres ne
> menaient à aucune action, et l'installation (desktop + équipe) frottait.

## 1. Parcours « première heure »

- **Onboarding, étape Ready** : carte « Mesure automatique (recommandé) »
  avec génération du code de pairing extension inline
  (`regenerate_pairing_code`, TTL 5 min) — le pairing n'est plus enfoui
  dans Paramètres. Hors Tauri : erreur orientée utilisateur, non bloquante.
- **M15 vide** : à zéro mesure, bloc d'accueil avec 2 CTA (« Estimer un
  premier prompt », « Associer l'extension ») au lieu d'un « Aucune
  requête » sec. + un tutoiement résiduel corrigé.

## 2. Boucle « Réduire » (web/src/lib/components/ReduceSuggestions.svelte)

Sous chaque résultat d'estimation : jusqu'à 3 alternatives plus sobres
(params actifs inférieurs ; éventail plus-proche / médian / plus-sobre),
**réestimées par le moteur** via `estimate_for_comparison` (mêmes tokens,
même méthodologie, éphémère) — deltas % affichés, liens Comparer /
Éco-budget. Aucune heuristique d'empreinte côté client (CLAUDE.md §13).
Si le modèle courant est déjà en bas du catalogue : message « déjà l'un
des plus sobres ». Capture : Claude Opus 4.8 → Phi-4 −99 %, Mistral
Medium −93 %, Claude Opus 4 −25 %.

## 3. Kits livrés par agents (vérifiés)

- **UAT (docs/qa/uat/)** : README protocole + consentement RGPD,
  script de session minute-par-minute (SUS 10 items), 27 tâches
  chronométrées sur les 5 personas, grille d'observation, template de
  synthèse — aligné sur le brief C36 (3 phases, DoD SUS ≥ 70).
- **Déploiement équipe (deploy/team/ + docs/operations/)** : Dockerfile
  multi-stage (web-team SvelteKit → rust-embed → runtime non-root),
  compose + entrypoint (init auto au 1er démarrage), guide TLS
  (auto-signé / Caddy), modèles d'emails CSE + salariés.
  **⚠ Images jamais buildées (pas de Docker ici) — à tester en premier.**

## 4. Tests

- e2e : estimate.spec étendu (boucle Réduire **+ rail C39 : essentiels
  visibles, /methodo derrière « Plus », clic toggle** — comble C39 §6.1) ;
  estimate ✓, onboarding ✓ (dont « Terminer » qui traverse la carte
  pairing), m15 ✓. svelte-check 0 erreur, prettier ✓, build ✓.
- Sync repo par checksums : 0 divergence (politique C39 §4).

## 5. Restes à faire

1. **Builder l'image Docker** sur un poste avec Docker (§3) + CI build.
2. Modèle par défaut du Composer = GPT-4o mini (deprecated) → choisir un
   défaut courant (ex. claude-haiku-4-5) ; la boucle Réduire y montrera
   en plus des alternatives réelles dès le premier essai.
3. Pré-remplissage de /comparer depuis la boucle Réduire (query params).
4. Dérouler l'UAT (docs/qa/uat/) — tout est prêt, il manque 5 humains.
5. Tooltips lexique (P5/P95, prefill…) reliés au glossaire M8 ; i18n EN
   post-candidature.
