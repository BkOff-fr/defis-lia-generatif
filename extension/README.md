# Sobr.ia — Extension navigateur (WebExtension MV3)

Extension WebExtension MV3 qui mesure l'empreinte environnementale des prompts
envoyés en direct sur **ChatGPT** (chat.openai.com), **Claude** (claude.ai) et
**Le Chat / Mistral** (chat.mistral.ai).

100 % local : moteur AFNOR + EcoLogits porté en JS, aucun envoi vers un serveur
externe. Peut s'apparier facultativement à l'app Sobr.ia desktop via _native
messaging_ + un code à 6 chiffres pour faire remonter les estimations dans le
Journal et le Dashboard de l'app.

> **Périmètre v0.6.0 (C27 Phase 1 — ADR-0013)** : extension + pairing personnel.
> Le mode Équipe self-hosted (`sobria-team-aggregator`) est différé à v0.7.0 (C28).

---

## Quickstart

### Pré-requis

- Node ≥ 22, npm ≥ 10.
- Pour la chaîne `native messaging` (optionnelle) : app Sobr.ia desktop ≥ 0.6.0
  installée, et le binaire `sobria-bridge` (livré avec l'app).

### Installation des dépendances

```bash
cd extension
npm install
```

### Vérifications rapides

```bash
npm run check         # type-check TypeScript strict
npm run lint          # prettier --check + eslint
npm run test          # vitest run (unit tests)
```

### Build de production

```bash
npm run build              # → dist/ (Chrome / Edge / Brave)
npm run build:firefox      # → dist-firefox/ (Firefox)
npm run package            # → dist/sobria-extension-{chrome,firefox}-v0.6.0.{zip,xpi}
```

Le script de build enchaîne 2 passes Vite (modules ES pour popup/options/SW,
IIFE pour les content scripts — contrainte MV3). `npm run package` produit des
ZIP/XPI déterministes (timestamps figés, deflate level 9) et imprime le SHA-256
de chaque archive.

### Mode dev (HMR popup/options)

```bash
npm run dev
```

> **Limite v0.6.0** : Vite watch n'observe que la passe `main`. Les content
> scripts ne se rechargent pas automatiquement — relancer `npm run build` pour
> les retester.

### Synchronisation des logos (mutualisation)

```bash
npm run sync-logos
```

Les logos sont **mutualisés** depuis le design system Sobr.ia et le frontend
web. Source de vérité :

| Source                                               | Destination                     | Rôle                             |
| ---------------------------------------------------- | ------------------------------- | -------------------------------- |
| `sobr-ia-design-system/project/assets/logo-mark.svg` | `src/assets/icons/mark.svg`     | Mark complet (gradients lime)    |
| `sobr-ia-design-system/project/assets/logo.svg`      | `src/assets/icons/wordmark.svg` | Mark + wordmark horizontal       |
| `web/static/favicon.svg`                             | `src/assets/icons/favicon.svg`  | Mark simplifié (lisible à 16 px) |
| `web/static/apple-touch-icon.svg`                    | `src/assets/icons/tile.svg`     | Mark sur tuile ink (vue dense)   |

Le manifest MV3 référence les SVG directement (supporté Chrome 88+ et
Firefox — conforme nos minima v0.6.0). Aucune rastérisation PNG embarquée,
cohérent avec la frugalité du projet (CLAUDE.md §8).

---

## Installation en mode développeur

### Chrome / Edge / Brave

1. `npm run build`
2. Ouvre `chrome://extensions/`.
3. Active « Mode développeur ».
4. Clic « Charger l'extension non empaquetée » → sélectionne `extension/dist/`.
5. L'icône Sobr.ia (carré lime) apparaît dans la barre.

### Firefox

1. `npm run build:firefox`
2. Ouvre `about:debugging#/runtime/this-firefox`.
3. Clic « Charger un module complémentaire temporaire ».
4. Sélectionne n'importe quel fichier dans `extension/dist-firefox/`
   (ex : `manifest.json`).
5. L'icône apparaît dans la barre d'outils.

> Note : `about:debugging` ne persiste pas l'extension entre les sessions. Pour
> un test persistant, utiliser Firefox Developer Edition + `xpinstall.signatures.required=false`.

---

## Structure

```
extension/
├── manifest.json              # MV3 Chrome
├── manifest.firefox.json      # variant Firefox (Gecko id, background.scripts)
├── package.json
├── tsconfig.json              # strict, ES2022, types chrome + firefox + node
├── vite.config.ts             # multi-entry, 2 passes (main / iife)
├── eslint.config.js           # flat config (aligné web/eslint.config.js)
├── .prettierrc
├── scripts/
│   ├── build.js               # orchestre les 4 passes Vite (main + 3 IIFE)
│   ├── package.js             # zip/xpi déterministes + SHA-256
│   └── sync-logos.js          # sync depuis web/static + design system
├── src/
│   ├── assets/
│   │   ├── fonts/             # 8 WOFF2 self-host (SIL OFL)
│   │   └── icons/             # SVG mutualisés (favicon/mark/tile/wordmark)
│   ├── styles/
│   │   └── tokens.css         # design tokens partagés popup + options
│   ├── background/
│   │   └── service-worker.ts  # event-driven, stub C27.1
│   ├── content/
│   │   ├── chatgpt.ts         # chat.openai.com (stub C27.1)
│   │   ├── claude.ts          # claude.ai (stub C27.1)
│   │   └── le-chat.ts         # chat.mistral.ai (stub C27.1)
│   ├── popup/
│   │   ├── index.html
│   │   ├── main.ts
│   │   └── popup.css
│   └── options/
│       ├── index.html
│       ├── main.ts
│       └── options.css
└── tests/                     # vitest + playwright (arrivent en C27.2+)
```

---

## Privacy & permissions

| Permission        | Raison                                                                  |
| ----------------- | ----------------------------------------------------------------------- |
| `activeTab`       | Lire le contenu de la page active pour détecter le prompt soumis.       |
| `storage`         | Persister localement les totaux journaliers et préférences utilisateur. |
| `nativeMessaging` | (Opt-in C27.5) Communiquer avec le bridge desktop pour pairing.         |

`host_permissions` strictement limités à 3 origines : ChatGPT, Claude, Le Chat.

CSP `script-src 'self'; object-src 'self'` — pas de `unsafe-eval`, pas de
`unsafe-inline`, pas de remote code, pas de tracking.

**Aucun trafic réseau** vers Sobr.ia ou un tiers. Tout est local. Les
estimations sont calculées par un moteur AFNOR + EcoLogits embarqué en JS
(C27.2). Le pairing facultatif passe par _native messaging_ WebExtensions
(sécurité OS, pas de socket réseau).

Voir `docs/extension/privacy-policy.md` (rédigé en C27.6) pour la politique
détaillée.

---

## Liens

- Brief : [`briefs/chantiers/C27-extension-navigateur.md`](../briefs/chantiers/C27-extension-navigateur.md)
- ADR-0013 : [`docs/adr/ADR-0013-extension-pairing-team-mode.md`](../docs/adr/ADR-0013-extension-pairing-team-mode.md)
- ADR-0012 : [`docs/adr/ADR-0012-multi-methodology-engine.md`](../docs/adr/ADR-0012-multi-methodology-engine.md)
- CLAUDE.md : [`../CLAUDE.md`](../CLAUDE.md)
