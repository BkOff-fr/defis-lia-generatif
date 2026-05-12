# Fonts — Sobr.ia

Polices auto-hébergées pour respecter la CSP `default-src 'self'` de Tauri
(cf. CLAUDE.md §3 anti-patterns : pas d'appel réseau externe).

Toutes ces polices sont publiées sous **SIL Open Font License 1.1** —
redistribution autorisée, modification autorisée, vente interdite seule.
Le fichier de licence complet est dans `docs/LICENSES-FONTS.md` à
l'implémentation finale (TODO).

## Inventaire

| Fichier                                   | Famille          | Style                          | Provenance                                                  |
| ----------------------------------------- | ---------------- | ------------------------------ | ----------------------------------------------------------- |
| `geist-latin.woff2`                       | Geist            | 300–700 (variable)             | Vercel — https://vercel.com/font                            |
| `geist-latin-ext.woff2`                   | Geist            | 300–700 (variable, ext. latin) | idem                                                        |
| `instrument-serif-latin.woff2`            | Instrument Serif | 400 normal                     | Instrument — https://github.com/Instrument/instrument-serif |
| `instrument-serif-latin-ext.woff2`        | Instrument Serif | 400 normal (ext.)              | idem                                                        |
| `instrument-serif-italic-latin.woff2`     | Instrument Serif | 400 italic                     | idem                                                        |
| `instrument-serif-italic-latin-ext.woff2` | Instrument Serif | 400 italic (ext.)              | idem                                                        |
| `jetbrains-mono-latin.woff2`              | JetBrains Mono   | 400–600 (variable)             | JetBrains — https://github.com/JetBrains/JetBrainsMono      |
| `jetbrains-mono-latin-ext.woff2`          | JetBrains Mono   | 400–600 (variable, ext.)       | idem                                                        |

## Réhydratation

Les WOFF2 viennent de l'API Google Fonts (one-shot, hors production).
Pour les régénérer : voir `web/scripts/fetch-fonts.sh` (TODO en C09.D).
