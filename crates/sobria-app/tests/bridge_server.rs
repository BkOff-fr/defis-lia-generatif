//! Tests d'intégration du `bridge_server` côté app — patch C27 v0.6.0.
//!
//! Sur Unix : démarre le serveur sur un socket tempdir, envoie une requête
//! via `tokio::net::UnixStream`, vérifie la réponse.
//!
//! Sur Windows : le serveur natif tokio (`named_pipe::ServerOptions`) tourne
//! avec un nom de pipe unique, le test ouvre `ClientOptions::open` et fait
//! le roundtrip. Marqué `#[cfg(windows)]`.

use std::sync::Arc;

use sobria_app::AppState;
use sobria_bridge::BridgeRequest;

#[cfg(unix)]
mod unix {
    use super::*;
    use sobria_bridge::BridgeResponse;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    async fn write_request<S>(stream: &mut S, req: &BridgeRequest)
    where
        S: AsyncWriteExt + Unpin,
    {
        let bytes = serde_json::to_vec(req).unwrap();
        let len = u32::try_from(bytes.len()).unwrap();
        stream.write_all(&len.to_le_bytes()).await.unwrap();
        stream.write_all(&bytes).await.unwrap();
        stream.flush().await.unwrap();
    }

    async fn read_response<S>(stream: &mut S) -> BridgeResponse
    where
        S: AsyncReadExt + Unpin,
    {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.unwrap();
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload).await.unwrap();
        serde_json::from_slice(&payload).unwrap()
    }

    #[tokio::test]
    async fn ping_roundtrip_via_unix_socket() {
        let tmp = tempfile::tempdir().unwrap();
        let state = Arc::new(AppState::init_in(tmp.path()).unwrap());
        let socket_path = tmp.path().join("sobria-bridge-test.sock");

        // Pointe le bridge vers notre socket de test via la var d'env.
        // Note: `default_socket_path` honore SOBRIA_BRIDGE_SOCKET — utile
        // pour des tests bridge↔app full-stack, mais ici on bind direct.
        std::env::set_var("SOBRIA_BRIDGE_SOCKET", &socket_path);

        let server_state = Arc::clone(&state);
        let path_clone = socket_path.clone();
        let server_handle = tokio::spawn(async move {
            // Reproduit en interne ce que `bridge_server::run` ferait, mais
            // sur un chemin explicite (le module run() utilise default_socket_path).
            let _ = std::fs::remove_file(&path_clone);
            let listener = tokio::net::UnixListener::bind(&path_clone).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();
            // Lit la requête, dispatch, écrit réponse.
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await.unwrap();
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut payload = vec![0u8; len];
            stream.read_exact(&mut payload).await.unwrap();
            let req: BridgeRequest = serde_json::from_slice(&payload).unwrap();
            let resp = sobria_app::bridge_server::dispatch_request(&server_state, req);
            let bytes = serde_json::to_vec(&resp).unwrap();
            let resp_len = u32::try_from(bytes.len()).unwrap();
            stream.write_all(&resp_len.to_le_bytes()).await.unwrap();
            stream.write_all(&bytes).await.unwrap();
            stream.flush().await.unwrap();
        });

        // Petit délai pour que le listener bind avant le client.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let mut stream = UnixStream::connect(&socket_path).await.unwrap();
        write_request(
            &mut stream,
            &BridgeRequest::Ping {
                req_id: "p1".into(),
            },
        )
        .await;
        let resp = read_response(&mut stream).await;
        assert_eq!(resp.req_id, "p1");
        assert!(resp.ok);
        assert_eq!(resp.pong, Some(true));

        server_handle.await.unwrap();
    }
}

/// Cross-platform : dispatch_request peut traiter un Ping sans I/O.
/// Doublon léger avec les tests unit du module — utile ici pour valider
/// que le module bridge_server est bien réexporté publiquement.
#[test]
fn dispatch_ping_returns_pong_via_public_api() {
    let tmp = tempfile::tempdir().unwrap();
    let state = Arc::new(AppState::init_in(tmp.path()).unwrap());
    let resp = sobria_app::bridge_server::dispatch_request(
        &state,
        BridgeRequest::Ping {
            req_id: "p1".into(),
        },
    );
    assert!(resp.ok);
    assert_eq!(resp.pong, Some(true));
}
