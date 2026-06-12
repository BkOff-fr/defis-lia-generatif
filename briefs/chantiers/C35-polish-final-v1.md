# Chantier C35 — Polish final v1.0 (production-grade)

> **Version cible** : v0.9.5 → v1.0-rc1
> **Sprint** : ~5-7 jours
> **Pré-requis** : v0.9.0 (C34) + site-v0.1.0 (C33) shippés
> **Lien** : ADR-0014 (privacy by design), brief C30 (audit datasets), audit C32.0 (produit)
> **Cible** : passage de "produit solide mais immature pour le pitch" à "production-grade défendable jury data.gouv.fr".

---

## 0. Pourquoi ce chantier

L'**état des lieux 2026-05-17** (`docs/etat-des-lieux-2026-05-17.md`) liste les dettes 🟠 importantes qui ne sont pas bloquantes individuellement mais qui, accumulées, font la différence entre un projet "ça marche" et un projet "production".

C35 nettoie ces dettes en un sprint focalisé avant le dépôt candidature v1.0.

---

## 1. Périmètre

### C35.1 — Code signing binaires Tauri (1-2 j)

- **macOS** :
  - Inscrire au programme Apple Developer ($99/an) si pas fait.
  - Générer un Developer ID Application certificate.
  - Signer chaque `.dmg` produit par `app-release.yml` (`codesign --deep --force --options runtime --sign "Developer ID Application: Thibault ..."`).
  - Notarization Apple (`xcrun notarytool submit ... --wait`).
  - Stapler le ticket (`xcrun stapler staple ...`).
  - GitHub Secrets : `APPLE_DEVELOPER_ID`, `APPLE_DEVELOPER_TEAM_ID`, `APPLE_CERTIFICATE_P12_BASE64`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID_USERNAME`, `APPLE_ID_PASSWORD`.
- **Windows** :
  - Acheter EV Code Signing Certificate (~150-300€/an via DigiCert/Sectigo) OU certificat OV moins cher mais reputation SmartScreen plus lente.
  - Signer chaque `.msi` + `.exe` (`signtool sign /tr ... /td sha256 /fd sha256 /f cert.pfx /p password`).
  - GitHub Secrets : `WINDOWS_CERTIFICATE_PFX_BASE64`, `WINDOWS_CERTIFICATE_PASSWORD`.
- **Linux** :
  - Pas de signing OS-level. GPG signer les `.deb` + `.AppImage` (optionnel mais bon).
- **DoD** :
  - macOS : `spctl -a -t exec /Applications/Sobr.ia.app` retourne `accepted` sans warning.
  - Windows : SmartScreen ne bloque pas le double-clic sur `.msi`.
  - Linux : `.deb` installable + `.AppImage` exécutable.
  - Site `/telecharger/` retire le disclaimer "signature en cours".

**Note coût** : ~250-400€/an cumulés (Apple Developer + EV cert Windows). À budgéter par Thibault avant.

### C35.2 — DVC remote public + référentiel Gold reproductible (0.5 j)

- Configurer un DVC remote public (HTTPS read-only) où le `data/gold/referentiel.sqlite` est publié à chaque release.
- Options :
  - **S3 / OVH Object Storage / Scaleway Object Storage** (~1-5€/mois selon volume).
  - **GitHub Releases** comme fallback (gratuit mais limite 2 GB / asset, ok pour notre cas).
- Workflow `.github/workflows/dvc-release.yml` :
  - Trigger : tag `v[0-9]+.[0-9]+.[0-9]+`.
  - Build pipeline médaillon complet (`cargo run -p sobria-ingest -- pipeline run` avec données réelles).
  - `dvc push` vers remote public.
  - Upload `referentiel.sqlite` + `analytics.parquet` + `datasheet.jsonld` + `MANIFEST.sha256` en assets GitHub Release.
- Doc `docs/operations/dvc-remote.md` : comment un évaluateur reproduit le Gold à partir des sources Etalab 2.0.

**DoD** : un externe peut clone le repo + `dvc pull` (ou télécharger les assets release) + `dvc repro` et obtenir le MÊME `MANIFEST.sha256` que celui publié. Pitch "reproductible scientifique" tenu.

### C35.3 — Privacy Policy publique RGPD (0.5 j)

- Rédiger `docs/legal/privacy-policy.md` complet (FR + EN si possible) couvrant :
  - **App Tauri standalone** : 100 % local, aucune donnée ne quitte le device.
  - **Extension navigateur** : `chrome.storage.local` uniquement, pas de tracking, pas d'analytics.
  - **Mode Équipe self-hosted** : données chez l'admin de l'entreprise, pas chez Sobr.ia.
  - **Future offre cloud managed** (placeholder) : conditions à venir, opt-in explicite.
- Ajouter section dans `/cloud/` du site avec lien.
- Ajouter footer `/legal/privacy/` sur tout le site.
- Pas de DPO obligatoire tant qu'on n'a pas de SaaS managed.

**DoD** : page publique en ligne sur le site + lien dans tous les footers.

### C35.4 — Polish personas Student + Enterprise (1-2 j)

L'audit C32.0 a donné Student 4/10 et Enterprise 5/10. C32 a partiellement corrigé. Reste à finir :

**Student** :
- Ajouter dans M1 Atelier un "mode Découverte" : 3 prompts d'exemple cliquables (1 question simple, 1 prompt code, 1 image) → estimation auto-calculée → équivalence humaine visible.
- Ajouter dans M15 Dashboard un widget "Votre semaine en équivalences" (X douches, Y km voiture).
- Ajouter dans M25 Eco-budget un slider rapide "Objectif simple : moitié de la moyenne" → fixe le budget auto.
- Cible : score audit C32.0 Student → 7/10.

**Enterprise** :
- Onboarding Mode Équipe enrichi : wizard 3 étapes dans `/parametres → Mode Équipe` (Télécharger serveur → Initialiser → Distribuer codes) avec visu graphique des étapes.
- Quickstart Mode Équipe en 1 page PDF téléchargeable depuis l'app et le site (`/cloud/`).
- Section "Cas d'usage entreprise" sur le site avec 3 scenarii : DSI 50 collab, RSE reporting CSRD, Procurement marchés publics frugaux.
- Cible : score audit C32.0 Enterprise → 7/10.

**DoD** : refaire smoke test 5 personas et documenter score actualisé dans `docs/qa/smoke-test-v1.0-rc.md`.

### C35.5 — Audits sécu finaux (0.3 j)

- `cargo audit` : 0 vulnérabilité critique/high.
- `cargo deny check` : licences OK, pas de dep banned.
- `cd web && npm audit --audit-level=high` : 0 critique/high (les 6 moderate de extension devDeps acceptables si justifiées).
- `cd web-team && npm audit --audit-level=high` : 0 critique/high.
- `cd extension && npm audit --audit-level=high` : 0 critique/high.
- `cd site && npm audit --audit-level=high` : 0 critique/high.
- Ajouter `cargo deny` + `npm audit` dans le workflow CI principal (fail si nouveau critique apparaît).

**DoD** : badge "Audits sécu OK" affichable.

### C35.6 — Dossier candidature data.gouv.fr (1 j)

- Compléter `docs/candidature/` :
  - PDF de 8-12 pages : pitch, valeur, méthodologie, datasets, ADRs clés, captures, équipe.
  - Vidéo démo 2 minutes (capture écran + voix-off) montrant :
    - 1 prompt sur M1 (avec équivalence carbone)
    - Comparaison de modèles M3
    - Vue Territoire FR M20 (différenciateur)
    - Export rapport CSRD M22
    - Extension navigateur en action sur ChatGPT
    - Mode Équipe dashboard admin
  - Captures écran haute résolution × 10.
  - Schéma architecture monorepo.
  - Liens vers : repo GitHub, site, DOI Zenodo, MANIFEST.sha256 du référentiel.
- Soumission sur la plateforme défi data.gouv.fr (à voir avec Thibault le timing).

**DoD** : dossier prêt à soumettre, validé par Thibault avant push.

### C35.7 — Bump v1.0 + tag + release notes (0.2 j)

- Bump versions Cargo + tauri.conf + web + extension + web-team + site → tous **1.0.0**.
- CHANGELOG `[1.0.0] — YYYY-MM-DD — Première release candidature data.gouv.fr` avec récap complet de v0.4 → v1.0.
- Tag `v1.0.0` annoté.
- Release notes GitHub complètes avec liens téléchargements.
- Push.

---

## 2. Definition of Done v1.0.0

- [ ] C35.1 binaires signés et installables sans warning Mac/Windows.
- [ ] C35.2 référentiel Gold reproductible publiquement via DVC remote.
- [ ] C35.3 privacy policy publique en ligne.
- [ ] C35.4 audit C32.0 v2 : Student ≥ 7/10, Enterprise ≥ 7/10.
- [ ] C35.5 `cargo audit`, `cargo deny`, `npm audit` propres (0 critique/high).
- [ ] C35.6 dossier candidature + vidéo démo prêts.
- [ ] C35.7 versions bumpées 0.9.x → 1.0.0 partout.
- [ ] Site `sobria.brilliantstudio.co` à jour avec v1.0.
- [ ] Workflow CI déclenché par tag v1.0.0 produit binaires signés.
- [ ] Tag `v1.0.0` poussé.

---

## 3. Anti-périmètre

- Pas de nouvelles features moteur (figé après C34).
- Pas de cloud beta (ADR-0014 Phase 5 = v1.3+).
- Pas de mobile (Android/iOS différé tag major).
- Pas d'i18n EN complète (anglais v1.1).
- Pas d'analytics même Plausible self-hosted (v1.1).

---

## 4. Risques + mitigations

| Risque | Mitigation |
|---|---|
| Achat certificats coûteux (Apple Dev + Windows EV) | Budget à valider Thibault, ~400€/an. Sinon, accepter SmartScreen le temps de bâtir réputation. |
| DVC remote public coûteux si trafic important | OVH Object Storage minimal ou GitHub Releases (gratuit jusqu'à 2 GB/asset). |
| Vidéo démo prend plus de temps que prévu | Limiter à 90 sec sans voix-off si pressé, captures longues. |
| Smoke test 5 personas révèle régression | Patches express avant v1.0 tag. |

---

## 5. Découpage temporel

| Jour | Sous-chantier | Livrable |
|---|---|---|
| J1 | C35.1 code signing (1/2) | Apple Developer setup + signing macOS |
| J2 | C35.1 fin + C35.2 + C35.3 | Windows signing + DVC remote + privacy policy |
| J3-J4 | C35.4 polish personas Student + Enterprise | Mode découverte, wizard mode équipe, score 7/10+ |
| J5 | C35.5 + C35.6 (1/2) | Audits sécu + début dossier candidature |
| J6 | C35.6 fin + vidéo démo | Dossier complet + vidéo 2 min |
| J7 | C35.7 ship v1.0 | Tag + release notes + soumission data.gouv.fr |

Total : **~7 jours**.

---

## 6. Et après v1.0 ?

- **C36 UAT externe** : peut être lancé en parallèle de C35 (cf. brief `C36-uat-externe.md`).
- **v1.1 C31 intégration Tier 2 datasets** : 10 j post-candidature.
- **v1.1 Doc quickstart self-hosting** : 0.5 j.
- **v1.2 Admin avancée (SSO/RBAC)** : sprint dédié.
- **v1.3 Cloud beta managed** : ADR-0014 Phase 5.
