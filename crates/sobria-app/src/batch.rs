//! Traitement par lots de prompts en CSV (M18 / C21).
//!
//! Cycle :
//! 1. Parse + validate CSV input.
//! 2. Exécute `estimate_prompt` par ligne (journalisé dans le ledger).
//! 3. Agrège stats globales + par modèle.
//! 4. Optionnel : écrit les résultats dans un CSV.
//!
//! Voir `briefs/chantiers/C21-batch-csv.md`.

use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Nombre max de lignes par batch (anti-abus + cap perf).
pub const MAX_BATCH_ROWS: usize = 1000;

/// Tolérance maximale de lignes rejetées avant erreur globale (50%).
const REJECTION_RATIO_LIMIT: f64 = 0.5;
/// Même seuil exposé en pourcentage entier (pour les messages d'erreur).
const REJECTION_RATIO_LIMIT_PCT: u8 = 50;

#[derive(Debug, Error)]
pub enum BatchError {
    #[error("fichier introuvable : {0}")]
    FileNotFound(std::path::PathBuf),
    #[error("io : {0}")]
    Io(#[from] std::io::Error),
    #[error("csv : {0}")]
    Csv(#[from] csv::Error),
    #[error("format CSV invalide : {0}")]
    Format(String),
    #[error("trop de lignes : {got} (max {max})")]
    TooManyRows { got: usize, max: usize },
    #[error("trop de lignes rejetées : {rejected} / {total} (> {limit_pct}%)")]
    TooManyRejections {
        rejected: usize,
        total: usize,
        limit_pct: u8,
    },
    #[error("batch vide (aucune ligne de données)")]
    EmptyBatch,
}

pub type BatchResult<T> = Result<T, BatchError>;

/// Une ligne du CSV d'entrée.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchInputRow {
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    #[serde(default)]
    pub datacenter_id: Option<String>,
}

/// Une ligne de résultat batch.
#[derive(Debug, Clone, Serialize)]
pub struct BatchOutputRow {
    pub row_index: u32,
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub datacenter_id: Option<String>,
    pub co2eq_p5_g: f64,
    pub co2eq_p50_g: f64,
    pub co2eq_p95_g: f64,
    pub energy_wh_p50: f64,
    pub water_l_p50: f64,
    pub audit_id: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser CSV
// ─────────────────────────────────────────────────────────────────────────────

/// Parse le CSV d'entrée et retourne les lignes valides au format `BatchInputRow`.
/// Les erreurs de format (fichier absent, header mal formé, > max rows) sont
/// remontées. Les lignes individuellement invalides sont **comptées** mais ne
/// stoppent pas le parsing (validations métier en aval).
pub fn parse_csv(path: &Path) -> BatchResult<Vec<BatchInputRow>> {
    if !path.exists() {
        return Err(BatchError::FileNotFound(path.to_path_buf()));
    }
    let file = File::open(path)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(BufReader::new(file));

    // Vérifie le header.
    let headers = reader.headers()?;
    let expected = ["model_id", "tokens_in", "tokens_out", "datacenter_id"];
    if headers.len() < 3 {
        return Err(BatchError::Format(format!(
            "header trop court : {} colonnes (≥ 3 attendues)",
            headers.len()
        )));
    }
    for (i, col) in headers.iter().enumerate() {
        if i >= expected.len() {
            break;
        }
        if col.eq_ignore_ascii_case(expected[i]) {
            continue;
        }
        return Err(BatchError::Format(format!(
            "header colonne {} : '{}' attendu, '{}' trouvé",
            i, expected[i], col
        )));
    }

    // Parse les lignes.
    let mut rows: Vec<BatchInputRow> = Vec::new();
    for (idx, record) in reader.records().enumerate() {
        let record = record?;
        if rows.len() >= MAX_BATCH_ROWS {
            return Err(BatchError::TooManyRows {
                got: idx + 1,
                max: MAX_BATCH_ROWS,
            });
        }
        // Lignes vides → skip silencieusement.
        if record.iter().all(|f| f.trim().is_empty()) {
            continue;
        }
        let row = parse_record(&record)?;
        rows.push(row);
    }

    if rows.is_empty() {
        return Err(BatchError::EmptyBatch);
    }
    Ok(rows)
}

fn parse_record(record: &csv::StringRecord) -> BatchResult<BatchInputRow> {
    let model_id = record
        .get(0)
        .ok_or_else(|| BatchError::Format("colonne model_id manquante".into()))?
        .trim()
        .to_string();
    if model_id.is_empty() {
        return Err(BatchError::Format("model_id vide".into()));
    }
    let tokens_in: u32 = record
        .get(1)
        .ok_or_else(|| BatchError::Format("colonne tokens_in manquante".into()))?
        .trim()
        .parse()
        .map_err(|e| BatchError::Format(format!("tokens_in : {e}")))?;
    let tokens_out: u32 = record
        .get(2)
        .ok_or_else(|| BatchError::Format("colonne tokens_out manquante".into()))?
        .trim()
        .parse()
        .map_err(|e| BatchError::Format(format!("tokens_out : {e}")))?;
    let datacenter_id = record
        .get(3)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Ok(BatchInputRow {
        model_id,
        tokens_in,
        tokens_out,
        datacenter_id,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Export CSV résultats
// ─────────────────────────────────────────────────────────────────────────────

/// Écrit les résultats dans un fichier CSV.
pub fn export_results_csv(path: &Path, rows: &[BatchOutputRow]) -> BatchResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "row_index,model_id,tokens_in,tokens_out,datacenter_id,\
         co2eq_p5_g,co2eq_p50_g,co2eq_p95_g,energy_wh_p50,water_l_p50,audit_id"
    )?;
    for r in rows {
        writeln!(
            writer,
            "{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{}",
            r.row_index,
            csv_escape(&r.model_id),
            r.tokens_in,
            r.tokens_out,
            r.datacenter_id.as_deref().unwrap_or(""),
            r.co2eq_p5_g,
            r.co2eq_p50_g,
            r.co2eq_p95_g,
            r.energy_wh_p50,
            r.water_l_p50,
            r.audit_id,
        )?;
    }
    writer.flush()?;
    Ok(())
}

/// Échappe un champ contenant virgule / guillemet / saut de ligne.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Vérification post-traitement
// ─────────────────────────────────────────────────────────────────────────────

/// Lève une erreur si trop de lignes rejetées (>50%).
///
/// `usize as f64` ne perd pas de précision tant que la valeur tient sur
/// 52 bits — borné ici par `MAX_BATCH_ROWS = 1000`.
#[allow(clippy::cast_precision_loss)]
pub fn check_rejection_ratio(rejected: usize, total: usize) -> BatchResult<()> {
    if total == 0 {
        return Ok(());
    }
    let ratio = rejected as f64 / total as f64;
    if ratio > REJECTION_RATIO_LIMIT {
        return Err(BatchError::TooManyRejections {
            rejected,
            total,
            limit_pct: REJECTION_RATIO_LIMIT_PCT,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // `Write` est déjà importé via le `use` du module parent (BufReader,
    // BufWriter, Write), donc inutile de le ré-importer ici.

    fn write_csv(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("input.csv");
        let mut f = File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (tmp, path)
    }

    #[test]
    fn parse_minimal_valid_csv() {
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
claude-3-5-sonnet,200,1000,aws-eu-west-3-paris
";
        let (_tmp, path) = write_csv(csv);
        let rows = parse_csv(&path).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].model_id, "gpt-4o-mini");
        assert_eq!(rows[0].tokens_in, 100);
        assert!(rows[0].datacenter_id.is_none());
        assert_eq!(
            rows[1].datacenter_id.as_deref(),
            Some("aws-eu-west-3-paris")
        );
    }

    #[test]
    fn parse_handles_blank_lines() {
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,

claude-3-5-sonnet,200,1000,
";
        let (_tmp, path) = write_csv(csv);
        let rows = parse_csv(&path).unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn parse_rejects_missing_header() {
        let csv = "\
gpt-4o-mini,100,500,
";
        let (_tmp, path) = write_csv(csv);
        // Sans header, la 1ère ligne est consommée comme header → parse échoue
        // soit sur le header (colonnes != attendues), soit sur des records absents.
        let result = parse_csv(&path);
        assert!(result.is_err(), "doit échouer sans header");
    }

    #[test]
    fn parse_rejects_wrong_header_name() {
        let csv = "\
modelid,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
";
        let (_tmp, path) = write_csv(csv);
        let err = parse_csv(&path).unwrap_err();
        assert!(matches!(err, BatchError::Format(_)));
    }

    #[test]
    fn parse_rejects_non_integer_tokens() {
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,abc,500,
";
        let (_tmp, path) = write_csv(csv);
        assert!(parse_csv(&path).is_err());
    }

    #[test]
    fn parse_rejects_empty_data() {
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
";
        let (_tmp, path) = write_csv(csv);
        let err = parse_csv(&path).unwrap_err();
        assert!(matches!(err, BatchError::EmptyBatch));
    }

    #[test]
    fn parse_caps_at_max_rows() {
        let mut csv = String::from("model_id,tokens_in,tokens_out,datacenter_id\n");
        for i in 0..MAX_BATCH_ROWS + 5 {
            csv.push_str(&format!("gpt-4o-mini,{},{},\n", i, i + 1));
        }
        let (_tmp, path) = write_csv(&csv);
        let err = parse_csv(&path).unwrap_err();
        assert!(matches!(err, BatchError::TooManyRows { .. }));
    }

    #[test]
    fn parse_file_not_found() {
        let err = parse_csv(Path::new("/nonexistent/path.csv")).unwrap_err();
        assert!(matches!(err, BatchError::FileNotFound(_)));
    }

    #[test]
    fn export_writes_header_and_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let out_path = tmp.path().join("out.csv");
        let rows = vec![
            BatchOutputRow {
                row_index: 1,
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out: 500,
                datacenter_id: None,
                co2eq_p5_g: 1.5,
                co2eq_p50_g: 2.1,
                co2eq_p95_g: 2.8,
                energy_wh_p50: 0.4,
                water_l_p50: 0.0012,
                audit_id: 1,
            },
            BatchOutputRow {
                row_index: 2,
                model_id: "claude-3-5-sonnet".into(),
                tokens_in: 200,
                tokens_out: 1000,
                datacenter_id: Some("aws-eu-west-3-paris".into()),
                co2eq_p5_g: 8.2,
                co2eq_p50_g: 11.4,
                co2eq_p95_g: 15.1,
                energy_wh_p50: 4.2,
                water_l_p50: 0.013,
                audit_id: 2,
            },
        ];
        export_results_csv(&out_path, &rows).unwrap();
        let content = std::fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3, "header + 2 rows");
        assert!(lines[0].starts_with("row_index,model_id"));
        assert!(lines[1].contains("gpt-4o-mini"));
        assert!(lines[2].contains("aws-eu-west-3-paris"));
    }

    #[test]
    fn check_rejection_ratio_accepts_below_50pct() {
        assert!(check_rejection_ratio(0, 10).is_ok());
        assert!(check_rejection_ratio(4, 10).is_ok());
        assert!(check_rejection_ratio(5, 10).is_ok());
    }

    #[test]
    fn check_rejection_ratio_rejects_above_50pct() {
        let err = check_rejection_ratio(6, 10).unwrap_err();
        assert!(matches!(err, BatchError::TooManyRejections { .. }));
    }

    #[test]
    fn csv_escape_handles_quotes_and_commas() {
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        assert_eq!(csv_escape("a\"b"), "\"a\"\"b\"");
    }
}
