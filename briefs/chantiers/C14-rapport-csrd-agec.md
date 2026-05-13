# Chantier #14 — M22 Rapport CSRD/AGEC

> **Pré-requis** : v0.2.3-territoire-fr mergé.
> **Crates touchées** : `sobria-export` (activation), `sobria-app` (DTOs + IPC).
> **Frontend** : `web/src/routes/(modules)/m22/+page.svelte` — chantier
> Claude Code séparé.
> **Durée cible** : 2 jours Rust.
> **Référence CDC** : v1.3 §4 M22, méthodologie §6.

---

## 0. Objectif

Produire, pour une période donnée, un **rapport PDF officiel** consommable
par la fonction RSE/DSI/CSRD d'une organisation, accompagné de son
**JSON-LD PROV-O** pour la reproductibilité scientifique et l'audit
réglementaire.

Le rapport répond à trois cas d'usage :

1. **CSRD** (Corporate Sustainability Reporting Directive — UE 2024+) :
   les entreprises >250 salariés doivent reporter leur scope 3 numérique,
   IA incluse. M22 livre un PDF prêt à insérer dans le rapport annuel ESG.
2. **AGEC** (loi française 2020 anti-gaspillage et économie circulaire) :
   exige des collectivités la mesure de leur empreinte numérique. M22
   produit l'extrait conforme.
3. **AFNOR SPEC 2314** (référentiel français mesure environnementale IA) :
   le format du rapport suit les sections obligatoires de la spec.

## 1. Contenu du PDF (8-12 pages typiques)

### 1.1 Page de garde
- Titre : « Empreinte environnementale IA générative — [Org] — [Période] »
- Logo Sobr.ia (placeholder en v1.0, image fournie par l'org en v1.1)
- Métadonnées : période, date d'émission, version Sobr.ia, seed Monte-Carlo
- Mention « Rapport conforme AFNOR SPEC 2314, traçable PROV-O »

### 1.2 Synthèse exécutive (1 page)
- 3 indicateurs majeurs : **CO₂eq P50** (kg) + intervalle P5-P95, **énergie**
  (kWh), **eau** (L).
- Nombre total de requêtes journalisées.
- Comparaison équivalents parlants (km voiture, douches, écrans-heures).
- Variation vs période précédente si calculable.

### 1.3 Méthodologie (1-2 pages)
- Formule de calcul (AFNOR SPEC 2314, citée).
- Sources des paramètres (HF AI Energy Score, RTE, Electricity Maps, ADEME).
- Distribution Monte-Carlo, N = 10⁴, seed.
- Validation croisée (Luccioni 2023, EcoLogits 2024, à ± 15%).

### 1.4 Résultats détaillés (2-3 pages)
- Tableau : par modèle, nombre de requêtes, CO₂eq P5/P50/P95 cumulé,
  énergie, eau.
- Top 10 prompts par CO₂eq (anonymisés si purgés RGPD).
- Décomposition compute / embodied par famille.

### 1.5 Audit (1 page)
- ID de la première et dernière entrée du ledger sur la période.
- SHA-256 chaîné final.
- État `verify_chain()` au moment de la génération.
- URL/chemin du ledger pour audit externe.

### 1.6 Provenance (1 page)
- Tableau résumé du PROV-O (cf. §2).
- URL du JSON-LD complet.
- Signature SHA-256 du PDF.

### 1.7 Annexes
- Glossaire (CO₂eq, PUE, WUE, embodied carbon, etc.).
- Références bibliographiques (≥ 5 sources scientifiques).
- Licences (Etalab 2.0, MIT, CC-BY).

## 2. JSON-LD PROV-O

Format **W3C PROV-O** (<https://www.w3.org/TR/prov-o/>). Schéma type :

```json
{
  "@context": {
    "prov": "http://www.w3.org/ns/prov#",
    "sobria": "https://sobr.ia/vocab#",
    "schema": "https://schema.org/"
  },
  "@graph": [
    {
      "@id": "sobria:report-2026-Q1",
      "@type": "prov:Entity",
      "prov:generatedAtTime": "2026-05-13T14:32:00Z",
      "prov:wasGeneratedBy": {"@id": "sobria:activity-report-gen-..."},
      "schema:contentSha256": "abc123...",
      "schema:datePublished": "2026-05-13"
    },
    {
      "@id": "sobria:activity-report-gen-...",
      "@type": "prov:Activity",
      "prov:startedAtTime": "2026-05-13T14:31:58Z",
      "prov:endedAtTime": "2026-05-13T14:32:00Z",
      "prov:used": [
        {"@id": "sobria:audit-entries-1-247"},
        {"@id": "sobria:estimator-engine-v0.2"}
      ],
      "prov:wasAssociatedWith": {"@id": "sobria:agent-cowork"}
    },
    {
      "@id": "sobria:audit-entries-1-247",
      "@type": "prov:Entity",
      "schema:contentSha256": "audit_chain_final_sig",
      "sobria:entryCount": 247
    },
    {
      "@id": "sobria:estimator-engine-v0.2",
      "@type": "prov:Entity",
      "schema:version": "0.2.0",
      "sobria:seed": 42,
      "sobria:n": 10000
    }
  ]
}
```

## 3. API Rust publique

### 3.1 `sobria-export::report`

```rust
pub struct ReportRequest {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub organization_name: String,
    /// Locale UI ("fr" / "en"). v1.0 : fr uniquement, structure prête pour en.
    pub locale: String,
}

pub struct ReportArtifacts {
    /// PDF binaire.
    pub pdf_bytes: Vec<u8>,
    /// SHA-256 du PDF (hex 64 chars).
    pub pdf_sha256: String,
    /// JSON-LD PROV-O.
    pub provo_jsonld: serde_json::Value,
    /// Résumé des indicateurs agrégés sur la période.
    pub summary: ReportSummary,
    /// Nombre d'entrées d'audit incluses.
    pub audit_entries_count: usize,
}

pub struct ReportSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: u32,
    pub total_co2eq_g_p50: f64,
    pub total_co2eq_g_p5: f64,
    pub total_co2eq_g_p95: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
}

pub fn generate_report(
    req: &ReportRequest,
    ledger_entries: &[AuditEntry],
) -> Result<ReportArtifacts, ExportError>;
```

### 3.2 `sobria-app::logic::export_csrd_report`

Wrapper IPC :

```rust
pub fn export_csrd_report(
    req: CsrdReportRequest,
    output_dir: &Path,
    state: &AppState,
) -> IpcResult<CsrdReportResult>;
```

Comportement :
1. Charge les entrées d'audit pour la période depuis le ledger.
2. Appelle `sobria_export::generate_report`.
3. Écrit `report.pdf` et `provo.jsonld` dans `output_dir`.
4. **Journalise une nouvelle entrée d'audit** marquant la génération
   du rapport (ce qui crée une chaîne récursive : un rapport sur un
   ledger qui inclut sa propre génération).
5. Retourne `CsrdReportResult` avec les chemins + SHA-256.

## 4. Dépendances Rust

Ajouter au workspace :
- `printpdf = "0.7"` — PDF Rust pur, supporte text + tables basiques.

Pas de wkhtmltopdf / headless Chrome (privacy + frugalité).

Fonts embarquées :
- Liberation Serif (déjà OFL) ou DejaVu Serif depuis le système ;
  fallback Helvetica built-in dans printpdf.

## 5. Definition of Done

### Rust
- [ ] `sobria-export` activée, ajout `printpdf` à workspace.
- [ ] Module `report.rs` avec `ReportBuilder` + sections fonctionnelles.
- [ ] Module `provo.rs` qui sérialise le PROV-O à partir de la liste
      d'entrées d'audit + métadonnées rapport.
- [ ] DTOs `CsrdReportRequest` et `CsrdReportResult` côté `sobria-app`.
- [ ] Commande IPC `export_csrd_report`.
- [ ] ≥ 8 tests : PDF non vide, header AFNOR présent, SHA-256
      reproductible (avec seed), JSON-LD valide @context, entrée
      d'audit créée à la génération.

### Doc
- [ ] `docs/methodology/RAPPORT-CSRD-AGEC.md` : structure du rapport,
      mapping AFNOR SPEC 2314 → sections, exemple PROV-O complet.

## 6. Non-objectifs (différés)

- **Signature GPG cryptographique** du PDF → backlog v1.1.
- **Templating multi-organisations** (logo, charte) → v1.1.
- **EN translation** → C18 i18n.
- **Annexe Sankey embarquée** → v1.1 si printpdf gère.

## 7. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| printpdf limité en tables / texte multiligne | Moyenne | Layout simplifié, paragraphes prédécoupés |
| Bundle binaire grossit (~5 MB pour printpdf + fonts) | Faible | Acceptable au regard de la valeur |
| Reproductibilité PDF (timestamp d'émission) | Moyenne | Timestamp fixé via `req.period_end` (déterministe), pas `now()` |

---

*Brief Cowork. Exécution C14.1 (sobria-export PDF), C14.2 (DTOs + IPC),
C14.3 (tests). Prompt Claude Code séparé pour l'écran M22 + bouton
« Générer rapport CSRD ».*
