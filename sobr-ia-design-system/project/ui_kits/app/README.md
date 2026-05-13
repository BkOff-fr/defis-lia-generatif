# UI Kit — Sobr.ia Desktop

Recréation interactive de l'application desktop Sobr.ia (Tauri + Svelte), au style immersif aligné sur le design system v2.

## Fichiers

- **`Sobria-immersif.html`** — Écran *Estimer un prompt* (M2) en version éditoriale immersive (hero, composer, résultat, distribution Monte-Carlo, équivalents, hypothèses).
- **`Sobria-screens.html`** — Écrans *Workbench* (M3), *Comparer* (M5), *Simuler* (M4), *Journal d'audit* (M7), navigables via le rail latéral.

## Langage visuel

- Fond `--ink` (#0a0d0b), accent unique `--lime` (#c5f04a), texte `--ivory`.
- Display en **Instrument Serif italic**, UI en **Geist**, chiffres en **JetBrains Mono**.
- Hairlines `rgba(255,255,255,.07)`, glassmorphisme léger, glow lime sur les actions primaires.
- Ambient mesh + grain + motif topographique en arrière-plan.

## Status

Recréation **spéculative** — fidèle à la maquette ASCII v1.0 et au CDC v1.1, à valider quand le code Svelte applicatif existera. Les visualisations sont des SVG mockés, à remplacer par Observable Plot + D3 en production (cf. ADR-0008).
