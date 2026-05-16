//! Binaire `sobria-bridge` — boucle stdin/stdout Native Messaging.
//!
//! Lance la lecture jusqu'à EOF, traite chaque requête via [`handle_request`]
//! et répond en length-prefixed sur stdout. Pour le protocole et la logique
//! métier, voir [`sobria_bridge`] (lib).

use anyhow::Result;
use sobria_bridge::{handle_request, read_message, write_message};

fn main() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();

    while let Some(req) = read_message(&mut input)? {
        let resp = handle_request(req);
        write_message(&mut output, &resp)?;
    }
    Ok(())
}
