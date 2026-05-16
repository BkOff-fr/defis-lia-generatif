# sobria-web-team

Dashboard du **Mode Équipe** Sobr.ia (C28.4) — SvelteKit + adapter-static,
servi par le binaire `sobria-team-aggregator` via `rust-embed`. Aucun cloud
Sobr.ia : votre navigateur parle uniquement au serveur que votre entreprise
a déployé.

## Pages

- `/login` — admin / employé / s'enrôler avec un code 12 chiffres.
- `/admin/dashboard` — analytics agrégés équipe (séries, top modèles, top
  employés, breakdown AFNOR/EcoLogits) + cards 4 KPI.
- `/admin/codes` — création/révocation d'enrollment codes.
- `/admin/users` — liste employés avec totaux.
- `/user/dashboard` — mon usage perso + ce qui est partagé avec l'équipe.

## Dev

```bash
# 1. Lance le binaire Rust en HTTPS local (auto-signé)
cargo run -p sobria-team-aggregator -- --data-dir ./team-data init \
    --admin-username admin --admin-password 'change-me'
cargo run -p sobria-team-aggregator -- --data-dir ./team-data serve

# 2. Lance le dev server Svelte sur :5174 (proxy /api → :8443)
cd web-team && npm install && npm run dev
```

## Production (embedded)

```bash
cd web-team && npm ci && npm run build
# → web-team/build/index.html + assets
cd .. && cargo build -p sobria-team-aggregator --release
# Le binaire embarque web-team/build/ via rust-embed et le sert à /
```

## Stack

- SvelteKit 2 + adapter-static + Svelte 5 (runes)
- TypeScript strict
- Pas de Plot/D3 : charts SVG manuels dans `src/lib/charts/`
  (~3 KB chacun gzip) — frugalité (CLAUDE.md §8).
- Pas de woff2 embarqué : system fonts uniquement.
- Auth client : JWT access en mémoire, refresh en sessionStorage,
  rotation automatique sur 401.

Voir `briefs/chantiers/C28-mode-equipe-self-hosted.md` §C28.4.
