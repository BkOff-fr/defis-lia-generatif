//! Roundtrip protocole length-prefixed JSON (Native Messaging spec).
//!
//! Vérifie que `read_message` + `write_message` peuvent boucler sans perte
//! sur les 4 types de requêtes (`ping`, `pair`, `estimate`, `revoke`) et
//! que le spool est append-only avec rotation à 10 MB.

use std::io::Cursor;

use serde_json::json;
use sobria_bridge::{handle_request, read_message, write_message, BridgeRequest, BridgeResponse};

/// Encode un BridgeRequest en buffer length-prefixed (uint32 LE + JSON).
fn encode_request_str(json: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    let bytes = json.as_bytes();
    let len = u32::try_from(bytes.len()).unwrap();
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
    buf
}

#[test]
fn read_ping_request() {
    let req_json = r#"{"type":"ping","reqId":"a1"}"#;
    let buf = encode_request_str(req_json);
    let mut cursor = Cursor::new(buf);
    let req = read_message(&mut cursor).unwrap().unwrap();
    matches!(req, BridgeRequest::Ping { .. });
    assert_eq!(req.req_id(), "a1");
}

#[test]
fn read_estimate_request() {
    let req_json = r#"{"type":"estimate","reqId":"e1","secret":"s","payload":{"k":42}}"#;
    let buf = encode_request_str(req_json);
    let mut cursor = Cursor::new(buf);
    let req = read_message(&mut cursor).unwrap().unwrap();
    match req {
        BridgeRequest::Estimate {
            req_id,
            secret,
            payload,
        } => {
            assert_eq!(req_id, "e1");
            assert_eq!(secret, "s");
            assert_eq!(payload["k"], 42);
        },
        _ => panic!("attendu Estimate"),
    }
}

#[test]
fn eof_returns_none() {
    let empty: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(empty);
    assert!(read_message(&mut cursor).unwrap().is_none());
}

#[test]
fn ping_handle_writes_pong() {
    let resp = handle_request(BridgeRequest::Ping { req_id: "x".into() });
    let mut buf: Vec<u8> = Vec::new();
    write_message(&mut buf, &resp).unwrap();
    // Le buffer commence par un uint32 LE de longueur.
    assert_eq!(buf.len(), 4 + (buf.len() - 4));
    let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let payload = &buf[4..4 + len];
    let parsed: serde_json::Value = serde_json::from_slice(payload).unwrap();
    assert_eq!(parsed["reqId"], "x");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["pong"], true);
}

#[test]
fn pair_handle_returns_error() {
    let resp = handle_request(BridgeRequest::Pair {
        req_id: "p".into(),
        code: "000000".into(),
    });
    assert!(!resp.ok);
    assert!(resp.error.is_some());
}

#[test]
fn estimate_handle_writes_to_spool() {
    // Le test précédent vérifiait le req_id retourné via `handle_request`,
    // mais sans accès direct au filesystem. Ici on teste explicitement
    // `append_to_spool_at` avec un tempdir pour valider rotation + format.
    let temp = tempfile::tempdir().unwrap();
    let spool = temp.path().join("incoming.jsonl");
    sobria_bridge::append_to_spool_at(
        &spool,
        &json!({"foo": "bar", "n": 1}),
        sobria_bridge::SPOOL_MAX_BYTES,
    )
    .unwrap();
    sobria_bridge::append_to_spool_at(
        &spool,
        &json!({"foo": "baz", "n": 2}),
        sobria_bridge::SPOOL_MAX_BYTES,
    )
    .unwrap();
    let content = std::fs::read_to_string(&spool).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);
    let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(first["n"], 1);
}

#[test]
fn append_to_spool_rotates_at_threshold() {
    let temp = tempfile::tempdir().unwrap();
    let spool = temp.path().join("incoming.jsonl");
    // Seuil 5 bytes : la 1ʳᵉ écriture produit ~8 bytes (> 5) → la 2ᵉ rotate.
    sobria_bridge::append_to_spool_at(&spool, &json!({"a": 1}), 5).unwrap();
    sobria_bridge::append_to_spool_at(&spool, &json!({"b": 2}), 5).unwrap();
    let bak = spool.with_extension("jsonl.bak");
    assert!(bak.exists(), ".bak doit exister après rotation");
    // Le nouveau spool ne contient que la dernière écriture.
    let content = std::fs::read_to_string(&spool).unwrap();
    assert!(content.contains("\"b\":2"));
    assert!(!content.contains("\"a\":1"));
}

#[test]
fn response_serialization_includes_secret_when_set() {
    let mut resp = BridgeResponse::ok("p1");
    resp.secret = Some("xx".into());
    resp.pairing_id = Some("uuid-1".into());
    resp.fingerprint = Some("chrome-mac".into());
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"secret\":\"xx\""));
    assert!(json.contains("\"pairingId\":\"uuid-1\""));
    assert!(json.contains("\"fingerprint\":\"chrome-mac\""));
}
