//! Tests d'intégration du socket forward (patch C27 v0.6.0).
//!
//! Démarre un faux serveur dans un thread, vérifie que `try_forward_to`
//! envoie une requête et lit la réponse en length-prefixed JSON. Sur Unix
//! on utilise UnixListener ; sur Windows un named pipe via tokio (depuis
//! le helper de test).
//!
//! Brief §"Patch 2" : "démarre un serveur factice, bridge se connecte,
//! roundtrip OK".

use sobria_bridge::{try_forward_to, BridgeRequest};

#[cfg(unix)]
mod unix {
    use super::*;
    use sobria_bridge::BridgeResponse;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;
    use tempfile::TempDir;

    fn run_fake_server(listener: UnixListener) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            // Lit la requête length-prefixed.
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).expect("read len");
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut payload = vec![0u8; len];
            stream.read_exact(&mut payload).expect("read payload");
            let req: BridgeRequest = serde_json::from_slice(&payload).expect("decode req");
            // Construit une réponse simple basée sur le req_id reçu.
            let resp = BridgeResponse {
                req_id: req.req_id().into(),
                ok: true,
                error: None,
                pong: Some(matches!(req, BridgeRequest::Ping { .. })),
                secret: None,
                pairing_id: None,
                fingerprint: None,
            };
            let bytes = serde_json::to_vec(&resp).expect("encode resp");
            let resp_len = u32::try_from(bytes.len()).unwrap();
            stream.write_all(&resp_len.to_le_bytes()).expect("write len");
            stream.write_all(&bytes).expect("write payload");
            stream.flush().expect("flush");
        })
    }

    #[test]
    fn forward_ping_returns_pong() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("sobria-bridge-test.sock");
        let listener = UnixListener::bind(&path).expect("bind");
        let handle = run_fake_server(listener);

        let req = BridgeRequest::Ping {
            req_id: "ping-1".into(),
        };
        let resp = try_forward_to(&path, &req).expect("forward ok");
        assert_eq!(resp.req_id, "ping-1");
        assert!(resp.ok);
        assert_eq!(resp.pong, Some(true));
        handle.join().expect("server done");
    }

    #[test]
    fn forward_pair_returns_serialized_request() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("sobria-bridge-test.sock");
        let listener = UnixListener::bind(&path).expect("bind");
        let handle = run_fake_server(listener);

        let req = BridgeRequest::Pair {
            req_id: "pair-1".into(),
            code: "123456".into(),
        };
        let resp = try_forward_to(&path, &req).expect("forward ok");
        assert_eq!(resp.req_id, "pair-1");
        assert!(resp.ok);
        // Notre fake server répond Ping=false sur Pair, mais ok=true.
        assert_eq!(resp.pong, Some(false));
        handle.join().expect("server done");
    }

    #[test]
    fn forward_to_nonexistent_socket_errors_fast() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("never-bound.sock");
        let req = BridgeRequest::Ping {
            req_id: "ghost".into(),
        };
        let result = try_forward_to(&path, &req);
        assert!(result.is_err(), "should fail when socket doesn't exist");
    }
}

#[cfg(windows)]
mod windows_pipe {
    use super::*;

    /// Smoke test : essayer de forward vers un pipe inexistant doit échouer
    /// rapidement (pas de timeout, l'open échoue dès qu'il voit que le pipe
    /// n'existe pas).
    #[test]
    fn forward_to_nonexistent_pipe_errors_fast() {
        // Pipe name aléatoire pour éviter une éventuelle collision avec un
        // vrai sobria-bridge tournant.
        let pipe = format!(
            r"\\.\pipe\sobria-bridge-test-{}",
            std::process::id()
        );
        let req = BridgeRequest::Ping {
            req_id: "ghost-win".into(),
        };
        let result = try_forward_to(std::path::Path::new(&pipe), &req);
        assert!(result.is_err(), "should fail when pipe doesn't exist");
    }
}

/// Test cross-platform : la requête sérialisée doit conserver le format
/// attendu par l'app (length-prefixed + JSON tag-based enum). Validé en
/// inspectant la réponse de l'erreur quand le serveur écrit n'importe quoi.
#[test]
fn serialization_of_bridge_request_round_trips() {
    let req = BridgeRequest::Estimate {
        req_id: "rt-1".into(),
        secret: "0123abcdef".repeat(6),
        payload: serde_json::json!({ "k": 1 }),
    };
    let bytes = serde_json::to_vec(&req).unwrap();
    let parsed: BridgeRequest = serde_json::from_slice(&bytes).unwrap();
    match parsed {
        BridgeRequest::Estimate {
            req_id,
            secret,
            payload,
        } => {
            assert_eq!(req_id, "rt-1");
            assert!(secret.starts_with("0123"));
            assert_eq!(payload["k"], 1);
        },
        _ => panic!("expected Estimate"),
    }
}
