# Sobr.ia pour Entreprise (DSI, RSE)

> **Piloter votre scope 3 IA, rapport CSRD, forecast budget carbone
> — sans envoyer un seul prompt vers un cloud externe.**

---

## Qui c'est ?

Vous êtes DSI, responsable RSE, contrôleur·euse de gestion dans une
TPE/PME, une ETI ou une grande organisation. Vos collaborateurs
utilisent des LLMs (Copilot, ChatGPT Team, Claude pour Entreprise,
Mistral) et vous devez **rendre des comptes** :

- Reporting CSRD trimestriel sur scope 3 IA.
- Budget carbone par équipe / business unit.
- Justifier votre choix de fournisseur LLM sur des critères
  environnementaux mesurables.

## Ce que Sobr.ia résout pour vous

| Question | Réponse Sobr.ia |
|---|---|
| « Comment agréger l'usage IA de mes 50 collaborateurs sans cloud externe ? » | **Mode Équipe self-hosted** — binaire Rust standalone sur un VPS interne, JWT + Argon2id, dashboard admin |
| « Comment sortir un rapport CSRD trimestriel ? » | **Rapport réglementaire (CSRD/AGEC)** — PDF signé + JSON-LD PROV-O conforme AFNOR SPEC 2314 |
| « Comment fixer un budget IA par équipe avec alertes ? » | **Alertes seuils v0.7.1** — webhook ou email quand un plafond gCO₂eq par jour/semaine/mois est franchi |

## Top 3 use cases

1. **Déployer le Mode Équipe pour 50 collaborateurs** — un binaire
   `sobria-team-aggregator` déployé en interne, codes d'enrôlement
   12 chiffres distribués aux employés, dashboard admin agrégé.
2. **Sortir un rapport CSRD trimestriel signé** — PDF + JSON-LD
   PROV-O, conformité AFNOR SPEC 2314, traçabilité méthodologique
   complète (colonne `method` dans le ledger).
3. **Définir des alertes seuils** — plafond mensuel par équipe,
   notification webhook → Slack / Teams ou email SMTP, sans
   intervention quotidienne.

## Modules pertinents

- **Estimer un prompt** — pour vos tests de référence
- **Journal d'audit** — preuve d'auditabilité scope 3
- **Datacenters Europe** — comprendre où tournent vos LLMs
- **Tableau de bord** — vue agrégée perso
- **Datasheet scientifique** — reproductibilité méthodologique
- **Territoire FR** — empreinte territoriale (si DSI publique)
- **Rapport réglementaire (CSRD/AGEC)** — l'output réglementaire
- **Eco-budget** — objectifs mensuels personnels (équivalent équipe
  via le Mode Équipe)

## Quickstart 5 minutes — Mode Équipe

```bash
# 1. Téléchargez sobria-team-aggregator-linux-x86_64 depuis Releases
chmod +x sobria-team-aggregator-linux-x86_64

# 2. Initialisez (1 fois) — admin + base SQLite + TLS auto-signé
./sobria-team-aggregator --data-dir ./team-data init \
    --admin-username admin --admin-password 'CHANGE-ME'

# 3. Lancez le serveur en écoute HTTPS sur le port 8443
./sobria-team-aggregator --data-dir ./team-data serve --port 8443

# 4. Ouvrez https://votre-serveur:8443/admin
#    → Créez des codes d'enrôlement 12 chiffres
#    → Distribuez-les à vos collaborateurs

# 5. Chaque collaborateur installe l'app Sobr.ia desktop ou
#    l'extension, colle le code dans /parametres, et ses
#    estimations remontent dans le dashboard agrégé
```

**Aucun cloud Sobr.ia n'est impliqué** — votre entreprise contrôle
son serveur, ses données, ses sauvegardes.

→ Doc complète : [`docs/operations/team-aggregator.md`](../operations/team-aggregator.md)

## Pour aller plus loin

- [Mode Équipe — opérations courantes](../operations/team-aggregator.md)
- [ADR-0013 Phase 2 — Mode Équipe](../adr/ADR-0013-extension-pairing-team-mode.md)
- [ADR-0014 — Dual-track local + cloud opt-in (v1.3+)](../adr/ADR-0014-dual-track-local-cloud.md)
- [Conformité AFNOR SPEC 2314](../methodology/AFNOR-SPEC-2314-synthese.md)
