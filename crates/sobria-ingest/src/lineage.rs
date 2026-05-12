//! Lignée — traçabilité Copper → Silver → Gold.
//!
//! Chaque ligne de la couche Gold est traçable jusqu'à son hash Copper
//! d'origine. C'est le pilier scientifique du pipeline médaillon
//! (ADR-0009).
//!
//! Les structures sont sérialisées en JSON pour intégration dans
//! `datasheet.jsonld` (Gebru et al. 2018) et dans le manifest Gold final.
//!
//! # Garantie principale
//!
//! Si un fichier Copper a contribué à une entité Silver, il apparaît dans
//! `SilverLineage::copper_refs`. Si une entité Silver a contribué au Gold,
//! elle apparaît dans `GoldLineage::silver_inputs`. **Aucun hash Copper ne
//! peut donc être perdu en remontant du Gold vers les sources.**

use std::{
    collections::BTreeSet,
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{IngestError, IngestResult};

/// Référence à un fichier Copper individuel.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub struct CopperRef {
    /// Identifiant de la source (ex: `"comparia"`).
    pub source_id: String,
    /// Chemin relatif du manifest qui décrit ce fichier
    /// (ex: `"copper/comparia/2026-05-12/manifest.json"`).
    pub manifest_path: PathBuf,
    /// Nom du fichier dans le snapshot (ex: `"conversations.parquet"`).
    pub file_name: String,
    /// SHA-256 hexadécimal sur 64 caractères minuscules.
    pub file_sha256: String,
}

impl CopperRef {
    /// Valide les invariants de la référence.
    pub fn validate(&self) -> IngestResult<()> {
        if self.source_id.trim().is_empty() {
            return Err(IngestError::schema("CopperRef.source_id vide"));
        }
        if self.file_name.trim().is_empty() {
            return Err(IngestError::schema("CopperRef.file_name vide"));
        }
        if self.file_sha256.len() != 64
            || !self.file_sha256.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(IngestError::schema(format!(
                "CopperRef.file_sha256 invalide : {:?}",
                self.file_sha256
            )));
        }
        Ok(())
    }
}

/// Lignée d'une entité Silver — décrit quels fichiers Copper l'ont produite.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SilverLineage {
    /// Nom de l'entité (ex: `"comparia_conversations"`).
    pub entity: String,
    /// Version du schéma Silver utilisé (ex: `"v1"`).
    pub schema_version: String,
    /// Chemin du Parquet écrit (relatif à `data/silver/`).
    pub silver_path: PathBuf,
    /// Référence(s) Copper d'origine. Toujours non vide.
    pub copper_refs: Vec<CopperRef>,
    /// Nombre de lignes écrites dans le Parquet.
    pub row_count: u64,
    /// Horodatage UTC d'écriture.
    pub written_at: DateTime<Utc>,
}

impl SilverLineage {
    /// Valide les invariants.
    pub fn validate(&self) -> IngestResult<()> {
        if self.entity.trim().is_empty() {
            return Err(IngestError::schema("SilverLineage.entity vide"));
        }
        if self.copper_refs.is_empty() {
            return Err(IngestError::BrokenLineage(format!(
                "entité {:?} sans CopperRef — impossible (lineage rompu)",
                self.entity
            )));
        }
        for r in &self.copper_refs {
            r.validate()?;
        }
        Ok(())
    }
}

/// Lignée du Gold — assemble toutes les entités Silver qui l'ont produit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GoldLineage {
    /// Entités Silver consommées pour produire le Gold.
    pub silver_inputs: Vec<SilverLineage>,
    /// Artefacts Gold produits (ex: `["referentiel.sqlite", "analytics.parquet"]`).
    pub gold_artifacts: Vec<String>,
    /// Horodatage UTC d'assemblage du Gold.
    pub assembled_at: DateTime<Utc>,
}

impl GoldLineage {
    /// Construit un Gold lineage vide horodaté maintenant.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            silver_inputs: Vec::new(),
            gold_artifacts: Vec::new(),
            assembled_at: Utc::now(),
        }
    }

    /// Ajoute une entité Silver à la lignée.
    pub fn add_silver(&mut self, silver: SilverLineage) {
        self.silver_inputs.push(silver);
    }

    /// Ajoute un artefact Gold produit.
    pub fn add_artifact(&mut self, name: impl Into<String>) {
        self.gold_artifacts.push(name.into());
    }

    /// Itère sur tous les hashes Copper uniques référencés (déduplication).
    pub fn copper_hashes(&self) -> impl Iterator<Item = &str> {
        let mut seen = BTreeSet::new();
        let mut all: Vec<&str> = Vec::new();
        for silver in &self.silver_inputs {
            for r in &silver.copper_refs {
                if seen.insert(r.file_sha256.as_str()) {
                    all.push(r.file_sha256.as_str());
                }
            }
        }
        all.into_iter()
    }

    /// Liste des `source_id` distincts ayant contribué.
    pub fn source_ids(&self) -> Vec<&str> {
        let mut seen = BTreeSet::new();
        for silver in &self.silver_inputs {
            for r in &silver.copper_refs {
                seen.insert(r.source_id.as_str());
            }
        }
        seen.into_iter().collect()
    }

    /// Valide la lignée complète.
    pub fn validate(&self) -> IngestResult<()> {
        for silver in &self.silver_inputs {
            silver.validate()?;
        }
        if self.gold_artifacts.is_empty() {
            return Err(IngestError::schema(
                "GoldLineage.gold_artifacts vide — un Gold doit produire au moins un artefact",
            ));
        }
        Ok(())
    }

    /// Sérialise en JSON-LD compatible PROV-O (W3C) + schema.org/Dataset.
    ///
    /// La sortie peut être incluse dans `datasheet.jsonld`.
    #[must_use]
    pub fn to_jsonld(&self) -> serde_json::Value {
        let sources: Vec<serde_json::Value> = self
            .source_ids()
            .iter()
            .map(|id| serde_json::json!({ "@id": format!("sobria:source:{id}") }))
            .collect();
        let copper_files: Vec<serde_json::Value> = self
            .silver_inputs
            .iter()
            .flat_map(|s| s.copper_refs.iter())
            .map(|r| {
                serde_json::json!({
                    "@type": "prov:Entity",
                    "@id": format!("sobria:copper:{}", r.file_sha256),
                    "schema:identifier": r.file_sha256,
                    "schema:name": r.file_name,
                    "prov:wasAttributedTo": format!("sobria:source:{}", r.source_id),
                })
            })
            .collect();
        let silver_entities: Vec<serde_json::Value> = self
            .silver_inputs
            .iter()
            .map(|s| {
                let copper_ids: Vec<String> = s
                    .copper_refs
                    .iter()
                    .map(|r| format!("sobria:copper:{}", r.file_sha256))
                    .collect();
                serde_json::json!({
                    "@type": "prov:Entity",
                    "@id": format!("sobria:silver:{}", s.entity),
                    "schema:name": s.entity,
                    "schema:version": s.schema_version,
                    "prov:wasDerivedFrom": copper_ids,
                })
            })
            .collect();

        serde_json::json!({
            "@context": {
                "prov": "http://www.w3.org/ns/prov#",
                "schema": "https://schema.org/",
                "sobria": "https://sobr.ia/vocab#"
            },
            "@type": ["prov:Entity", "schema:Dataset"],
            "@id": "sobria:gold",
            "schema:dateCreated": self.assembled_at.to_rfc3339(),
            "schema:distribution": self.gold_artifacts,
            "schema:isBasedOn": sources,
            "prov:wasDerivedFrom": silver_entities,
            "sobria:copperReferences": copper_files
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn cref(source: &str, file: &str, hash_byte: char) -> CopperRef {
        CopperRef {
            source_id: source.into(),
            manifest_path: PathBuf::from(format!("copper/{source}/2026-05-12/manifest.json")),
            file_name: file.into(),
            file_sha256: std::iter::repeat(hash_byte).take(64).collect(),
        }
    }

    fn sample_silver() -> SilverLineage {
        SilverLineage {
            entity: "comparia_conversations".into(),
            schema_version: "v1".into(),
            silver_path: PathBuf::from("silver/comparia/conversations.parquet"),
            copper_refs: vec![cref("comparia", "conversations.parquet", 'a')],
            row_count: 250_000,
            written_at: Utc::now(),
        }
    }

    fn sample_gold() -> GoldLineage {
        let mut g = GoldLineage::empty();
        g.add_silver(sample_silver());
        g.add_silver(SilverLineage {
            entity: "iris_consommation".into(),
            schema_version: "v1".into(),
            silver_path: PathBuf::from("silver/rte-iris/consommation.parquet"),
            copper_refs: vec![cref("rte-iris", "iris.csv", 'b')],
            row_count: 50_000,
            written_at: Utc::now(),
        });
        g.add_artifact("referentiel.sqlite");
        g.add_artifact("analytics.parquet");
        g
    }

    #[test]
    fn copper_ref_validates() {
        assert!(cref("comparia", "x.parquet", 'a').validate().is_ok());
    }

    #[test]
    fn copper_ref_rejects_bad_hash() {
        let mut r = cref("c", "x", 'a');
        r.file_sha256 = "abc".into();
        assert!(r.validate().is_err());
    }

    #[test]
    fn silver_lineage_rejects_empty_refs() {
        let mut s = sample_silver();
        s.copper_refs.clear();
        assert!(s.validate().is_err());
    }

    #[test]
    fn gold_validates_and_has_artifacts() {
        assert!(sample_gold().validate().is_ok());
    }

    #[test]
    fn gold_rejects_without_artifacts() {
        let mut g = GoldLineage::empty();
        g.add_silver(sample_silver());
        assert!(g.validate().is_err());
    }

    #[test]
    fn gold_copper_hashes_deduplicates() {
        let mut g = sample_gold();
        // Ajoute une entité Silver qui partage un hash Copper avec une autre.
        g.add_silver(SilverLineage {
            entity: "comparia_votes".into(),
            schema_version: "v1".into(),
            silver_path: PathBuf::from("silver/comparia/votes.parquet"),
            copper_refs: vec![cref("comparia", "conversations.parquet", 'a')],
            row_count: 150_000,
            written_at: Utc::now(),
        });
        let hashes: Vec<&str> = g.copper_hashes().collect();
        assert_eq!(hashes.len(), 2, "duplicats éliminés"); // 'a' une fois, 'b' une fois
    }

    #[test]
    fn gold_source_ids_distinct() {
        let g = sample_gold();
        let mut ids = g.source_ids();
        ids.sort_unstable();
        assert_eq!(ids, vec!["comparia", "rte-iris"]);
    }

    #[test]
    fn gold_serializes_round_trip() {
        let g = sample_gold();
        let json = serde_json::to_string(&g).unwrap();
        let back: GoldLineage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.silver_inputs.len(), g.silver_inputs.len());
        assert_eq!(back.gold_artifacts, g.gold_artifacts);
    }

    #[test]
    fn jsonld_contains_required_keys() {
        let g = sample_gold();
        let ld = g.to_jsonld();
        assert!(ld.get("@context").is_some());
        assert_eq!(ld["@type"], serde_json::json!(["prov:Entity", "schema:Dataset"]));
        assert!(ld["schema:dateCreated"].is_string());
        assert!(ld["schema:distribution"].is_array());
    }

    #[test]
    fn jsonld_links_silver_to_copper() {
        let g = sample_gold();
        let ld = g.to_jsonld();
        let silver_array = ld["prov:wasDerivedFrom"].as_array().unwrap();
        assert!(!silver_array.is_empty());
        for s in silver_array {
            assert!(s["prov:wasDerivedFrom"].is_array());
        }
    }

    proptest! {
        /// Garantie principale du module : aucun hash Copper ne disparaît
        /// en remontant du Gold vers les sources.
        #[test]
        fn prop_gold_preserves_all_copper_hashes(
            n_silver in 1usize..=5,
            n_refs_per_silver in 1usize..=4,
        ) {
            let mut gold = GoldLineage::empty();
            let mut all_hashes: BTreeSet<String> = BTreeSet::new();

            for i in 0..n_silver {
                let refs: Vec<CopperRef> = (0..n_refs_per_silver)
                    .map(|j| {
                        let h: String = format!("{i:032x}{j:032x}");
                        all_hashes.insert(h.clone());
                        CopperRef {
                            source_id: format!("source_{i}"),
                            manifest_path: PathBuf::from("manifest.json"),
                            file_name: format!("f{j}.parquet"),
                            file_sha256: h,
                        }
                    })
                    .collect();
                gold.add_silver(SilverLineage {
                    entity: format!("entity_{i}"),
                    schema_version: "v1".into(),
                    silver_path: PathBuf::from("x.parquet"),
                    copper_refs: refs,
                    row_count: 1,
                    written_at: Utc::now(),
                });
            }
            gold.add_artifact("x");

            let found: BTreeSet<String> = gold
                .copper_hashes()
                .map(std::string::ToString::to_string)
                .collect();
            prop_assert_eq!(found, all_hashes);
        }
    }
}
