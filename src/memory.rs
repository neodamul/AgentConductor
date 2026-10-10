pub mod checkpoints;
mod client;
use crate::state::Store;
pub use client::Client;

/// Optional delivery to an existing loopback daemon with confirmed creation receipts.
pub fn sync(store: &Store, port: u16) -> Result<usize, String> {
    let client = Client::from_environment(port)?;
    let mut count = 0;
    for (id, content) in store.pending_memory()? {
        client.remember_note(&format!("AgentConductor outbox item {id}: {content}"))?;
        store.memory_delivered(id)?;
        count += 1;
    }
    Ok(count)
}
