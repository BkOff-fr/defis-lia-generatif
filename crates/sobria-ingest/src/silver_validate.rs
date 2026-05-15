//! Validation structurelle des entités Silver — voir ADR-0009 §"Silver Layer".
//!
//! Chaque entité Silver est associée à un schéma JSON Schema 2020-12 versionné
//! sous `schemas/silver/<entity>-v<n>.json`. Avant qu'une entité Silver ne soit
//! considérée comme valide, [`validate_silver`] :
//!
//! 1. Charge le schéma embarqué (`include_str!`) correspondant à `entity.name`.
//! 2. Lit le schéma Arrow du Parquet via `polars::LazyFrame::scan_parquet`.
//! 3. Vérifie que toutes les colonnes `required` du JSON Schema sont présentes.
//! 4. Pour chaque colonne `properties` typée, vérifie que le type Arrow est
//!    compatible avec le `"type"` JSON Schema (string→Utf8, integer→Int*, etc.).
//!
//! Les colonnes lineage (`_copper_sha256`, `_ingested_at`) sont systématiques
//! et leur respect est vérifié à 100 %. Les contraintes métier (ex: `code_iris`
//! pour RTE IRIS) sont également validées si présentes dans le schéma.
//!
//! La validation est appelée par les implémentations `promote_silver` de chaque
//! source juste avant de retourner les `SilverEntity` à l'orchestrateur.

use std::{collections::BTreeMap, path::Path};

use serde_json::Value;

use crate::{
    error::{IngestError, IngestResult},
    layer::SilverEntity,
};

/// Schéma Silver embarqué pour `comparia_conversations` (v1).
const SCHEMA_COMPARIA_CONVERSATIONS: &str =
    include_str!("../../../schemas/silver/comparia_conversations-v1.json");

/// Schéma Silver embarqué pour `comparia_votes` (v1).
const SCHEMA_COMPARIA_VOTES: &str = include_str!("../../../schemas/silver/comparia_votes-v1.json");

/// Schéma Silver embarqué pour `comparia_reactions` (v1).
const SCHEMA_COMPARIA_REACTIONS: &str =
    include_str!("../../../schemas/silver/comparia_reactions-v1.json");

/// Schéma Silver embarqué pour `rte_iris_consommation` (v1).
const SCHEMA_RTE_IRIS_CONSOMMATION: &str =
    include_str!("../../../schemas/silver/rte_iris_consommation-v1.json");

/// Retourne le JSON Schema Silver embarqué pour une entité connue.
///
/// Renvoie `None` si l'entité n'a pas de schéma — auquel cas la validation
/// échoue avec une erreur explicite plutôt que de passer silencieusement.
#[must_use]
pub fn embedded_schema(entity_name: &str) -> Option<&'static str> {
    match entity_name {
        "comparia_conversations" => Some(SCHEMA_COMPARIA_CONVERSATIONS),
        "comparia_votes" => Some(SCHEMA_COMPARIA_VOTES),
        "comparia_reactions" => Some(SCHEMA_COMPARIA_REACTIONS),
        "rte_iris_consommation" => Some(SCHEMA_RTE_IRIS_CONSOMMATION),
        _ => None,
    }
}

/// Liste les noms d'entités Silver pour lesquelles un schéma est embarqué.
/// Utile pour l'introspection CLI et les tests d'inventaire.
#[must_use]
pub fn known_entities() -> &'static [&'static str] {
    &[
        "comparia_conversations",
        "comparia_votes",
        "comparia_reactions",
        "rte_iris_consommation",
    ]
}

/// Valide structurellement une entité Silver contre son schéma JSON Schema
/// embarqué.
///
/// `polars` étant synchrone, l'exécution est déléguée à `spawn_blocking` —
/// la fonction reste donc utilisable depuis un contexte tokio sans bloquer
/// le runtime.
pub async fn validate_silver(entity: &SilverEntity) -> IngestResult<()> {
    let schema_str = embedded_schema(&entity.name).ok_or_else(|| {
        IngestError::schema(format!(
            "aucun schéma Silver embarqué pour l'entité « {} » (entités connues : {})",
            entity.name,
            known_entities().join(", ")
        ))
    })?;

    let schema_json: Value = serde_json::from_str(schema_str)?;
    let parquet_path = entity.path.clone();
    let entity_label = entity.name.clone();

    tokio::task::spawn_blocking(move || -> IngestResult<()> {
        validate_parquet_against(&parquet_path, &schema_json, &entity_label)
    })
    .await
    .map_err(|e| IngestError::Other(format!("spawn_blocking validate_silver: {e}")))?
}

/// Variante synchrone testable — utilisée par [`validate_silver`] et les tests
/// proptest. Lit le schéma Arrow du Parquet et vérifie les colonnes `required`
/// + cohérence des types.
pub(crate) fn validate_parquet_against(
    parquet_path: &Path,
    schema: &Value,
    entity_label: &str,
) -> IngestResult<()> {
    use polars::prelude::*;

    if !parquet_path.exists() {
        return Err(IngestError::schema(format!(
            "validation Silver « {entity_label} » : Parquet introuvable {}",
            parquet_path.display()
        )));
    }

    let required: Vec<String> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut lf =
        LazyFrame::scan_parquet(parquet_path, ScanArgsParquet::default()).map_err(|e| {
            IngestError::schema(format!(
                "validation Silver « {entity_label} » : scan_parquet({}): {e}",
                parquet_path.display()
            ))
        })?;
    let arrow_schema = lf.collect_schema().map_err(|e| {
        IngestError::schema(format!(
            "validation Silver « {entity_label} » : collect_schema: {e}"
        ))
    })?;

    let columns: BTreeMap<String, DataType> = arrow_schema
        .iter()
        .map(|(name, dtype)| (name.to_string(), dtype.clone()))
        .collect();

    for col in &required {
        if !columns.contains_key(col) {
            return Err(IngestError::schema(format!(
                "validation Silver « {entity_label} » : colonne requise « {col} » absente du Parquet {} — colonnes présentes : [{}]",
                parquet_path.display(),
                columns.keys().cloned().collect::<Vec<_>>().join(", ")
            )));
        }
    }

    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for (col_name, prop_def) in properties {
            let Some(actual_dtype) = columns.get(col_name) else {
                continue;
            };
            let allowed_types = collect_allowed_json_types(prop_def);
            if allowed_types.is_empty() {
                continue;
            }
            if !allowed_types
                .iter()
                .any(|t| arrow_matches_json_type(actual_dtype, t))
            {
                return Err(IngestError::schema(format!(
                    "validation Silver « {entity_label} » : colonne « {col_name} » \
                     type Arrow {actual_dtype:?} incompatible avec types JSON Schema {allowed_types:?}"
                )));
            }
        }
    }

    Ok(())
}

/// Extrait la liste des types JSON Schema autorisés pour une propriété
/// (`"type": "string"` → `["string"]`, `"type": ["string", "integer"]` →
/// `["string", "integer"]`). Vide si aucun type n'est précisé.
fn collect_allowed_json_types(prop_def: &Value) -> Vec<String> {
    match prop_def.get("type") {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

/// Compatibilité Arrow ↔ JSON Schema 2020-12 (mapping pragmatique).
fn arrow_matches_json_type(arrow: &polars::prelude::DataType, json_type: &str) -> bool {
    use polars::prelude::DataType;
    match json_type {
        "string" => matches!(
            arrow,
            DataType::String
                | DataType::Binary
                | DataType::Categorical(_, _)
                | DataType::Enum(_, _)
        ),
        "integer" => matches!(
            arrow,
            DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
        ),
        "number" => matches!(
            arrow,
            DataType::Float32 | DataType::Float64 | DataType::Decimal(_, _)
        ),
        "boolean" => matches!(arrow, DataType::Boolean),
        "null" => matches!(arrow, DataType::Null),
        // "object", "array" : pas représentables en colonne plate, on laisse passer
        // par défaut (le schéma de table Silver n'est pas censé décrire des structures
        // imbriquées dans le passthrough v1).
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schemas_present_for_all_known_entities() {
        for name in known_entities() {
            let schema = embedded_schema(name)
                .unwrap_or_else(|| panic!("schéma embarqué manquant : {name}"));
            let parsed: Value = serde_json::from_str(schema)
                .unwrap_or_else(|_| panic!("schéma {name} doit être un JSON valide"));
            assert!(parsed.get("$schema").is_some(), "manque $schema : {name}");
            assert!(parsed.get("required").is_some(), "manque required : {name}");
        }
    }

    #[test]
    fn unknown_entity_returns_none() {
        assert!(embedded_schema("inconnu").is_none());
    }

    #[test]
    fn rte_schema_requires_code_iris() {
        let schema = embedded_schema("rte_iris_consommation").unwrap();
        let parsed: Value = serde_json::from_str(schema).unwrap();
        let required: Vec<&str> = parsed["required"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert!(
            required.contains(&"code_iris"),
            "RTE IRIS doit exiger code_iris"
        );
        assert!(required.contains(&"_copper_sha256"));
        assert!(required.contains(&"_ingested_at"));
    }

    #[test]
    fn arrow_string_matches_json_string() {
        assert!(arrow_matches_json_type(
            &polars::prelude::DataType::String,
            "string"
        ));
        assert!(arrow_matches_json_type(
            &polars::prelude::DataType::Int64,
            "integer"
        ));
        assert!(arrow_matches_json_type(
            &polars::prelude::DataType::Float64,
            "number"
        ));
        assert!(!arrow_matches_json_type(
            &polars::prelude::DataType::Int64,
            "string"
        ));
    }

    #[test]
    fn collect_allowed_types_handles_string_or_array() {
        let v = serde_json::json!({ "type": "string" });
        assert_eq!(collect_allowed_json_types(&v), vec!["string".to_string()]);

        let v = serde_json::json!({ "type": ["string", "integer"] });
        assert_eq!(
            collect_allowed_json_types(&v),
            vec!["string".to_string(), "integer".to_string()]
        );

        let v = serde_json::json!({ "description": "no type" });
        assert!(collect_allowed_json_types(&v).is_empty());
    }
}
