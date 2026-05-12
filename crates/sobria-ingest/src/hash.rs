//! Hashing SHA-256 en streaming.
//!
//! Conçu pour traiter des fichiers ≥ 5 GB (ComparIA) sans saturer la mémoire :
//! lecture par chunks de 64 KiB, mise à jour incrémentale du hasher.
//!
//! # Exemple
//!
//! ```no_run
//! # async fn _doc() -> Result<(), Box<dyn std::error::Error>> {
//! use std::path::Path;
//! use sobria_ingest::hash;
//!
//! let digest = hash::sha256_file(Path::new("data/copper/comparia/conversations.parquet")).await?;
//! assert_eq!(digest.len(), 64);
//! # Ok(()) }
//! ```

use std::path::Path;

use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::error::IngestResult;

/// Taille de buffer pour la lecture streaming (64 KiB — bon compromis I/O).
const CHUNK_SIZE: usize = 64 * 1024;

/// Encode un slice de bytes en chaîne hexadécimale minuscule.
#[must_use]
pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize]);
        out.push(HEX[(b & 0x0f) as usize]);
    }
    // SAFETY (en termes de logique) : `out` ne contient que des caractères ASCII.
    String::from_utf8(out).expect("hex_encode invariant: caractères hex ASCII")
}

/// Hashe un fichier en streaming et retourne le digest hexadécimal sur 64 caractères.
pub async fn sha256_file(path: &Path) -> IngestResult<String> {
    let file = tokio::fs::File::open(path).await?;
    sha256_reader(file).await
}

/// Hashe un flux `AsyncRead` quelconque en streaming.
pub async fn sha256_reader<R>(mut reader: R) -> IngestResult<String>
where
    R: AsyncRead + Unpin,
{
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_encode(&hasher.finalize()))
}

/// Vérifie qu'un fichier correspond à un hash hexadécimal attendu.
///
/// La comparaison est insensible à la casse (le digest peut être minuscule
/// ou majuscule en entrée).
pub async fn verify_file(path: &Path, expected_hex: &str) -> IngestResult<bool> {
    let actual = sha256_file(path).await?;
    Ok(actual.eq_ignore_ascii_case(expected_hex))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    use hex_literal::hex;

    /// Vecteurs RFC 6234 + cas connus.
    const VECTORS: &[(&[u8], [u8; 32])] = &[
        // Chaîne vide
        (
            b"",
            hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        ),
        // "abc"
        (
            b"abc",
            hex!("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        ),
        // "hello world"
        (
            b"hello world",
            hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"),
        ),
    ];

    /// Les vecteurs RFC 6234 sont validés via [`sha256_file`] (qui appelle
    /// [`sha256_reader`] en interne).
    #[tokio::test]
    async fn sha256_known_vectors_via_file() {
        for (input, expected) in VECTORS {
            let mut tmp = tempfile::NamedTempFile::new().unwrap();
            tmp.write_all(input).unwrap();
            tmp.flush().unwrap();
            let digest = sha256_file(tmp.path()).await.unwrap();
            assert_eq!(digest, hex_encode(expected), "input = {input:?}");
        }
    }

    #[tokio::test]
    async fn verify_file_ok() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"abc").unwrap();
        tmp.flush().unwrap();
        let ok = verify_file(
            tmp.path(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        )
        .await
        .unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn verify_file_case_insensitive() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"abc").unwrap();
        tmp.flush().unwrap();
        let ok = verify_file(
            tmp.path(),
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD",
        )
        .await
        .unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn verify_file_rejects_mismatch() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"abc").unwrap();
        tmp.flush().unwrap();
        let ok = verify_file(tmp.path(), &"0".repeat(64)).await.unwrap();
        assert!(!ok);
    }

    #[tokio::test]
    async fn sha256_file_missing_returns_io_err() {
        let res = sha256_file(std::path::Path::new("/does/not/exist/at/all/xyz")).await;
        assert!(res.is_err());
    }

    #[test]
    fn hex_encode_known_values() {
        assert_eq!(hex_encode(&[0x00]), "00");
        assert_eq!(hex_encode(&[0xff]), "ff");
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    /// Property : hash dépend de l'intégralité du contenu — modifier un byte
    /// change le digest.
    #[tokio::test]
    async fn modifying_byte_changes_digest() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"Hello, Sobr.ia").unwrap();
        tmp.flush().unwrap();
        let h1 = sha256_file(tmp.path()).await.unwrap();

        let mut tmp2 = tempfile::NamedTempFile::new().unwrap();
        tmp2.write_all(b"Hello, Sobr.iA").unwrap(); // dernière lettre changée
        tmp2.flush().unwrap();
        let h2 = sha256_file(tmp2.path()).await.unwrap();

        assert_ne!(h1, h2);
    }
}
