//! DD-68: on-demand read-only goal sources; no resident MCP daemon.
mod config;
mod http;
mod imports;
mod jsonc;
mod schema;
mod transport;
pub use config::{Binding, Server, Settings};
pub use imports::{
    Applied, Preview, Record, Reference, apply, context, discover, preview, references,
};
pub const MAX_RESPONSE: usize = 256 * 1024;
pub const MAX_CONTEXT: usize = 48 * 1024;
pub use imports::{load, store};
