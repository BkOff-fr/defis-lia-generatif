# Chantier #8 — sobria-audit : ledger ACID chaîné

> **Pré-requis** : v0.1.1-estimator (C05-C07) mergé.
> **Crate touchée** : `sobria-audit` (nouvelle implémentation).
> **Durée cible** : 1-2 jours.
> **Référence CDC** : module M7, EF-M7-01 à EF-M7-05.

---

## 0. Objectif

Implémenter le **journal d'audit** de Sobr.ia : chaque `EstimationResult`
produit par le moteur est inscrit dans un ledger SQLite **ACID + WAL +
chaîné SHA-256**. Cela permet :

- **Traçabilité réglementaire** (CSRD) : prouver qu'une estimation
  publiée à une date donnée a bien été calculée avec ces paramètres.
- **Anti-tampering** : si quelqu'un modifie *a posteriori* une entrée
  passée, la chaîne se brise et `verify_chain()` le détecte.
- **Export pour audit externe** : NDJSON signé, vérifiable hors-app.
- **Conformité RGPD** : purge possible avec maintien de l'intégrité.

## 1. Modèle de chaînage

Chaque entrée `i` contient :

```
entry_i = {
    id: i,                       // auto-increment SQLite
    timestamp: T_i,              // UTC RFC 3339
    estimation_result: R_i,      // JSON sérialisé d'un EstimationResult
    prev_sig: sig_{i-1},         // signature de l'entrée précédente
    sig: SHA256(T_i || R_i || prev_sig_i)
}
```

L'entrée *genesis* (`i=1`) a `prev_sig = ""` (chaîne vide).

**Vérification** : pour valider la chaîne entière, on itère ordered by id
et on vérifie :
1. `entry_i.sig == SHA256(T_i || R_i || prev_sig_i)` (signature interne).
2. `entry_i.prev_sig == entry_{i-1}.sig` (continuité chaîne).

Si l'une de ces propriétés est cassée, on connaît exactement la première
entrée altérée.

## 2. Schéma SQLite

```sql
CREATE TABLE IF NOT EXISTS audit_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,            -- ISO 8601 UTC
    estimation_result_json TEXT NOT NULL,
    prev_sig TEXT NOT NULL,             -- "" pour la genesis
    sig TEXT NOT NULL                   -- 64 hex chars
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_entries(timestamp);
```

Mode WAL activé à l'open. Index sur timestamp pour les requêtes par période.

## 3. API publique

```rust
pub struct AuditLedger { /* private fields */ }

impl AuditLedger {
    /// Ouvre ou crée un ledger sur disque (mode WAL).
    pub fn open(path: &Path) -> AuditResult<Self>;

    /// Ajoute une entrée chaînée. Retourne l'entrée écrite.
    pub fn append(&mut self, result: &EstimationResult) -> AuditResult<AuditEntry>;

    /// Vérifie l'intégrité complète de la chaîne.
    pub fn verify_chain(&self) -> AuditResult<IntegrityReport>;

    /// Nombre d'entrées.
    pub fn len(&self) -> AuditResult<usize>;

    /// Exporte toutes les entrées en NDJSON dans `writer`.
    pub fn export_ndjson(&self, writer: &mut impl io::Write) -> AuditResult<usize>;

    /// Purge RGPD : marque les entrées antérieures à `before` comme
    /// supprimées (champ `estimation_result_json` remplacé par sentinel
    /// "PURGED"), mais conserve le hash original dans `sig` → la chaîne
    /// reste vérifiable.
    pub fn purge_before(&mut self, before: DateTime<Utc>) -> AuditResult<usize>;
}

pub struct IntegrityReport {
    pub total_entries: usize,
    pub valid: bool,
    pub first_invalid_id: Option<i64>,
    pub message: String,
}
```

## 4. Stratégie RGPD avec maintien d'intégrité

Le problème classique d'un ledger chaîné : si on supprime une entrée
pour respecter le droit à l'oubli RGPD, la chaîne casse.

**Solution Sobr.ia v1** : la purge **n'efface pas la signature**, elle
remplace seulement le payload par un sentinel `"PURGED"` (avec hash du
payload original conservé pour audit). Ainsi :
- L'utilisateur ne peut plus reconstituer le prompt d'origine.
- L'auditeur externe constate qu'une purge a eu lieu (date, nb d'entrées).
- La chaîne reste intacte cryptographiquement.

```sql
ALTER TABLE audit_entries ADD COLUMN purged_at TEXT;
-- On garde sig original (pas de re-signature après purge).
```

Trade-off explicite : on satisfait le **droit à l'oubli** sur le contenu
sensible (prompts, résultats) tout en préservant la **piste d'audit**
réglementaire. Documenté dans `docs/methodology/AUDIT-LEDGER.md`.

## 5. Tests requis

### 5.1 Comportement standard
- `open` + `append` × 1 → `verify_chain` retourne `valid: true, total: 1`.
- `append` × 100 → chaîne valide de 100 entrées.
- Le `prev_sig` de l'entrée *i* égale le `sig` de l'entrée *i-1*.

### 5.2 Anti-tampering
- Modifier directement `estimation_result_json` via SQL → `verify_chain`
  retourne `valid: false, first_invalid_id: Some(...)`.
- Modifier un `prev_sig` → idem.

### 5.3 Export
- `export_ndjson` produit N lignes JSON valides.
- Re-parse de chaque ligne → contenu cohérent.

### 5.4 Purge RGPD
- `purge_before(t)` purge les entrées antérieures.
- `verify_chain` reste valide après purge.
- Entrée purgée : payload = sentinel, sig conservé.

## 6. Definition of Done

- [ ] `cargo build` + `cargo test -p sobria-audit` verts.
- [ ] `cargo clippy -p sobria-audit -- -D warnings` passe.
- [ ] ≥ 10 tests unitaires.
- [ ] `docs/methodology/AUDIT-LEDGER.md` documente le chaînage + RGPD.
- [ ] Pas de panic en code de production, errors typées.

## 7. Non-objectifs (v2)

- Signature GPG des exports NDJPN (cryptographique avancée).
- Compression du ledger (zstd) — non critique au volume attendu.
- Multi-tenant (plusieurs ledgers parallèles).
- Synchronisation avec un timestamping service externe (TSA).
