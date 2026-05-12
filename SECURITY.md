# Politique de sécurité — Sobr.ia

## Versions supportées

Tant que le projet n'est pas en v1.0, seule la branche `main` est supportée.

| Version | Statut |
|---------|--------|
| 0.x (pré-release) | Branche `main` uniquement |

---

## Signaler une vulnérabilité

**NE PAS ouvrir d'issue publique** pour une vulnérabilité de sécurité.

Envoyez un email à : `security@sobr.ia` (à configurer) — ou ouvrez un [Security Advisory privé sur GitHub](https://docs.github.com/en/code-security/security-advisories/privately-reporting-a-security-vulnerability).

Inclure dans le rapport :
- Description du problème
- Étapes de reproduction
- Impact potentiel
- (Optionnel) suggestion de correctif

**Engagement de réponse** :
- Accusé de réception sous 72 h.
- Évaluation initiale sous 7 jours.
- Correctif ou plan de remédiation sous 30 jours pour les vulnérabilités critiques.

---

## Périmètre de sécurité

Sobr.ia traite :
- Des prompts utilisateurs **localement** (pas d'envoi externe sauf opt-in explicite).
- Un audit ledger ACID signé SHA-256.
- Une extension navigateur MV3 avec permissions minimales.

Hypothèses de sécurité :
- Le système d'exploitation et le navigateur de l'utilisateur sont sains.
- Les sources de données publiques (data.gouv.fr, ADEME, HF) ne sont pas compromises.

---

## Pratiques sécurité dans le code

Voir CLAUDE.md §7.

- TLS via `rustls` (pas d'OpenSSL).
- Hashs : SHA-256 / BLAKE3 / Argon2.
- Pas de secrets en clair, jamais commités.
- Dépendances auditées en CI (`cargo audit`, `cargo deny`, `npm audit`).
- SBOM (CycloneDX) généré à chaque release.

---

## Bug bounty

Pas de programme actif en pré-release. Toute découverte responsable sera créditée
publiquement dans le `CHANGELOG.md` (sauf demande contraire du chercheur).
