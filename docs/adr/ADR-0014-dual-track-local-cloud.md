# ADR-0014 — Stratégie dual-track : local-first + cloud opt-in

- **Statut** : Accepted
- **Date** : 2026-05-16
- **Décideurs** : Thibault, Cowork
- **Contexte** : après v0.7.0 (Mode Équipe self-hosted shippé), avant la candidature data.gouv.fr v1.0
- **Liens** : CLAUDE.md §7 (privacy by design), §13 (anti-patterns), ADR-0013 (extension + pairing + mode équipe)

---

## Contexte et énoncé du problème

ADR-0013 a explicitement rejeté l'**Alt 2 — SaaS Sobr.ia centralisé** au motif d'un conflit avec :

- CLAUDE.md §7 *« privacy by design, tout traitement par défaut local, pas d'envoi externe »* ;
- le pitch défi data.gouv.fr (souveraineté, frugalité) ;
- les coûts infra continus, GDPR, sécurité.

Mais le retour terrain post-v0.7.0 révèle un besoin légitime non couvert :

1. **TPE/PME sans IT** veulent utiliser le Mode Équipe sans déployer ni opérer un serveur ;
2. **Freelances et indépendants** veulent une option managed avec mail SMTP géré, backup auto, monitoring ;
3. **Admins avancés** ont besoin de SSO, RBAC, gestion centralisée des seuils que le mode self-hosted basique ne couvre pas en v0.7.

La question n'est plus *« faut-il proposer du cloud ? »* mais *« comment proposer du cloud sans trahir le pitch souverain et le CLAUDE.md ? »*

---

## Décision

Adopter une **stratégie dual-track « local-first + cloud opt-in »**, inspirée des projets open-source matures (Bitwarden/Vaultwarden, Plausible, PostHog, Cal.com, Linkwarden).

### Principe

- **Le local reste le défaut absolu**. Particulier qui télécharge l'app : aucune inscription, aucun cloud, ça marche en 100 % local. Comportement actuel inchangé.
- **Le cloud Sobr.ia est une offre managed parmi d'autres**. C'est juste l'hébergement par nous du même binaire `sobria-team-aggregator` que tout le monde peut self-hoster gratuitement.
- **Le binaire est identique**. Pas d'« édition cloud premium » avec des features fermées. Pas de fork.
- **L'utilisateur choisit son hébergeur** : lui-même (self-hosted gratuit), Sobr.ia managed (payant), ou un autre prestataire managed (tiers).

### 5 conditions impératives

Toute offre cloud Sobr.ia doit respecter ces 5 conditions, faute de quoi le pitch défi data.gouv.fr et CLAUDE.md §7 sont trahis :

1. **Local par défaut** — l'app + l'extension fonctionnent en 100 % local sans inscription, sans compte, sans signal vers Sobr.ia. Le cloud est un choix conscient et explicite.
2. **Binaire open-source identique** — le serveur managé chez Sobr.ia est le même `sobria-team-aggregator` que tout self-hoster. Toute amélioration backend est merge sur la même branche, accessible à tous. Pas de privatisation de features.
3. **Pas d'envoi de prompts en clair** — le cloud Sobr.ia n'ingère que des metadata + résultats (tokens, gCO₂eq, méthodologie), jamais le contenu textuel des prompts. Strict respect du contrat d'extension/app actuel.
4. **GDPR-compliant** — hébergement UE (OVH Gravelines, Scaleway, Clever Cloud), DPO désigné, clauses contractuelles types, privacy policy publique, DPIA documentée, droit à l'effacement implémenté, export RGPD self-service.
5. **Free tier transparent + tarif lisible** — l'offre managed propose un free tier réel (ex : 1 000 estimations/mois, 5 utilisateurs) sans carte bancaire. Au-delà, tarif public en €/utilisateur/mois ou en pay-per-use, sans engagement annuel obligatoire.

Si une décision produit future viole une de ces 5 conditions, le ADR doit être amendé explicitement par décision de Thibault — pas de dérive silencieuse.

### Modèle technique

```
                            ┌─────────────────────────────────┐
                            │   Binaire sobria-team-aggregator│
                            │   (Rust, open source, MIT/Apache)│
                            └─────────────────────────────────┘
                                          │
                ┌─────────────────────────┼─────────────────────────┐
                ▼                         ▼                         ▼
        ┌──────────────┐          ┌──────────────┐          ┌──────────────┐
        │ Self-hosted  │          │   Managed    │          │  Tiers       │
        │  (gratuit)   │          │   Sobr.ia    │          │  managed     │
        │              │          │  (payant)    │          │  (libre)     │
        │ TPE/PME IT,  │          │ TPE/PME sans │          │ Hébergeurs   │
        │ DSI, souver. │          │ IT, indép.,  │          │ tiers, ESN   │
        │              │          │ freelances   │          │              │
        └──────────────┘          └──────────────┘          └──────────────┘
                │                         │                         │
                └─────────────────────────┴─────────────────────────┘
                                          │
                            ┌─────────────▼─────────────┐
                            │  Extension navigateur     │
                            │  + app Tauri personnelle  │
                            │  (parlent à n'importe     │
                            │  quel des 3 par URL)      │
                            └───────────────────────────┘
```

Côté client (extension + app), aucune notion de « Sobr.ia cloud » n'est privilégiée. L'URL serveur est saisie librement par l'utilisateur. La seule différence pour l'offre managed Sobr.ia : `https://team.sobr.ia` est dans la liste des serveurs suggérés en autocomplete, comme on pourrait suggérer `https://sobria.ovh.fr/xxx`.

### Différenciation features

Le **backend** est identique partout. La différenciation managed se fait sur l'**infrastructure** opérée par Sobr.ia :

| Capacité | Self-hosted | Managed Sobr.ia |
|---|---|---|
| Binaire open-source | ✓ | ✓ |
| Pairing perso 6 chiffres | ✓ | ✓ |
| Mode Équipe (enrollment codes, JWT, dashboard admin) | ✓ | ✓ |
| Exports CSRD/PROV-O/CSV | ✓ | ✓ |
| Alertes seuils (webhook, log) | ✓ | ✓ |
| Alertes mail SMTP | configuration manuelle | **SMTP géré inclus** |
| Backup automatique SQLite | à la charge de l'admin | **Backup chiffré quotidien S3 inclus** |
| Monitoring uptime + alertes ops | à la charge de l'admin | **Statuspage publique + PagerDuty inclus** |
| Update du référentiel modèles (Gold) | `dvc pull` manuel | **Update automatique sans action** |
| Support technique | communauté GitHub | **Support email/chat 24h** |
| SSO (SAML, OIDC) | code commun v0.8+ | **Pré-configuré avec providers populaires** |
| Multi-device | code commun v0.8+ | **Pré-configuré** |
| Sauvegarde géo-redondante | à la charge de l'admin | **3 zones UE inclus** |

L'offre managed apporte de l'**opex géré** et du **support**, pas du code propriétaire. Tout le code reste open-source et installable gratuitement.

---

## Alternatives rejetées (rappel)

### Alt 1 — Rester 100 % local-only, refuser le cloud

Cohérent avec ADR-0013 originel mais ne couvre pas le besoin TPE/PME sans IT. À terme, on perd ces utilisateurs au profit de SaaS concurrents qui n'ont aucun scrupule sur la privacy. Rejeté.

### Alt 2 — SaaS Sobr.ia centralisé propriétaire

Cloud as default, fork du backend en édition premium. Conflit frontal CLAUDE.md §7 + ADR-0013. Rejeté définitivement.

### Alt 3 — Cloud opt-in avec code open-source identique (CHOISI)

Le compromis qui préserve les principes et étend le marché. Choisi.

---

## Conséquences

**Positives** :
- Élargit la cible utilisateurs (particuliers ET TPE/PME sans IT ET DSI ET freelances).
- Préserve intégralement le pitch souverain : le local-first reste le défaut, le cloud est un choix conscient.
- Le binaire reste 100 % open-source, auditable, fork-able. La communauté self-hosting reste vivante.
- Création potentielle d'une source de revenus pérenne pour financer le développement open-source (modèle Plausible/Cal.com).
- Renforce le pitch défi data.gouv.fr : *« 100 % open-source, self-hostable ou managed, vos données souveraines dans tous les cas. »*

**Négatives** :
- Coûts opex managed (serveurs, support, monitoring, backup) à amortir.
- GDPR + DPO + DPA à constituer pour le cloud Sobr.ia.
- Sécurité opérationnelle (pentest annuel, bug bounty éventuel).
- Risque réputationnel si une faille touche le cloud Sobr.ia, même si le binaire est partagé.
- Maintenance double UX : il faut tester self-hosted ET managed à chaque release.

**Neutres** :
- Aucun impact sur l'app perso et l'extension actuelles. Le cloud est invisible tant qu'on ne l'active pas.

---

## Implémentation phasée

| Phase | Version | Chantier | Périmètre |
|---|---|---|---|
| **0 — Acter** | — | ADR-0014 (ce doc) | Décision figée par écrit |
| 1 — Polish | v0.7.1 | C29 (en cours) | UI Mode Équipe + admin reset-password + regen-cert + alertes seuils (cf. ADR-0013 Phase 2 enrichie) |
| 2 — Candidature | v1.0.0 | C-candidature | Sprint final data.gouv.fr (dossier + vidéo + UAT). **Pitch souverain pur**, le cloud n'est pas annoncé. |
| 3 — Doc self-hosting accessible | v1.1.0 | C-doc-quickstart | Quickstart « 5 minutes » non-IT pour le self-hosted (complément `docs/operations/team-aggregator.md`). |
| 4 — Admin avancée commune | v1.2.0 | C-admin-avance | SSO (SAML/OIDC), RBAC fin, multi-device — dans le binaire open-source commun. |
| 5 — Cloud beta managed | v1.3.0 | C-cloud-beta | Infrastructure managed (OVH/Scaleway), Stripe, landing page, privacy policy, DPIA, free tier. Beta sur invitation. |
| 6 — Cloud GA | v2.0.0 | C-cloud-ga | Public, marketing, support, statuspage, bug bounty. |

**Annonce publique cloud** : pas avant v1.3.0, jamais simultanément avec la candidature data.gouv.fr (dilution du pitch souverain).

---

## Validation

- Cohérence CLAUDE.md §7 ✓ — local par défaut, cloud opt-in explicite.
- Cohérence ADR-0013 ✓ — l'Alt 2 reste rejetée (cloud propriétaire fermé). L'Alt 3 (cloud open-source) est un raffinement, pas une contradiction.
- Couverture besoin TPE/PME sans IT ✓ — offre managed.
- Couverture besoin souverain ✓ — self-hosted reste l'option par défaut, gratuit, libre.
- Modèle économique soutenable ✓ — opex managé, pas de privatisation de features.

---

## Lien avec les chantiers en cours et à venir

- **C29 (v0.7.1)** : pas impacté. Continue tel que prévu.
- **C30 (état des lieux datasets)** : à exécuter après v1.0, enrichit le référentiel Gold qui sera diffusé via DVC à tous les utilisateurs (self-hosted ET managed). Pas de différenciation.
- **C-candidature (v1.0)** : pitch souverain pur, le cloud n'est pas mentionné.
- **C-admin-avance (v1.2)** : SSO, RBAC, multi-device. **Open-source dans le binaire commun**, pas exclusif managed.
- **C-cloud-beta (v1.3)** : infrastructure managed dédiée. Code = binaire commun.

---

## Révision

Ce ADR est révisable mais pas amendable silencieusement. Toute modification d'une des 5 conditions impératives nécessite :

1. Une décision écrite de Thibault dans un nouveau ADR-0014-amendement.
2. Une justification publique dans le CHANGELOG + README.
3. Une période de préavis de 30 jours pour les utilisateurs managed avant application.

Le but est de **garantir aux utilisateurs que les règles du jeu ne changeront pas en silence** — c'est ce qui distingue Sobr.ia d'un SaaS classique.
