# Fonts — Sobr.ia

**Familles utilisées** :
- **Inter Variable** — UI (corps, titres, boutons, labels) — poids 400/500/600/700.
- **JetBrains Mono** — chiffres tabulaires, code, hashes audit — poids 400/500/600.

## Statut

⚠️ **Substitution flaggée** : ce projet charge les deux familles depuis **Google Fonts CDN** (voir `colors_and_type.css`). Le CDC v1.1 de Sobr.ia précise « *auto-hébergées* » côté app native (Tauri 2 + SvelteKit) — il faudra rapatrier les TTF/WOFF2 dans `web/static/fonts/` lors de l'implémentation S6.

Les deux familles sont publiées sous **SIL Open Font License 1.1** (compatible MIT/Etalab 2.0/CC-BY → aucun conflit avec les licences Sobr.ia).

## Téléchargement officiel

- Inter : https://github.com/rsms/inter/releases (variable WOFF2 recommandée)
- JetBrains Mono : https://github.com/JetBrains/JetBrainsMono/releases

## Pourquoi ces choix ?

- **Inter** : neutre, dessinée pour les UI denses, support tabular-nums, lecture parfaite à 13–14 px. Le CDC les nomme explicitement.
- **JetBrains Mono** : monospace lisible avec une grille tabulaire stable — essentiel pour aligner les colonnes de métriques (`2,14 g`, `4,87 Wh`) dans le Workbench, le Comparateur et le Journal d'audit.
