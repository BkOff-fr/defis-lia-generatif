# ADR-0013 — Extension navigateur + pairing local + mode Équipe self-hosted

- **Statut** : Phase 1 Implemented (v0.6.0, 2026-05-16) ; Phase 2 Implemented (v0.7.0, 2026-05-16, C28) ; Phase 3 Backlog (v0.8+)
- **Date** : 2026-05-15
- **Décideurs** : Thibault, Cowork
- **Contexte** : avant chantier C27 (v0.6.0 extension) et C28 (v0.7.0 mode équipe)

## Contexte et énoncé du problème

Sobr.ia v0.5.0 ship une app Tauri et un pipeline médaillon de référentiel. Le CDC §M6 prévoit une extension navigateur pour mesurer l'empreinte des prompts en direct sur ChatGPT, Claude, Le Chat. Deux cas d'usage à servir :

1. **Particulier** : l'utilisateur installe l'app Tauri + l'extension sur ses navigateurs. Il veut que les estimations de l'extension remontent dans le Dashboard/Journal de son app personnelle sans configuration complexe.
2. **Entreprise** : une équipe (5–500 personnes) veut centraliser les usages IA de ses collaborateurs pour reporting CSRD, audit FinOps, gouvernance. Chaque employé garde la visibilité de son propre usage. L'admin agrège.

Le CLAUDE.md §7 (privacy by design) interdit :
- l'envoi de prompts utilisateurs vers un serveur externe ;
- tout tracking sans opt-in explicite ;
- la création d'un cloud Sobr.ia centralisé.

Le pitch défi data.gouv.fr repose sur la souveraineté et la frugalité — un SaaS centralisé serait incohérent.

## Décision

**Architecture hybride à deux étages**, opt-in par étage, sans cloud Sobr.ia.

### Étage 1 — Particulier (v0.6.0, C27)

L'extension et l'app personnelle se **pairent localement par code à 6 chiffres**, via le mécanisme **native messaging** standard des WebExtensions (sécurité OS, pas de port ouvert).

Flux :

1. L'app Tauri, au premier lancement après v0.6.0, génère un code à 6 chiffres (entropie 20 bits, suffisant pour usage local éphémère) et propose à l'utilisateur d'installer le bridge natif. L'app écrit alors les fichiers manifest natifs aux bons emplacements OS (Chrome, Firefox, Edge si présents) — avec consentement explicite.
2. L'utilisateur installe l'extension Chrome/Firefox.
3. Au premier ouvrir-popup, l'extension détecte le bridge natif via `chrome.runtime.connectNative`. Si OK, elle demande à l'utilisateur de saisir le code 6 chiffres affiché dans son app. Si le code est correct, le pairing est établi et un secret partagé (32 bytes random, stocké dans `chrome.storage.local` côté extension et SQLite côté app) remplace le code temporaire.
4. À chaque estimation extension, le secret est envoyé avec la charge dans la même session native messaging. L'app vérifie le secret avant d'ingérer.
5. Le secret peut être révoqué côté app (bouton "Dépaire l'extension" dans `/parametres`).

Les estimations ingérées apparaissent dans Dashboard M15 et Journal avec un tag `source = 'extension'`. Filtres "Toutes / App / Extension" disponibles.

**Garanties** :
- Aucun trafic réseau (tout est native messaging local OS).
- L'utilisateur peut désinstaller l'extension ou révoquer le pairing à tout moment.
- Pas de compte, pas d'inscription, pas de cloud.

### Étage 2 — Équipe self-hosted (v0.7.0, C28)

Pour le cas entreprise, un nouveau binaire **`sobria-team-aggregator`** (crate Rust, ~10 MB, équivalent serveur HTTPS local). L'entreprise déploie ce binaire sur SON infrastructure (poste admin, serveur LAN, VPS interne). **Aucun service Sobr.ia n'est impliqué.**

Flux :

1. L'admin lance `sobria-team-aggregator init` qui :
   - Génère une paire TLS auto-signée (ou import certif fourni).
   - Crée une base SQLite locale `team.sqlite`.
   - Affiche l'URL d'accès (ex: `https://192.168.1.42:8443`) et un mot de passe admin initial.
2. L'admin accède au dashboard web (Svelte servi par le binaire), crée N "enrollment codes" (jetons à 12 chiffres, valides 7 jours).
3. Chaque employé reçoit son enrollment code (mail, chat interne).
4. Dans son extension OU dans son app Sobr.ia : section "Mode Équipe" → saisit l'URL serveur (`https://192.168.1.42:8443`) + son enrollment code.
5. L'extension/app obtient un token JWT court (24 h, refresh auto) signé par le serveur. Toutes les estimations sont POST en JSON via HTTPS.
6. Le serveur agrège, affiche le dashboard admin (qui a fait combien d'estimations, modèles populaires, tendances CSRD).
7. Chaque employé peut consulter SON usage personnel via le dashboard (auth par token JWT).

**Garanties** :
- Le serveur est CHEZ l'entreprise, déployé par eux, contrôlé par eux.
- Sobr.ia n'a aucun accès aux données.
- Compatible télétravail (VPN ou exposition HTTPS publique gérée par l'entreprise).
- L'employé peut continuer à utiliser son app perso en local (les deux flux coexistent : pairing perso ET enrollment équipe).
- Code source du serveur 100 % open source (même crate Rust, build reproductible).

### Ce qu'on ne fera PAS

- Pas de cloud Sobr.ia centralisé (SaaS).
- Pas de comptes utilisateurs hébergés par nous.
- Pas de télémétrie vers Sobr.ia depuis le serveur entreprise.
- Pas de pairing par OAuth/SSO d'entreprise en v1 — JWT simple suffit. SSO en v0.8+.
- Pas de mode "synchro multi-postes du même utilisateur" en v1 — l'utilisateur s'attache à UN device. Multi-device en v0.8+.

## Alternatives rejetées

### Alt 1 — 100 % local sans mode équipe

Respecte le CLAUDE.md mais ne couvre pas le cas entreprise. Or le défi data.gouv.fr met en avant les cas d'usage CSRD/AGEC qui sont par essence collectifs. Rejeté.

### Alt 2 — SaaS Sobr.ia (cloud centralisé)

Conflit frontal avec le CLAUDE.md §7 et le pitch défi data.gouv.fr. Coûts infra + GDPR + sécurité à porter. Rejeté.

### Alt 3 — mDNS / découverte LAN uniquement

Trop limitant (pas de télétravail), trop fragile (mDNS bloqué dans beaucoup de réseaux d'entreprise). Rejeté.

### Alt 4 — Native messaging avec auto-pairing (sans code)

Plus simple mais aucune protection contre une extension malveillante installée sur le même OS qui se ferait passer pour Sobr.ia. Le code de pairing est un anti-spoofing utile. Retenu.

## Conséquences

**Positives** :
- Le particulier a une expérience fluide (1 install + 1 code à saisir).
- L'entreprise a un mode souverain sans cloud Sobr.ia.
- Notre pitch défi data.gouv.fr reste cohérent (frugalité, souveraineté).
- Le secret partagé chiffré (Argon2 côté app SQLite) rend le pairing révocable.
- Le serveur entreprise est code open source — auditable, déployable, modifiable.

**Négatives** :
- Le binaire `sobria-team-aggregator` ajoute une crate Rust à maintenir (HTTPS, JWT, dashboard Svelte intégré).
- Le code 6 chiffres est court → on doit le rendre éphémère (TTL 5 min, régénération automatique) pour limiter le brute force, même si c'est local.
- Le mode équipe ajoute un protocole HTTPS — il faut documenter sécurité TLS, rotation certificats, etc.
- L'admin doit auto-héberger : ce n'est pas une expérience "1 clic".

**Neutres** :
- v0.6.0 ship perso seul. v0.7.0 ajoute équipe. Pas d'effet sur l'app actuelle.

## Implémentation phasée

| Phase | Version | Chantier | Périmètre |
|-------|---------|----------|-----------|
| Phase 1 | v0.6.0 | C27 | Extension + pairing perso 6 chiffres + bridge natif + ingestion Journal/Dashboard |
| Phase 2 | v0.7.0 | C28 | `sobria-team-aggregator` (binaire HTTPS standalone) + JWT 24h/refresh 7j + Argon2id partout + dashboard Svelte embedded (4 cards + 3 charts SVG) + exports CSRD PDF + PROV-O JSON-LD + CSV + mode Équipe extension/app (URL + ping + enroll + dispatch local/team/both + logout). Voir `briefs/chantiers/C28-mode-equipe-self-hosted.md` et `docs/operations/team-aggregator.md`. |
| Phase 3 | v0.8.0 | C29 (TBD) | SSO entreprise (SAML/OIDC), multi-device, RBAC fin |

## Liens

- CLAUDE.md §7 (privacy by design), §13 (anti-patterns).
- CDC §M6 (Extension navigateur).
- ADR-0012 (multi-méthodologie) — l'extension respecte le catalogue méthodo.
- Brief C27 `briefs/chantiers/C27-extension-navigateur.md`.
- Brief C28 (à rédiger pour v0.7.0).

## Validation

- Cohérence CLAUDE.md §7 ✓ (pas de cloud central, opt-in explicite à chaque étage).
- Cohérence pitch data.gouv.fr ✓ (souveraineté, code ouvert, self-hosted).
- Couverture cas particulier ✓ (pairing simple).
- Couverture cas entreprise ✓ (self-hosted aggregator).
- Pas de blocage roadmap v1.0 (extension est un nice-to-have, mode équipe est différentiateur).
