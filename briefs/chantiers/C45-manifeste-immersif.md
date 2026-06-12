# C45 — Manifeste immersif : direction artistique + page /manifeste

> **Statut** : DA + implémentation v1 le 2026-06-12 (session Cowork) —
> **NON RENDUE À L'ÉCRAN** (environnement d'exécution indisponible) :
> l'itération visuelle reste à faire ensemble, captures à l'appui.
> **Références validées par Thibault** : hubtown.co.in (manifeste en
> scènes — un mot, une déclaration géante par écran, navigation au
> scroll) ; pacomepertant.com (jeu, marquees, rythme, entrée immersive).

## 1. Diagnostic

L'accueil C43 est un bon scrollytelling *documentaire* : il montre le
produit. Les pages C44 (/produit /equipe /methode) sont des pages de
contenu propres mais sages. Ce qui manque — et que les références ont —
c'est un lieu où le site **prend position** : une thèse, mise en scène,
une idée par écran, sans interface.

## 2. Concept : « Le poids invisible » — un manifeste en 7 scènes

Un mot-scène géant (Instrument Serif italique, ~20vw, en arrière-plan,
ton ivoire 4 %) + une déclaration en 2-3 lignes (display ~7vw) + une
ligne de corps + UN seul élément vivant par scène. Voix engagée,
vouvoiement, phrases courtes.

| # | Mot-scène | Déclaration (3 lignes) | Élément vivant |
|---|-----------|------------------------|----------------|
| 0 | INVISIBLE | « Chaque prompt / a un poids. / Personne ne le voit. » | curseur lumineux qui « pèse » ; marquee `le poids invisible •` |
| 1 | MESURER | « On ne réduit pas / ce qu'on ne / mesure pas. » | compteur 0 → 0,45 g CO₂eq (moteur, seed 42) |
| 2 | DOUTER | « Un chiffre sans / intervalle est un / mensonge confortable. » | barre P5–P50–P95 qui se déploie |
| 3 | CHOISIR | « Le même travail. / Cent fois / plus léger. » | bascule Opus 7,2 g → Phi-4 59,7 mg (−99 %) |
| 4 | ENSEMBLE | « Compter / sans / surveiller. » | rangée d'avatars qui se fondent en agrégat « k = 5 » |
| 5 | OUVRIR | « Pas de boîte noire. / Du code, des sources, / des intervalles. » | marquee `MIT • AFNOR SPEC 2314 • seed 42 • ComparIA • RTE •` |
| 6 | COMMENCER | « Votre premier / prompt mesuré. / 30 secondes. » | les 3 CTA, respiration lente du fond |

Tous les chiffres affichés sont les valeurs réelles du moteur (fixtures
C37/C40) — le manifeste reste factuel, c'est sa force.

## 3. Système d'animation (frugal, sans dépendance — CLAUDE.md §3/§8)

- **Scènes** : sections 100svh, `position: sticky` du contenu, le scroll
  fait défiler les scènes (pas de hijack du scroll : défilement natif).
- **Reveals** : IntersectionObserver ajoute `.on` → les lignes montent
  (`clip-path` + translateY, stagger 80 ms) ; mot-scène en parallaxe
  douce via `animation-timeline: view()` (CSS scroll-driven, fallback
  IO si non supporté).
- **Compteur / bascule / barre** : rAF court (≤ 1,2 s) déclenché à
  l'entrée de scène, une seule fois.
- **Marquees** : boucle CSS `translateX` (pattern Pacôme), `aria-hidden`,
  dupliquées pour la continuité.
- **Rail** : points de progression réutilisés de l'accueil (cohérence).
- **`prefers-reduced-motion`** : tout statique, contenu intégral — la
  page sans JS est déjà complète (progressive enhancement strict).
- **Budget perf** : zéro lib, zéro image, ~0 réseau ; uniquement type +
  CSS + ~150 lignes de JS vanilla.

## 4. Implémenté (v1, à itérer à l'écran)

- `site/src/pages/manifeste.astro` — page autonome (styles + script
  inline, pattern index.astro), 7 scènes, copy complète, système §3.
- Entrées : lien « Lire le manifeste → » dans le hero de l'accueil +
  colonne Produit du footer.
- Pas d'autre page modifiée : risque zéro sur l'existant.

## 5. Vérifications — état honnête

**Rien n'a été rendu à l'écran** (VM morte). Le code est en amélioration
progressive (sans JS : page complète et lisible), mais le réglage fin
(timings, chevauchements, tailles fluides, mobile) DOIT se faire devant
un navigateur. À lancer :

```bash
cd site && npx astro build && npm run preview   # puis /manifeste
npm run lint
```

Puis on itère ensemble sur captures (timings, vw clamps, scène 3-4).

## 6. Phase 2 (après itération visuelle)

1. Retrofit du système de scènes sur /produit /equipe /methode (chaque
   page s'ouvre sur UNE scène-manifeste avant le contenu).
2. Entrée sonore optionnelle ? (référence Pacôme — à débattre, sobriété
   oblige : plutôt non).
3. Curseur custom léger sur le manifeste seulement.
4. Tests a11y (axe) + Lighthouse sur la page.
