//! Binaire `sobria-bridge` — boucle stdin/stdout Native Messaging.
//!
//! Lance la lecture jusqu'à EOF, et pour chaque requête :
//!   1. Tente un forward au socket de l'app desktop (patch C27 v0.6.0)
//!      via [`try_forward_to`]. Si l'app est joignable, `Pair`/`Revoke`
//!      reçoivent une réponse synchrone en temps réel.
//!   2. Si le socket est injoignable, retombe sur [`handle_request`] qui
//!      écrit dans le spool fichier (fallback offline historique).
//!
//! Pour le protocole et la logique métier, voir [`sobria_bridge`] (lib).

use anyhow::Result;
use sobria_bridge::{
    default_socket_path, handle_request, read_message, try_forward_to, write_message,
};

fn main() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();

    let socket = default_socket_path();
    while let Some(req) = read_message(&mut input)? {
        let resp = match try_forward_to(&socket, &req) {
            Ok(r) => r,
            Err(_) => handle_request(req),
        };
        write_message(&mut output, &resp)?;
    }
    Ok(())
}
