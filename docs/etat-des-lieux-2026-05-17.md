# État des lieux Sobr.ia — 2026-05-17

> **Statut** : v0.8.0 shippée, **PAS prête pour v1.0 candidature data.gouv.fr**.
> **Auteur** : Cowork (audit interne honnête).
> **Demande Thibault** : *« on n'est pas prêts pour v1.0 »* → confirmé. Voici pourquoi et quoi faire.

---

## 1. Ce qui est shippé (v0.4 → v0.8)

| Version | Date | Périmètre | Qualité |
|---|---|---|---|
| v0.4.0 | 2026-05-14 | Catalogue multi-méthodologie AFNOR + EcoLogits | ✅ Solide |
| v0.5.0 | 2026-05-15 | Pipeline médaillon Copper→Silver→Gold (ComparIA + RTE-IRIS) | ✅ Solide |
| v0.6.0 | 2026-05-16 | Extension navigateur + pairing perso + auto-install bridge | ✅ Solide |
| v0.7.0 | 2026-05-16 | Mode Équipe self-hosted (binaire) + dashboard + exports CSRD/PROV-O | ✅ Solide |
| v0.7.1 | 2026-05-16 | Polish Mode Équipe (UI Tauri + admin reset-password + alertes) | ✅ Solide |
| **v0.8.0** | **2026-05-17** | **Clarté produit (C32)** : README refondu, 5 personas, équivalences carbone, vendors disclosure (Mistral × ADEME, Google Gemini, Meta Llama), DOI Zenodo | ✅ **Solide** |

Côté code on a fait du **bon boulot**. 8 versions en 3 jours, tests verts, clippy propre, méthodologie sérieuse.

---

## 2. Dettes techniques accumulées (par criticité pour v1.0)

### 🔴 Bloquant candidature data.gouv.fr — DOIT être fait avant v1.0

| # | Dette | Effort | Sprint |
|---|---|---|---|
| 1 | **Catalogue modèles obsolète** : 8 presets 2024 (gpt-4o, llama-3.1, etc.). Manque Claude 4.7, GPT-5/5.5, Gemini 2.5, Llama 4, DeepSeek R1, etc. | 1.5 j | **C34** |
| 2 | **Modalités absentes** : pas de support vision/document/audio. 30 % usage 2026 est multimodal → moteur aveugle. | 1.5 j | **C34** |
| 3 | **Overhead contextualisation** : moteur ignore system prompt (1k-2k tokens) → P50 sous-estimé 5×-30×. Inacceptable scientifiquement. | 1.5 j | **C34** |
| 4 | **Workflows CI pour binaires** : aucun workflow build Tauri (Win/macOS/Linux) ni extension (zip/xpi). Tags v0.4 → v0.8 = pas d'assets dans GitHub Releases. **Rien à télécharger**. | 1 j | **C33** |
| 5 | **Site internet** : pas de pitch externe pour candidature. Repo GitHub austère = jury data.gouv.fr ne croit pas le projet. | 6-8 j | **C33** |
| 6 | **Cross-validation moteur vs facturation réelle** : aucune comparaison Sobr.ia P50 vs facture API OpenAI/Anthropic/Google. Si écart > 30 %, pitch invalidé en 30 sec. | 0.5 j | **C34.6** |

### 🟠 Important pour crédibilité v1.0 — Devrait être fait

| # | Dette | Effort | Sprint |
|---|---|---|---|
| 7 | **Code signing binaires** : Tauri non signé → Gatekeeper macOS + SmartScreen Windows = warnings agressifs. Adoption tuée. | 0.5-2 j | Polish v0.9.1 ou v1.0 |
| 8 | **DVC remote public** : `data/gold` produit mais pas publié → un évaluateur ne peut pas reproduire. Le pitch "scientifique reproductible" s'effondre. | 0.5 j | v1.0 |
| 9 | **UAT externe 5 testeurs** : promis par le brief candidature, jamais fait. Bugs UX non découverts. | 3-5 j | v1.0 |
| 10 | **Privacy Policy publique** : page placeholder seulement. RGPD-compliant doc nécessaire pour offre cloud future + crédibilité actuelle. | 0.5 j | v1.0 |
| 11 | **Audit sécu cargo audit + npm audit clean** : 6 vulnérabilités moderate dans extension devDeps (vite/vitest). Pas bloquant mais à nettoyer. | 0.3 j | Patch v0.8.1 |
| 12 | **Stack trace en clair côté Tauri** : si erreur runtime, on affiche probablement le panic Rust brut. Mauvaise UX. | 0.5 j | v1.0 |

### 🟡 Nice-to-have — Différé v1.1+

| # | Dette | Effort | Sprint |
|---|---|---|---|
| 13 | i18n EN partielle (toggle désactivé en C32) | 2-3 j | v1.1 |
| 14 | Intégration Tier 2 datasets (ADEME, ML.ENERGY, IEA, EpochAI…) | ~10 j | **C31 v1.1** |
| 15 | Performance Tauri profilée + optimisée | 1-2 j | v1.1 |
| 16 | Pentest externe sécurité | budget | v1.x |
| 17 | Tests E2E Playwright couverture > 70 % | 3-4 j | v1.1 |
| 18 | Mobile builds Android (tag major uniquement) | 3-5 j | v1.1+ |
| 19 | iOS distribution App Store ou TestFlight | budget Apple | v1.x |
| 20 | Cloud beta managed (ADR-0014 Phase 5) | sprint dédié | v1.3+ |

### 🟢 Pas une dette mais résultats audit C32.0 à surveiller

| Persona | Score clarté actuel | Risque pitch |
|---|---|---|
| Researcher | 8/10 | ✅ Servi |
| Pro Tech | 7/10 | ✅ OK |
| Public Sector | 6/10 | ⚠️ Tagline "marchés publics" non délivrée par module |
| Enterprise | 5/10 | ⚠️ Mode Équipe install pas vraiment guidé malgré C32 |
| Student | 4/10 | 🔴 Plus gros gap, persona avec plus gros potentiel adoption |

---

## 3. Roadmap réaliste vers v1.0

### Option A — Sprint final court (10-12 jours)

Focus minimum vital pour candidature crédible :

| Sprint | Effort | Livrable |
|---|---|---|
| **C34 catalogue 2026 + modalités + overhead** | 6 j | v0.9.0 — moteur crédible scientifiquement |
| **Validation moteur vs API réelles** (C34.6 enrichi) | 0.5 j | Confiance pitch |
| **C33 site internet** | 6-8 j | Visibilité externe |
| **C33.x workflows CI binaires** (inclus C33) | 1 j | Téléchargements fonctionnels |
| **Polish final pré-candidature** | 1 j | DVC remote public, privacy policy, audits sécu |
| **Total minimum réaliste** | **~14-16 j** | **v1.0 RC** |

### Option B — Sprint final propre (3-4 semaines)

Tout Option A + dettes 🟠 importantes :

| Sprint | Effort cumulé | Ajout |
|---|---|---|
| Option A | 14-16 j | Base |
| + Code signing binaires | +2 j | Adoption sereine Mac/Win |
| + UAT 5 testeurs externes | +3-5 j | Bugs UX découverts |
| + Polish persona Student + Enterprise | +2 j | Audit C32.0 score → 7/10 minimum partout |
| + Privacy policy publique RGPD | +0.5 j | Crédibilité |
| **Total propre** | **~22-26 j** | **v1.0 production-grade** |

### Option C — Ship v0.9.0 maintenant, candidature plus tard

On accepte qu'on n'est pas prêts, on continue chantier par chantier, et on **ne candidate qu'en v1.0 réellement prêt**. Pas de pression artificielle.

| Phase | Quand | Livrable |
|---|---|---|
| v0.9.0 | +6 j | C34 catalogue + modalités |
| site-v0.1.0 | +8 j après v0.9 | C33 |
| v0.9.x patches | +X j | UAT + code signing + polish |
| v1.0.0 candidature | Quand vraiment prêt | Soumission data.gouv.fr |

**Recommandation Cowork** : option **C** — pas de pression sur la date candidature. On ship chaque sprint quand il est solide. v1.0 = date d'arrivée, pas date fixée.

---

## 4. Décision à prendre

Trois axes de décision indépendants à confirmer :

1. **Timing candidature data.gouv.fr** : date butoir externe imposée ? Si oui, on adapte. Si pas, option C.
2. **Périmètre v1.0** : minimum vital (Option A) ou production-grade (Option B) ?
3. **Ordre des chantiers immédiats** : C34 d'abord (déjà briefé), C33 ensuite, puis polish final.

---

## 5. Mon avis honnête

- v0.8.0 actuelle = **produit techniquement solide** mais **immature côté pitch externe** (pas de site, modèles 2024, modalités absentes).
- Si on candidate maintenant, on est honnêtes mais on se prend une **objection majeure** dès la lecture du moteur (catalogue obsolète).
- Si on attaque les 3 chantiers déjà briefés (C34 + C33 + polish final), on a **un bon v1.0 dans ~2-3 semaines** — c'est rapide pour ce qu'on a accompli.
- **Ne pas rusher la candidature** est probablement la bonne décision. Un projet refusé pour "moteur obsolète" est pire qu'un projet déposé 2 semaines plus tard avec tous les arguments.

---

## 6. Récap actions ouvertes

| ID | Chantier | Statut | Effort |
|---|---|---|---|
| C34 | Catalogue 2026 + modalités + overhead → v0.9.0 | 📋 Brief prêt, à lancer | ~6 j |
| C33 | Site internet (Astro + 3D + workflows CI binaires) → site-v0.1.0 | 📋 Brief prêt, après C34 | ~6-8 j |
| C31 | Intégration Tier 2 datasets | 📋 Différé v1.1 post-candidature | ~10 j |
| **Polish final** | Code signing + DVC public + privacy policy + UAT | 📋 À organiser | ~3-5 j |
| **v1.0.0 candidature** | Soumission data.gouv.fr | 📋 Date à fixer | ~1 sem |

Pas de pression. On ship quand c'est solide.
